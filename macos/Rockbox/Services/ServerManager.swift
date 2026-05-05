//
//  ServerManager.swift
//  Rockbox
//

import Foundation
import Network
import SwiftUI

// MARK: - Observable manager (UI layer)

@MainActor
class ServerManager: ObservableObject {
  static let shared = ServerManager()

  @Published var currentServer: RockboxServerInfo = .localhost
  @Published var discoveredServers: [RockboxServerInfo] = []
  @Published var isScanning = false

  private var scanner: MDNSScanner?

  private init() {
    // Kick off an automatic background scan on startup.
    Task { await scan() }
  }

  func selectServer(_ info: RockboxServerInfo) {
    currentServer = info
    ServerConfig.shared.set(info)
    NotificationCenter.default.post(name: .rockboxServerDidChange, object: nil)
  }

  func scan() async {
    guard !isScanning else { return }
    isScanning = true
    let s = MDNSScanner()
    scanner = s
    let found = await s.scan(timeout: 5)
    discoveredServers = found
    isScanning = false
    scanner = nil
  }
}

extension Notification.Name {
  static let rockboxServerDidChange = Notification.Name("rockboxServerDidChange")
}

// MARK: - LAN address ranking
//
// Lower score = higher preference.
//   192.168.x.x — home / office LAN                → 0
//   10.x.x.x    — corporate / VPN LAN              → 1
//   172.16-31.x — Docker / VM virtual bridges       → 2
//   anything else (127.x, 169.254.x, public, …)    → 3

private func lanScore(_ ip: String) -> Int {
  let parts = ip.split(separator: ".").compactMap { Int($0) }
  guard parts.count == 4 else { return 3 }
  switch (parts[0], parts[1]) {
  case (192, 168):                              return 0
  case (10, _):                                 return 1
  case (172, let b) where b >= 16 && b <= 31:  return 2
  default:                                      return 3
  }
}

// MARK: - mDNS scanner (NWBrowser + NetService resolution)

final class MDNSScanner {
  private var browser: NWBrowser?
  private var pendingResolvers: [NetServiceResolver] = []
  private var continuation: CheckedContinuation<[RockboxServerInfo], Never>?
  private var timer: DispatchWorkItem?
  private let lock = NSLock()

  // Keyed by device ID — the shared suffix in "grpc-<id>", "http-<id>", etc.
  // All four services (grpc, graphql, http, mpd) from the same physical machine
  // carry the same device ID, so they always merge into one entry here even if
  // individual service records resolve to different IPs (Docker vs LAN).
  private var deviceMap: [String: DeviceInfo] = [:]

  private struct DeviceInfo {
    var hosts: Set<String> = []
    var grpcPort: Int    = 6061
    var graphqlPort: Int = 6062
    var httpPort: Int    = 6063

    // Best host = lowest LAN score among every IP seen for this device.
    var bestHost: String { hosts.min(by: { lanScore($0) < lanScore($1) }) ?? "" }
  }

  func scan(timeout: TimeInterval) async -> [RockboxServerInfo] {
    await withCheckedContinuation { cont in
      self.continuation = cont

      let params = NWParameters()
      params.includePeerToPeer = true
      let b = NWBrowser(for: .bonjour(type: "_rockbox._tcp", domain: "local"), using: params)
      self.browser = b

      b.browseResultsChangedHandler = { [weak self] _, changes in
        guard let self = self else { return }
        for change in changes {
          guard case .added(let result) = change else { continue }
          if case .service(let name, let type, let domain, _) = result.endpoint {
            guard let devId = Self.deviceId(from: name) else { continue }
            let svc = NetService(domain: domain, type: type, name: name)
            let resolver = NetServiceResolver(service: svc) { [weak self] host, port in
              guard let self = self, !host.isEmpty else { return }
              self.lock.withLock {
                var info = self.deviceMap[devId] ?? DeviceInfo()
                info.hosts.insert(host)
                if name.hasPrefix("grpc-")         { info.grpcPort    = port }
                else if name.hasPrefix("graphql-") { info.graphqlPort = port }
                else if name.hasPrefix("http-")    { info.httpPort    = port }
                self.deviceMap[devId] = info
              }
            }
            self.pendingResolvers.append(resolver)
            resolver.start()
          }
        }
      }

      b.stateUpdateHandler = { _ in }
      b.start(queue: .main)

      let item = DispatchWorkItem { [weak self] in self?.finish() }
      self.timer = item
      DispatchQueue.main.asyncAfter(deadline: .now() + timeout, execute: item)
    }
  }

  private func finish() {
    guard let cont = continuation else { return }
    continuation = nil
    browser?.cancel()
    browser = nil
    timer = nil
    for r in pendingResolvers { r.stop() }
    pendingResolvers = []
    let servers: [RockboxServerInfo] = lock.withLock {
      deviceMap.values.compactMap { info -> RockboxServerInfo? in
        let host = info.bestHost
        guard !host.isEmpty else { return nil }
        let entry = RockboxServerInfo(
          name: host, host: host,
          grpcPort: info.grpcPort, graphqlPort: info.graphqlPort, httpPort: info.httpPort
        )
        return entry.isLocalhost ? nil : entry
      }
      .sorted { $0.host < $1.host }
    }
    cont.resume(returning: servers)
  }

  // Extract the device ID from a service name like "grpc-abc123" → "abc123".
  // Returns nil for unrecognised roles so they are silently ignored.
  private static func deviceId(from name: String) -> String? {
    for prefix in ["grpc-", "http-", "graphql-", "mpd-"] {
      if name.hasPrefix(prefix) { return String(name.dropFirst(prefix.count)) }
    }
    return nil
  }
}

// MARK: - NetService resolver helper

final class NetServiceResolver: NSObject, NetServiceDelegate {
  private let service: NetService
  private let completion: (String, Int) -> Void

  init(service: NetService, completion: @escaping (String, Int) -> Void) {
    self.service = service
    self.completion = completion
    super.init()
    service.delegate = self
  }

  func start() {
    service.schedule(in: .main, forMode: .default)
    service.resolve(withTimeout: 4)
  }

  func stop() {
    service.stop()
  }

  func netServiceDidResolveAddress(_ sender: NetService) {
    let host = resolvedHost(from: sender)
    guard !host.isEmpty else { return }
    completion(host, sender.port)
  }

  func netService(_ sender: NetService, didNotResolve errorDict: [String: NSNumber]) {}

  // Pick the best IPv4 address from the sockaddr list for this single service
  // resolution. Because all resolved IPs are also stored in DeviceInfo.hosts,
  // the MDNSScanner re-ranks across all services for the same device anyway —
  // this call just avoids handing an obviously-wrong IP to the callback.
  private func resolvedHost(from sender: NetService) -> String {
    var candidates: [String] = []

    if let addresses = sender.addresses {
      for data in addresses {
        // On Darwin sockaddr_in: sin_len at offset 0, sin_family at offset 1
        let af = data.withUnsafeBytes { ptr -> UInt8 in
          ptr.baseAddress!.advanced(by: 1).assumingMemoryBound(to: UInt8.self).pointee
        }
        guard Int32(af) == AF_INET else { continue }
        let ip = data.withUnsafeBytes { ptr -> String? in
          var sin = ptr.load(as: sockaddr_in.self)
          var buf = [CChar](repeating: 0, count: Int(INET_ADDRSTRLEN))
          guard inet_ntop(AF_INET, &sin.sin_addr, &buf, socklen_t(INET_ADDRSTRLEN)) != nil else { return nil }
          return String(cString: buf)
        }
        if let ip = ip, !ip.isEmpty { candidates.append(ip) }
      }
    }

    if let best = candidates.min(by: { lanScore($0) < lanScore($1) }) { return best }

    // Last resort: strip trailing dot from hostName.
    if let raw = sender.hostName {
      let h = raw.hasSuffix(".") ? String(raw.dropLast()) : raw
      if !h.isEmpty { return h }
    }
    return ""
  }
}
