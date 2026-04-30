//
//  AlbumService.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 14/12/2025.
//

import Foundation
import GRPCCore
import GRPCNIOTransportHTTP2

func fetchAlbums() async throws -> [Rockbox_V1alpha1_Album] {
  try await withRockboxGRPCClient { grpcClient in
    let library = Rockbox_V1alpha1_LibraryService.Client(wrapping: grpcClient)
    let req = Rockbox_V1alpha1_GetAlbumsRequest()
    let res = try await library.getAlbums(req)
    return res.albums
  }
}

func fetchAlbum(id: String) async throws -> Rockbox_V1alpha1_Album {
  try await withRockboxGRPCClient { grpcClient in
    let library = Rockbox_V1alpha1_LibraryService.Client(wrapping: grpcClient)
    var req = Rockbox_V1alpha1_GetAlbumRequest()
    req.id = id
    let res = try await library.getAlbum(req)
    return res.album
  }
}

func fetchAlbumTracks(albumID: String) async throws -> [Rockbox_V1alpha1_Track] {
  let album = try await fetchAlbum(id: albumID)
  return album.tracks
}

func likeAlbum(id: String) async throws {
  try await withRockboxGRPCClient { grpcClient in
    let library = Rockbox_V1alpha1_LibraryService.Client(wrapping: grpcClient)
    var req = Rockbox_V1alpha1_LikeAlbumRequest()
    req.id = id
    let _ = try await library.likeAlbum(req)
  }
}

func unlikeAlbum(id: String) async throws {
  try await withRockboxGRPCClient { grpcClient in
    let library = Rockbox_V1alpha1_LibraryService.Client(wrapping: grpcClient)
    var req = Rockbox_V1alpha1_UnlikeAlbumRequest()
    req.id = id
    let _ = try await library.unlikeAlbum(req)
  }
}
