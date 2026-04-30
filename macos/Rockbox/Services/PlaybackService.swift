//
//  PlaybackService.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 14/12/2025.
//

import Foundation
import GRPCCore
import GRPCNIOTransportHTTP2

func play(elapsed: Int64) async throws {
  try await withRockboxGRPCClient { grpcClient in
    let playback = Rockbox_V1alpha1_PlaybackService.Client(wrapping: grpcClient)
    var req = Rockbox_V1alpha1_PlayRequest()
    req.elapsed = elapsed
    req.offset = 0
    let _ = try await playback.play(req)
  }
}

func resume() async throws {
  try await withRockboxGRPCClient { grpcClient in
    let playback = Rockbox_V1alpha1_PlaybackService.Client(wrapping: grpcClient)
    let req = Rockbox_V1alpha1_ResumeRequest()
    let _ = try await playback.resume(req)
  }
}

func pause() async throws {
  try await withRockboxGRPCClient { grpcClient in
    let playback = Rockbox_V1alpha1_PlaybackService.Client(wrapping: grpcClient)
    let req = Rockbox_V1alpha1_PauseRequest()
    let _ = try await playback.pause(req)
  }
}

func previous() async throws {
  try await withRockboxGRPCClient { grpcClient in
    let playback = Rockbox_V1alpha1_PlaybackService.Client(wrapping: grpcClient)
    let req = Rockbox_V1alpha1_PreviousRequest()
    let _ = try await playback.previous(req)
  }
}

func next() async throws {
  try await withRockboxGRPCClient { grpcClient in
    let playback = Rockbox_V1alpha1_PlaybackService.Client(wrapping: grpcClient)
    let req = Rockbox_V1alpha1_NextRequest()
    let _ = try await playback.next(req)
  }
}

func fetchPlaybackStatus() async throws -> Int32 {
  try await withRockboxGRPCClient { grpcClient in
    let playback = Rockbox_V1alpha1_PlaybackService.Client(wrapping: grpcClient)
    let req = Rockbox_V1alpha1_StatusRequest()
    let response = try await playback.status(req)
    return response.status
  }
}

func currentTrackStream() -> AsyncThrowingStream<Rockbox_V1alpha1_CurrentTrackResponse, Error> {
  AsyncThrowingStream { continuation in
    Task {
      do {
        try await withRockboxGRPCClient { grpcClient in
          let playback = Rockbox_V1alpha1_PlaybackService.Client(wrapping: grpcClient)
          let req = Rockbox_V1alpha1_StreamCurrentTrackRequest()
          try await playback.streamCurrentTrack(req) { response in
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

func playbackStatusStream() -> AsyncThrowingStream<Rockbox_V1alpha1_StatusResponse, Error> {
  AsyncThrowingStream { continuation in
    Task {
      do {
        try await withRockboxGRPCClient { grpcClient in
          let playback = Rockbox_V1alpha1_PlaybackService.Client(wrapping: grpcClient)
          let req = Rockbox_V1alpha1_StreamStatusRequest()
          try await playback.streamStatus(req) { response in
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

func playAlbum(albumID: String, shuffle: Bool = false, position: Int32 = 0) async throws {
  try await withRockboxGRPCClient { grpcClient in
    let playback = Rockbox_V1alpha1_PlaybackService.Client(wrapping: grpcClient)
    var req = Rockbox_V1alpha1_PlayAlbumRequest()
    req.albumID = albumID
    req.shuffle = shuffle
    req.position = position
    let _ = try await playback.playAlbum(req)
  }
}

func playDirectory(path: String, shuffle: Bool = false, position: Int32 = 0) async throws {
  try await withRockboxGRPCClient { grpcClient in
    if path.isEmpty {
      let playback = Rockbox_V1alpha1_PlaybackService.Client(wrapping: grpcClient)
      var req = Rockbox_V1alpha1_PlayMusicDirectoryRequest()
      req.position = position
      req.shuffle = shuffle
      let _ = try await playback.playMusicDirectory(req)
      return
    }
    let playback = Rockbox_V1alpha1_PlaybackService.Client(wrapping: grpcClient)
    var req = Rockbox_V1alpha1_PlayDirectoryRequest()
    req.path = path
    req.shuffle = shuffle
    req.position = position
    let _ = try await playback.playDirectory(req)
  }
}

func playTrack(path: String) async throws {
  try await withRockboxGRPCClient { grpcClient in
    let playback = Rockbox_V1alpha1_PlaybackService.Client(wrapping: grpcClient)
    var req = Rockbox_V1alpha1_PlayTrackRequest()
    req.path = path
    let _ = try await playback.playTrack(req)
  }
}

func playAllTracks(shuffle: Bool = false, position: Int32 = 0) async throws {
  try await withRockboxGRPCClient { grpcClient in
    let playback = Rockbox_V1alpha1_PlaybackService.Client(wrapping: grpcClient)
    var req = Rockbox_V1alpha1_PlayAllTracksRequest()
    req.shuffle = shuffle
    req.position = position
    let _ = try await playback.playAllTracks(req)
  }
}

func playLikedTracks(shuffle: Bool = false, position: Int32 = 0) async throws {
  try await withRockboxGRPCClient { grpcClient in
    let playback = Rockbox_V1alpha1_PlaybackService.Client(wrapping: grpcClient)
    var req = Rockbox_V1alpha1_PlayLikedTracksRequest()
    req.shuffle = shuffle
    req.position = position
    let _ = try await playback.playLikedTracks(req)
  }
}

func playArtistTracks(artistID: String, shuffle: Bool = false, position: Int32 = 0) async throws {
  try await withRockboxGRPCClient { grpcClient in
    let playback = Rockbox_V1alpha1_PlaybackService.Client(wrapping: grpcClient)
    var req = Rockbox_V1alpha1_PlayArtistTracksRequest()
    req.shuffle = shuffle
    req.artistID = artistID
    req.position = position
    let _ = try await playback.playArtistTracks(req)
  }
}
