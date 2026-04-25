//
//  SavedPlaylistService.swift
//  Rockbox
//

import GRPCCore
import GRPCNIOTransportHTTP2

func fetchSavedPlaylists(
  folderID: String? = nil,
  host: String = "127.0.0.1",
  port: Int = 6061
) async throws -> [Rockbox_V1alpha1_SavedPlaylist] {
  try await withGRPCClient(
    transport: .http2NIOPosix(
      target: .dns(host: host, port: port),
      transportSecurity: .plaintext
    )
  ) { grpcClient in
    let service = Rockbox_V1alpha1_SavedPlaylistService.Client(wrapping: grpcClient)
    var req = Rockbox_V1alpha1_GetSavedPlaylistsRequest()
    if let fid = folderID { req.folderID = fid }
    let res = try await service.getSavedPlaylists(req)
    return res.playlists
  }
}

func fetchSavedPlaylist(id: String, host: String = "127.0.0.1", port: Int = 6061) async throws
  -> Rockbox_V1alpha1_SavedPlaylist?
{
  try await withGRPCClient(
    transport: .http2NIOPosix(
      target: .dns(host: host, port: port),
      transportSecurity: .plaintext
    )
  ) { grpcClient in
    let service = Rockbox_V1alpha1_SavedPlaylistService.Client(wrapping: grpcClient)
    var req = Rockbox_V1alpha1_GetSavedPlaylistRequest()
    req.id = id
    let res = try await service.getSavedPlaylist(req)
    return res.hasPlaylist ? res.playlist : nil
  }
}

func createSavedPlaylist(
  name: String,
  description: String? = nil,
  trackIDs: [String] = [],
  host: String = "127.0.0.1",
  port: Int = 6061
) async throws -> Rockbox_V1alpha1_SavedPlaylist {
  try await withGRPCClient(
    transport: .http2NIOPosix(
      target: .dns(host: host, port: port),
      transportSecurity: .plaintext
    )
  ) { grpcClient in
    let service = Rockbox_V1alpha1_SavedPlaylistService.Client(wrapping: grpcClient)
    var req = Rockbox_V1alpha1_CreateSavedPlaylistRequest()
    req.name = name
    if let desc = description { req.description_p = desc }
    req.trackIDs = trackIDs
    let res = try await service.createSavedPlaylist(req)
    return res.playlist
  }
}

func updateSavedPlaylist(
  id: String,
  name: String,
  description: String? = nil,
  host: String = "127.0.0.1",
  port: Int = 6061
) async throws {
  try await withGRPCClient(
    transport: .http2NIOPosix(
      target: .dns(host: host, port: port),
      transportSecurity: .plaintext
    )
  ) { grpcClient in
    let service = Rockbox_V1alpha1_SavedPlaylistService.Client(wrapping: grpcClient)
    var req = Rockbox_V1alpha1_UpdateSavedPlaylistRequest()
    req.id = id
    req.name = name
    if let desc = description { req.description_p = desc }
    let _ = try await service.updateSavedPlaylist(req)
  }
}

func deleteSavedPlaylist(id: String, host: String = "127.0.0.1", port: Int = 6061) async throws {
  try await withGRPCClient(
    transport: .http2NIOPosix(
      target: .dns(host: host, port: port),
      transportSecurity: .plaintext
    )
  ) { grpcClient in
    let service = Rockbox_V1alpha1_SavedPlaylistService.Client(wrapping: grpcClient)
    var req = Rockbox_V1alpha1_DeleteSavedPlaylistRequest()
    req.id = id
    let _ = try await service.deleteSavedPlaylist(req)
  }
}

func fetchSavedPlaylistTracks(
  playlistID: String,
  host: String = "127.0.0.1",
  port: Int = 6061
) async throws -> [String] {
  try await withGRPCClient(
    transport: .http2NIOPosix(
      target: .dns(host: host, port: port),
      transportSecurity: .plaintext
    )
  ) { grpcClient in
    let service = Rockbox_V1alpha1_SavedPlaylistService.Client(wrapping: grpcClient)
    var req = Rockbox_V1alpha1_GetSavedPlaylistTracksRequest()
    req.playlistID = playlistID
    let res = try await service.getSavedPlaylistTracks(req)
    return res.trackIDs
  }
}

func addTracksToSavedPlaylist(
  playlistID: String,
  trackIDs: [String],
  host: String = "127.0.0.1",
  port: Int = 6061
) async throws {
  try await withGRPCClient(
    transport: .http2NIOPosix(
      target: .dns(host: host, port: port),
      transportSecurity: .plaintext
    )
  ) { grpcClient in
    let service = Rockbox_V1alpha1_SavedPlaylistService.Client(wrapping: grpcClient)
    var req = Rockbox_V1alpha1_AddTracksToSavedPlaylistRequest()
    req.playlistID = playlistID
    req.trackIDs = trackIDs
    let _ = try await service.addTracksToSavedPlaylist(req)
  }
}

func removeTrackFromSavedPlaylist(
  playlistID: String,
  trackID: String,
  host: String = "127.0.0.1",
  port: Int = 6061
) async throws {
  try await withGRPCClient(
    transport: .http2NIOPosix(
      target: .dns(host: host, port: port),
      transportSecurity: .plaintext
    )
  ) { grpcClient in
    let service = Rockbox_V1alpha1_SavedPlaylistService.Client(wrapping: grpcClient)
    var req = Rockbox_V1alpha1_RemoveTrackFromSavedPlaylistRequest()
    req.playlistID = playlistID
    req.trackID = trackID
    let _ = try await service.removeTrackFromSavedPlaylist(req)
  }
}

func playSavedPlaylist(id: String, host: String = "127.0.0.1", port: Int = 6061) async throws {
  try await withGRPCClient(
    transport: .http2NIOPosix(
      target: .dns(host: host, port: port),
      transportSecurity: .plaintext
    )
  ) { grpcClient in
    let service = Rockbox_V1alpha1_SavedPlaylistService.Client(wrapping: grpcClient)
    var req = Rockbox_V1alpha1_PlaySavedPlaylistRequest()
    req.playlistID = id
    let _ = try await service.playSavedPlaylist(req)
  }
}
