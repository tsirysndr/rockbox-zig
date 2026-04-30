//
//  TrackService.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 14/12/2025.
//

import Foundation
import GRPCCore
import GRPCNIOTransportHTTP2

func fetchTrack(id: String) async throws -> Rockbox_V1alpha1_Track {
  try await withRockboxGRPCClient { grpcClient in
    let library = Rockbox_V1alpha1_LibraryService.Client(wrapping: grpcClient)
    var req = Rockbox_V1alpha1_GetTrackRequest()
    req.id = id
    return try await library.getTrack(req).track
  }
}

func fetchTracks() async throws -> [Rockbox_V1alpha1_Track] {
  try await withRockboxGRPCClient { grpcClient in
    let library = Rockbox_V1alpha1_LibraryService.Client(wrapping: grpcClient)
    let req = Rockbox_V1alpha1_GetTracksRequest()
    return try await library.getTracks(req).tracks
  }
}

func fetchLikedTracks() async throws -> [Rockbox_V1alpha1_Track] {
  try await withRockboxGRPCClient { grpcClient in
    let library = Rockbox_V1alpha1_LibraryService.Client(wrapping: grpcClient)
    let req = Rockbox_V1alpha1_GetLikedTracksRequest()
    return try await library.getLikedTracks(req).tracks
  }
}

func likeTrack(id: String) async throws {
  try await withRockboxGRPCClient { grpcClient in
    let library = Rockbox_V1alpha1_LibraryService.Client(wrapping: grpcClient)
    var req = Rockbox_V1alpha1_LikeTrackRequest()
    req.id = id
    let _ = try await library.likeTrack(req)
  }
}

func unlikeTrack(id: String) async throws {
  try await withRockboxGRPCClient { grpcClient in
    let library = Rockbox_V1alpha1_LibraryService.Client(wrapping: grpcClient)
    var req = Rockbox_V1alpha1_UnlikeTrackRequest()
    req.id = id
    let _ = try await library.unlikeTrack(req)
  }
}
