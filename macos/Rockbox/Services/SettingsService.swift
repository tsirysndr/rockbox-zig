//
//  SettingsService.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 21/12/2025.
//

import Foundation
import GRPCCore
import GRPCNIOTransportHTTP2

func fetchGlobalSettings(host: String = "127.0.0.1", port: Int = 6061) async throws
  -> Rockbox_V1alpha1_GetGlobalSettingsResponse
{
  try await withGRPCClient(
    transport: .http2NIOPosix(
      target: .dns(host: host, port: port),
      transportSecurity: .plaintext
    )
  ) { grpcClient in
    let settings = Rockbox_V1alpha1_SettingsService.Client(wrapping: grpcClient)

    let req = Rockbox_V1alpha1_GetGlobalSettingsRequest()
    let res = try await settings.getGlobalSettings(req)

    return res
  }
}

func updatePlaylistShuffle(enabled: Bool, host: String = "127.0.0.1", port: Int = 6061) async throws {
    try await withGRPCClient(
      transport: .http2NIOPosix(
        target: .dns(host: host, port: port),
        transportSecurity: .plaintext
      )
    ) { grpcClient in
      let settings = Rockbox_V1alpha1_SettingsService.Client(wrapping: grpcClient)

      var req = Rockbox_V1alpha1_SaveSettingsRequest()
      req.playlistShuffle = enabled
      let _ = try await settings.saveSettings(req)
    }
}

func updateRepeatMode(repeatMode: Int32, host: String = "127.0.0.1", port: Int = 6061) async throws {
    try await withGRPCClient(
      transport: .http2NIOPosix(
        target: .dns(host: host, port: port),
        transportSecurity: .plaintext
      )
    ) { grpcClient in
      let settings = Rockbox_V1alpha1_SettingsService.Client(wrapping: grpcClient)

      var req = Rockbox_V1alpha1_SaveSettingsRequest()
      req.repeatMode = repeatMode
      let _ = try await settings.saveSettings(req)
    }
}
