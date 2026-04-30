//
//  ServerInfo.swift
//  Rockbox
//

import Foundation
import GRPCCore
import GRPCNIOTransportHTTP2

// MARK: - Server model

struct RockboxServerInfo: Identifiable, Equatable, Hashable {
  var id: String { host }
  let name: String
  let host: String
  let grpcPort: Int
  let graphqlPort: Int
  let httpPort: Int

  static let localhost = RockboxServerInfo(
    name: "localhost", host: "127.0.0.1",
    grpcPort: 6061, graphqlPort: 6062, httpPort: 6063
  )

  var coversBaseURL: String { "http://\(host):\(graphqlPort)/covers/" }
  var httpBaseURL: String { "http://\(host):\(httpPort)" }

  var displayName: String {
    if host == "127.0.0.1" || host == "localhost" { return "localhost" }
    if !name.isEmpty && name != host { return "\(name) (\(host))" }
    return host
  }

  var isLocalhost: Bool { host == "127.0.0.1" || host == "localhost" }
}

// MARK: - Thread-safe config

final class ServerConfig {
  static let shared = ServerConfig()
  private let lock = NSLock()
  private var _info: RockboxServerInfo = .localhost

  private init() {}

  var grpcHost: String { lock.withLock { _info.host } }
  var grpcPort: Int { lock.withLock { _info.grpcPort } }
  var coversBaseURL: String { lock.withLock { _info.coversBaseURL } }
  var httpBaseURL: String { lock.withLock { _info.httpBaseURL } }

  func set(_ info: RockboxServerInfo) { lock.withLock { _info = info } }
  func current() -> RockboxServerInfo { lock.withLock { _info } }
}

// MARK: - gRPC convenience wrapper
//
// The generated stubs are generic over Transport, so the body closure must receive
// the concrete GRPCClient<HTTP2ClientTransport.Posix> rather than an existential.

func withRockboxGRPCClient<R: Sendable>(
  _ body: (GRPCClient<HTTP2ClientTransport.Posix>) async throws -> R
) async throws -> R {
  let host = ServerConfig.shared.grpcHost
  let port = ServerConfig.shared.grpcPort
  return try await withGRPCClient(
    transport: try HTTP2ClientTransport.Posix.http2NIOPosix(
      target: .dns(host: host, port: port),
      transportSecurity: .plaintext
    )
  ) { client in try await body(client) }
}
