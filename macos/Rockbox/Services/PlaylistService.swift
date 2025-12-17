//
//  PlaylistService.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 17/12/2025.
//

import GRPCCore
import GRPCNIOTransportHTTP2

func fetchCurrentPlaylist(host: String = "127.0.0.1", port: Int = 6061) async throws -> Rockbox_V1alpha1_GetCurrentResponse {
  try await withGRPCClient(
    transport: .http2NIOPosix(
      target: .dns(host: host, port: port),
      transportSecurity: .plaintext
    )
  ) { grpcClient in
    let playlist = Rockbox_V1alpha1_PlaylistService.Client(wrapping: grpcClient)

    let req = Rockbox_V1alpha1_GetCurrentRequest()

    let res = try await playlist.getCurrent(req)

    return res
  }
}
