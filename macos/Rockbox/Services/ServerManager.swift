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

// MARK: - mDNS scanner (NWBrowser + NetService resolution)

final class MDNSScanner {
  private var browser: NWBrowser?
  private var pendingResolvers: [NetServiceResolver] = []
  private var continuation: CheckedContinuation<[RockboxServerInfo], Never>?
  private var timer: DispatchWorkItem?

  // Accumulated port info per host (we see grpc/graphql/http services separately)
  private var serversByHost: [String: RockboxServerInfo] = [:]
  private let lock = NSLock()

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
            let svc = NetService(domain: domain, type: type, name: name)
            let resolver = NetServiceResolver(service: svc) { [weak self] host, port in
              guard let self = self, !host.isEmpty else { return }
              self.lock.withLock {
                var entry = self.serversByHost[host] ?? RockboxServerInfo(
                  name: host, host: host, grpcPort: 6061, graphqlPort: 6062, httpPort: 6063
                )
                if name.hasPrefix("grpc-") {
                  entry = RockboxServerInfo(
                    name: entry.name, host: host,
                    grpcPort: port, graphqlPort: entry.graphqlPort, httpPort: entry.httpPort
                  )
                } else if name.hasPrefix("graphql-") {
                  entry = RockboxServerInfo(
                    name: entry.name, host: host,
                    grpcPort: entry.grpcPort, graphqlPort: port, httpPort: entry.httpPort
                  )
                } else if name.hasPrefix("http-") {
                  entry = RockboxServerInfo(
                    name: entry.name, host: host,
                    grpcPort: entry.grpcPort, graphqlPort: entry.graphqlPort, httpPort: port
                  )
                }
                self.serversByHost[host] = entry
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
    let servers = lock.withLock {
      serversByHost.values
        .filter { !$0.isLocalhost }
        .sorted { $0.host < $1.host }
    }
    cont.resume(returning: servers)
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

  private func resolvedHost(from sender: NetService) -> String {
    // Always prefer a raw IPv4 address from the sockaddr list.
    // `hostName` is a .local mDNS hostname that may resolve to an IPv6 link-local
    // address, causing gRPC connections to fail or reach the wrong interface.
    if let addresses = sender.addresses {
      for data in addresses {
        // On Darwin sockaddr_in: sin_len at offset 0, sin_family at offset 1
        let af = data.withUnsafeBytes { ptr -> UInt8 in
          ptr.baseAddress!.advanced(by: 1).assumingMemoryBound(to: UInt8.self).pointee
        }
        if Int32(af) == AF_INET {
          let ip = data.withUnsafeBytes { ptr -> String? in
            var sin = ptr.load(as: sockaddr_in.self)
            var buf = [CChar](repeating: 0, count: Int(INET_ADDRSTRLEN))
            guard inet_ntop(AF_INET, &sin.sin_addr, &buf, socklen_t(INET_ADDRSTRLEN)) != nil else { return nil }
            return String(cString: buf)
          }
          if let ip = ip, !ip.isEmpty { return ip }
        }
      }
    }
    // Last resort: strip trailing dot from hostName (may still be a hostname, not an IP).
    if let raw = sender.hostName {
      let h = raw.hasSuffix(".") ? String(raw.dropLast()) : raw
      if !h.isEmpty { return h }
    }
    return ""
  }
}
