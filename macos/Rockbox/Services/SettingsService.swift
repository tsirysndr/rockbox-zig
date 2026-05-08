//
//  SettingsService.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 21/12/2025.
//

import Foundation
import GRPCCore
import GRPCNIOTransportHTTP2

func fetchGlobalSettings() async throws -> Rockbox_V1alpha1_GetGlobalSettingsResponse {
  try await withRockboxGRPCClient { grpcClient in
    let settings = Rockbox_V1alpha1_SettingsService.Client(wrapping: grpcClient)
    let req = Rockbox_V1alpha1_GetGlobalSettingsRequest()
    return try await settings.getGlobalSettings(req)
  }
}

func updatePlaylistShuffle(enabled: Bool) async throws {
  try await withRockboxGRPCClient { grpcClient in
    let settings = Rockbox_V1alpha1_SettingsService.Client(wrapping: grpcClient)
    var req = Rockbox_V1alpha1_SaveSettingsRequest()
    req.playlistShuffle = enabled
    let _ = try await settings.saveSettings(req)
  }
}

func updateRepeatMode(repeatMode: Int32) async throws {
  try await withRockboxGRPCClient { grpcClient in
    let settings = Rockbox_V1alpha1_SettingsService.Client(wrapping: grpcClient)
    var req = Rockbox_V1alpha1_SaveSettingsRequest()
    req.repeatMode = repeatMode
    let _ = try await settings.saveSettings(req)
  }
}

func saveAllSettings(_ req: Rockbox_V1alpha1_SaveSettingsRequest) async throws {
  try await withRockboxGRPCClient { grpcClient in
    let settings = Rockbox_V1alpha1_SettingsService.Client(wrapping: grpcClient)
    let _ = try await settings.saveSettings(req)
  }
}
