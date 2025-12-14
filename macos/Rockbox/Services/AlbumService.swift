//
//  AlbumService.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 14/12/2025.
//

import Foundation
import GRPCCore
import GRPCNIOTransportHTTP2

func fetchAlbums(host: String = "127.0.0.1", port: Int = 6061) async throws -> [Rockbox_V1alpha1_Album] {
  try await withGRPCClient(
    transport: .http2NIOPosix(
      target: .dns(host: host, port: port),
      transportSecurity: .plaintext
    )
  ) { grpcClient in
    let library = Rockbox_V1alpha1_LibraryService.Client(wrapping: grpcClient)

    let req = Rockbox_V1alpha1_GetAlbumsRequest()

    let res = try await library.getAlbums(req)

    return res.albums
  }
}

func fetchAlbum(id: String, host: String = "127.0.0.1", port: Int = 6061) async throws -> Rockbox_V1alpha1_Album {
  try await withGRPCClient(
    transport: .http2NIOPosix(
      target: .dns(host: host, port: port),
      transportSecurity: .plaintext
    )
  ) { grpcClient in
    let library = Rockbox_V1alpha1_LibraryService.Client(wrapping: grpcClient)

    var req = Rockbox_V1alpha1_GetAlbumRequest()
      req.id = id

    let res = try await library.getAlbum(req)

    return res.album
  }
}


