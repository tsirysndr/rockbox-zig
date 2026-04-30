//
//  PlaylistService.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 17/12/2025.
//

import GRPCCore
import GRPCNIOTransportHTTP2

func fetchCurrentPlaylist() async throws -> Rockbox_V1alpha1_GetCurrentResponse {
  try await withRockboxGRPCClient { grpcClient in
    let playlist = Rockbox_V1alpha1_PlaylistService.Client(wrapping: grpcClient)
    let req = Rockbox_V1alpha1_GetCurrentRequest()
    return try await playlist.getCurrent(req)
  }
}

func resumeTrack() async throws {
  try await withRockboxGRPCClient { grpcClient in
    let playlist = Rockbox_V1alpha1_PlaylistService.Client(wrapping: grpcClient)
    let req = Rockbox_V1alpha1_ResumeTrackRequest()
    let _ = try await playlist.resumeTrack(req)
  }
}

func currentPlaylistStream() -> AsyncThrowingStream<Rockbox_V1alpha1_PlaylistResponse, Error> {
  AsyncThrowingStream { continuation in
    Task {
      do {
        try await withRockboxGRPCClient { grpcClient in
          let playback = Rockbox_V1alpha1_PlaybackService.Client(wrapping: grpcClient)
          let req = Rockbox_V1alpha1_StreamPlaylistRequest()
          try await playback.streamPlaylist(req) { response in
            for try await message in response.messages {
              continuation.yield(message)
            }
            continuation.finish()
          }
        }
      } catch {
        continuation.finish(throwing: error)
      }
    }
  }
}

func startPlaylist(position: Int32) async throws {
  try await withRockboxGRPCClient { grpcClient in
    let playlist = Rockbox_V1alpha1_PlaylistService.Client(wrapping: grpcClient)
    var req = Rockbox_V1alpha1_StartRequest()
    req.startIndex = position
    let _ = try await playlist.start(req)
  }
}

func removeFromPlaylist(position: Int32) async throws {
  try await withRockboxGRPCClient { grpcClient in
    let playlist = Rockbox_V1alpha1_PlaylistService.Client(wrapping: grpcClient)
    var req = Rockbox_V1alpha1_RemoveTracksRequest()
    req.positions = [position]
    let _ = try await playlist.removeTracks(req)
  }
}

func insertTracks(tracks: [String], position: Int32, shuffle: Bool = false) async throws {
  try await withRockboxGRPCClient { grpcClient in
    let playlist = Rockbox_V1alpha1_PlaylistService.Client(wrapping: grpcClient)
    var req = Rockbox_V1alpha1_InsertTracksRequest()
    req.tracks = tracks
    req.position = position
    req.shuffle = shuffle
    let _ = try await playlist.insertTracks(req)
  }
}

func insertAlbum(albumID: String, position: Int32, shuffle: Bool = false) async throws {
  try await withRockboxGRPCClient { grpcClient in
    let playlist = Rockbox_V1alpha1_PlaylistService.Client(wrapping: grpcClient)
    var req = Rockbox_V1alpha1_InsertAlbumRequest()
    req.albumID = albumID
    req.position = position
    req.shuffle = shuffle
    let _ = try await playlist.insertAlbum(req)
  }
}

func insertDirectory(
  directory: String, position: Int32, recurse: Bool = false, shuffle: Bool = false
) async throws {
  try await withRockboxGRPCClient { grpcClient in
    let playlist = Rockbox_V1alpha1_PlaylistService.Client(wrapping: grpcClient)
    var req = Rockbox_V1alpha1_InsertDirectoryRequest()
    req.directory = directory
    req.position = position
    req.recurse = recurse
    req.shuffle = shuffle
    let _ = try await playlist.insertDirectory(req)
  }
}

func clearPlaylist() async throws {
  try await withRockboxGRPCClient { grpcClient in
    let playlist = Rockbox_V1alpha1_PlaylistService.Client(wrapping: grpcClient)
    let req = Rockbox_V1alpha1_RemoveAllTracksRequest()
    let _ = try await playlist.removeAllTracks(req)
  }
}
