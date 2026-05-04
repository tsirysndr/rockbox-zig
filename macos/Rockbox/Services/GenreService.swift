//
//  GenreService.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 14/12/2025.
//

import Foundation
import GRPCCore
import GRPCNIOTransportHTTP2

func fetchGenres() async throws -> [Rockbox_V1alpha1_Genre] {
  try await withRockboxGRPCClient { grpcClient in
    let genres = Rockbox_V1alpha1_GenreService.Client(wrapping: grpcClient)
    let req = Rockbox_V1alpha1_GetGenresRequest()
    let res = try await genres.getGenres(req)
    return res.genres
  }
}

func fetchGenre(id: String) async throws -> Rockbox_V1alpha1_Genre {
  try await withRockboxGRPCClient { grpcClient in
    let genres = Rockbox_V1alpha1_GenreService.Client(wrapping: grpcClient)
    var req = Rockbox_V1alpha1_GetGenreRequest()
    req.id = id
    let res = try await genres.getGenre(req)
    return res.genre
  }
}

func fetchGenreTracks(genreID: String) async throws -> [Rockbox_V1alpha1_Track] {
  try await withRockboxGRPCClient { grpcClient in
    let genres = Rockbox_V1alpha1_GenreService.Client(wrapping: grpcClient)
    var req = Rockbox_V1alpha1_GetGenreTracksRequest()
    req.id = genreID
    let res = try await genres.getGenreTracks(req)
    return res.tracks
  }
}

func fetchGenreAlbums(genreID: String) async throws -> [Rockbox_V1alpha1_Album] {
  try await withRockboxGRPCClient { grpcClient in
    let genres = Rockbox_V1alpha1_GenreService.Client(wrapping: grpcClient)
    var req = Rockbox_V1alpha1_GetGenreAlbumsRequest()
    req.id = genreID
    let res = try await genres.getGenreAlbums(req)
    return res.albums
  }
}

func fetchGenreArtists(genreID: String) async throws -> [Rockbox_V1alpha1_Artist] {
  try await withRockboxGRPCClient { grpcClient in
    let genres = Rockbox_V1alpha1_GenreService.Client(wrapping: grpcClient)
    var req = Rockbox_V1alpha1_GetGenreArtistsRequest()
    req.id = genreID
    let res = try await genres.getGenreArtists(req)
    return res.artists
  }
}
