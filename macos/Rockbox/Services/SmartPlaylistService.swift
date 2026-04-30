//
//  SmartPlaylistService.swift
//  Rockbox
//

import GRPCCore
import GRPCNIOTransportHTTP2

func fetchSmartPlaylists() async throws -> [Rockbox_V1alpha1_SmartPlaylist] {
  try await withRockboxGRPCClient { grpcClient in
    let service = Rockbox_V1alpha1_SmartPlaylistService.Client(wrapping: grpcClient)
    let req = Rockbox_V1alpha1_GetSmartPlaylistsRequest()
    let res = try await service.getSmartPlaylists(req)
    return res.playlists
  }
}

func fetchSmartPlaylistTracks(id: String) async throws -> [String] {
  try await withRockboxGRPCClient { grpcClient in
    let service = Rockbox_V1alpha1_SmartPlaylistService.Client(wrapping: grpcClient)
    var req = Rockbox_V1alpha1_GetSmartPlaylistTracksRequest()
    req.id = id
    let res = try await service.getSmartPlaylistTracks(req)
    return res.trackIDs
  }
}

func playSmartPlaylist(id: String) async throws {
  try await withRockboxGRPCClient { grpcClient in
    let service = Rockbox_V1alpha1_SmartPlaylistService.Client(wrapping: grpcClient)
    var req = Rockbox_V1alpha1_PlaySmartPlaylistRequest()
    req.id = id
    let _ = try await service.playSmartPlaylist(req)
  }
}

func deleteSmartPlaylist(id: String) async throws {
  try await withRockboxGRPCClient { grpcClient in
    let service = Rockbox_V1alpha1_SmartPlaylistService.Client(wrapping: grpcClient)
    var req = Rockbox_V1alpha1_DeleteSmartPlaylistRequest()
    req.id = id
    let _ = try await service.deleteSmartPlaylist(req)
  }
}

func recordTrackPlayed(trackID: String) async throws {
  try await withRockboxGRPCClient { grpcClient in
    let service = Rockbox_V1alpha1_SmartPlaylistService.Client(wrapping: grpcClient)
    var req = Rockbox_V1alpha1_RecordTrackPlayedRequest()
    req.trackID = trackID
    let _ = try await service.recordTrackPlayed(req)
  }
}
