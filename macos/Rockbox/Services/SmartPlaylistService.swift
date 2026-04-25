//
//  SmartPlaylistService.swift
//  Rockbox
//

import GRPCCore
import GRPCNIOTransportHTTP2

func fetchSmartPlaylists(host: String = "127.0.0.1", port: Int = 6061) async throws
  -> [Rockbox_V1alpha1_SmartPlaylist]
{
  try await withGRPCClient(
    transport: .http2NIOPosix(
      target: .dns(host: host, port: port),
      transportSecurity: .plaintext
    )
  ) { grpcClient in
    let service = Rockbox_V1alpha1_SmartPlaylistService.Client(wrapping: grpcClient)
    let req = Rockbox_V1alpha1_GetSmartPlaylistsRequest()
    let res = try await service.getSmartPlaylists(req)
    return res.playlists
  }
}

func fetchSmartPlaylistTracks(
  id: String,
  host: String = "127.0.0.1",
  port: Int = 6061
) async throws -> [String] {
  try await withGRPCClient(
    transport: .http2NIOPosix(
      target: .dns(host: host, port: port),
      transportSecurity: .plaintext
    )
  ) { grpcClient in
    let service = Rockbox_V1alpha1_SmartPlaylistService.Client(wrapping: grpcClient)
    var req = Rockbox_V1alpha1_GetSmartPlaylistTracksRequest()
    req.id = id
    let res = try await service.getSmartPlaylistTracks(req)
    return res.trackIDs
  }
}

func playSmartPlaylist(id: String, host: String = "127.0.0.1", port: Int = 6061) async throws {
  try await withGRPCClient(
    transport: .http2NIOPosix(
      target: .dns(host: host, port: port),
      transportSecurity: .plaintext
    )
  ) { grpcClient in
    let service = Rockbox_V1alpha1_SmartPlaylistService.Client(wrapping: grpcClient)
    var req = Rockbox_V1alpha1_PlaySmartPlaylistRequest()
    req.id = id
    let _ = try await service.playSmartPlaylist(req)
  }
}

func deleteSmartPlaylist(id: String, host: String = "127.0.0.1", port: Int = 6061) async throws {
  try await withGRPCClient(
    transport: .http2NIOPosix(
      target: .dns(host: host, port: port),
      transportSecurity: .plaintext
    )
  ) { grpcClient in
    let service = Rockbox_V1alpha1_SmartPlaylistService.Client(wrapping: grpcClient)
    var req = Rockbox_V1alpha1_DeleteSmartPlaylistRequest()
    req.id = id
    let _ = try await service.deleteSmartPlaylist(req)
  }
}

func recordTrackPlayed(trackID: String, host: String = "127.0.0.1", port: Int = 6061) async throws {
  try await withGRPCClient(
    transport: .http2NIOPosix(
      target: .dns(host: host, port: port),
      transportSecurity: .plaintext
    )
  ) { grpcClient in
    let service = Rockbox_V1alpha1_SmartPlaylistService.Client(wrapping: grpcClient)
    var req = Rockbox_V1alpha1_RecordTrackPlayedRequest()
    req.trackID = trackID
    let _ = try await service.recordTrackPlayed(req)
  }
}
