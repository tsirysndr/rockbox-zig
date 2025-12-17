//
//  SystemService.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 17/12/2025.
//

import Foundation
import GRPCCore
import GRPCNIOTransportHTTP2

func fetchGlobalStatus(host: String = "127.0.0.1", port: Int = 6061) async throws -> Rockbox_V1alpha1_GetGlobalStatusResponse {
  try await withGRPCClient(
    transport: .http2NIOPosix(
      target: .dns(host: host, port: port),
      transportSecurity: .plaintext
    )
  ) { grpcClient in
    let system = Rockbox_V1alpha1_SystemService.Client(wrapping: grpcClient)

    let req = Rockbox_V1alpha1_GetGlobalStatusRequest()
    let res = try await system.getGlobalStatus(req)

    return res
  }
}
