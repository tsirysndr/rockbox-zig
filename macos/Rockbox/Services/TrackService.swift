//
//  TrackService.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 14/12/2025.
//


import Foundation
import GRPCCore
import GRPCNIOTransportHTTP2

func fetchTracks(host: String = "127.0.0.1", port: Int = 6061) async throws -> [Rockbox_V1alpha1_Track] {
  try await withGRPCClient(
    transport: .http2NIOPosix(
      target: .dns(host: host, port: port),
      transportSecurity: .plaintext
    )
  ) { grpcClient in
    let library = Rockbox_V1alpha1_LibraryService.Client(wrapping: grpcClient)

    let req = Rockbox_V1alpha1_GetTracksRequest()

    let res = try await library.getTracks(req)

    return res.tracks
  }
}

func fetchLikedTracks(host: String = "127.0.0.1", port: Int = 6061) async throws -> [Rockbox_V1alpha1_Track] {
  try await withGRPCClient(
    transport: .http2NIOPosix(
      target: .dns(host: host, port: port),
      transportSecurity: .plaintext
    )
  ) { grpcClient in
    let library = Rockbox_V1alpha1_LibraryService.Client(wrapping: grpcClient)

    let req = Rockbox_V1alpha1_GetLikedTracksRequest()

    let res = try await library.getLikedTracks(req)

    return res.tracks
  }
}

func likeTrack(id: String, host: String = "127.0.0.1", port: Int = 6061) async throws -> Void {
  try await withGRPCClient(
    transport: .http2NIOPosix(
      target: .dns(host: host, port: port),
      transportSecurity: .plaintext
    )
  ) { grpcClient in
    let library = Rockbox_V1alpha1_LibraryService.Client(wrapping: grpcClient)

    var req = Rockbox_V1alpha1_LikeTrackRequest()
    req.id = id
    let _ = try await library.likeTrack(req)
  }
}


func unlikeTrack(id: String, host: String = "127.0.0.1", port: Int = 6061) async throws -> Void {
  try await withGRPCClient(
    transport: .http2NIOPosix(
      target: .dns(host: host, port: port),
      transportSecurity: .plaintext
    )
  ) { grpcClient in
    let library = Rockbox_V1alpha1_LibraryService.Client(wrapping: grpcClient)

    var req = Rockbox_V1alpha1_UnlikeTrackRequest()
    req.id = id
    let _ = try await library.unlikeTrack(req)
  }
}


