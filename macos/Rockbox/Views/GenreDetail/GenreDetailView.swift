//
//  GenreDetailView.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 04/05/2026.
//

import SwiftUI

struct GenreDetailView: View {
    let genre: Genre
    @State private var tracks: [Song] = []
    @State private var albums: [Album] = []
    @State private var artists: [Artist] = []
    @State private var errorText: String?
    @ObservedObject var library: MusicLibrary
    var onBack: () -> Void
    var onAlbumSelected: (Album) -> Void
    var onArtistSelected: (Artist) -> Void

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 0) {
                GenreHeaderView(
                    genre: genre,
                    trackCount: tracks.count,
                    albumCount: albums.count,
                    artistCount: artists.count,
                    onBack: onBack
                )

                Divider().padding(.top, 20)

                if !tracks.isEmpty {
                    SectionHeaderView(title: "Songs", count: tracks.count)

                    LazyVStack(spacing: 0) {
                        ForEach(Array(tracks.prefix(20).enumerated()), id: \.element.id) {
                            index, track in
                            ArtistTrackRowView(
                                track: track,
                                artist: Artist(
                                    cuid: track.artistID,
                                    name: track.artist,
                                    image: nil,
                                    genre: genre.name,
                                    color: .gray.opacity(0.3)
                                ),
                                index: index + 1,
                                isEven: index % 2 == 0,
                                library: library
                            )
                        }
                    }
                }

                if !albums.isEmpty {
                    SectionHeaderView(title: "Albums", count: albums.count)
                        .padding(.top, 24)
                    ArtistAlbumsGridView(albums: albums, onAlbumSelected: onAlbumSelected)
                }

                if !artists.isEmpty {
                    SectionHeaderView(title: "Artists", count: artists.count)
                        .padding(.top, 24)

                    let columns = [
                        GridItem(.adaptive(minimum: 130, maximum: 160), spacing: 14)
                    ]
                    LazyVGrid(columns: columns, spacing: 16) {
                        ForEach(artists) { artist in
                            GenreArtistCell(artist: artist, onTap: { onArtistSelected(artist) })
                        }
                    }
                    .padding(.horizontal, 24)
                    .padding(.bottom, 30)
                }
            }
        }
        .task {
            do {
                async let tData = fetchGenreTracks(genreID: genre.cuid)
                async let aData = fetchGenreAlbums(genreID: genre.cuid)
                async let arData = fetchGenreArtists(genreID: genre.cuid)
                let (trackData, albumData, artistData) = try await (tData, aData, arData)

                tracks = trackData.map {
                    Song(
                        cuid: $0.id,
                        path: $0.path,
                        title: $0.title,
                        artist: $0.artist,
                        album: $0.album,
                        albumArt: URL(string: ServerConfig.shared.coversBaseURL + $0.albumArt),
                        duration: TimeInterval($0.length / 1000),
                        trackNumber: Int($0.trackNumber),
                        discNumber: Int($0.discNumber),
                        albumID: $0.albumID,
                        artistID: $0.artistID,
                        color: .gray.opacity(0.3)
                    )
                }
                albums = albumData.map {
                    Album(
                        cuid: $0.id,
                        title: $0.title,
                        artist: $0.artist,
                        year: Int($0.year),
                        color: .gray.opacity(0.3),
                        cover: ServerConfig.shared.coversBaseURL + $0.albumArt,
                        releaseDate: $0.yearString,
                        copyrightMessage: $0.copyrightMessage,
                        artistID: $0.artistID,
                        tracks: []
                    )
                }
                artists = artistData.map {
                    Artist(
                        cuid: $0.id,
                        name: $0.name,
                        image: $0.hasImage ? $0.image : nil,
                        genre: $0.hasGenres ? $0.genres : "",
                        color: .gray.opacity(0.3)
                    )
                }
            } catch {
                errorText = String(describing: error)
            }
        }
        .alert("gRPC Error", isPresented: .constant(errorText != nil)) {
            Button("OK") { errorText = nil }
        } message: {
            Text(errorText ?? "")
        }
    }

    private var placeholderArtistImage: some View {
        ZStack {
            Circle().fill(Color.gray.opacity(0.25))
            Image(systemName: "music.mic")
                .font(.system(size: 28))
                .foregroundStyle(.secondary)
        }
    }
}

private struct GenreArtistCell: View {
    let artist: Artist
    let onTap: () -> Void

    var body: some View {
        Button(action: onTap) {
            VStack(spacing: 8) {
                avatar
                    .frame(width: 100, height: 100)
                    .clipShape(Circle())
                Text(artist.name)
                    .font(.system(size: 13, weight: .medium))
                    .lineLimit(1)
                    .frame(width: 110)
            }
        }
        .buttonStyle(.plain)
    }

    @ViewBuilder
    private var avatar: some View {
        if let imageURL = artist.image, let url = URL(string: imageURL) {
            CachedAsyncImage(url: url) { phase in
                switch phase {
                case .success(let image):
                    image.resizable().scaledToFill()
                default:
                    placeholder
                }
            }
        } else {
            placeholder
        }
    }

    private var placeholder: some View {
        ZStack {
            Circle().fill(Color.gray.opacity(0.25))
            Image(systemName: "music.mic")
                .font(.system(size: 28))
                .foregroundStyle(.secondary)
        }
    }
}

struct GenreHeaderView: View {
    let genre: Genre
    let trackCount: Int
    let albumCount: Int
    let artistCount: Int
    let onBack: () -> Void

    var body: some View {
        ZStack(alignment: .bottomLeading) {
            LinearGradient(
                colors: [genre.color, genre.color.opacity(0.5), Color.black.opacity(0.4)],
                startPoint: .topLeading,
                endPoint: .bottomTrailing
            )
            .frame(height: 220)

            Text(genre.name)
                .font(.system(size: 110, weight: .heavy))
                .foregroundStyle(Color.white.opacity(0.16))
                .rotationEffect(.degrees(-10))
                .offset(x: 200, y: 20)
                .frame(height: 220, alignment: .bottomTrailing)
                .clipped()

            VStack(alignment: .leading, spacing: 6) {
                Text("GENRE")
                    .font(.system(size: 11, weight: .bold))
                    .tracking(2)
                    .foregroundStyle(Color.white.opacity(0.85))
                Text(genre.name)
                    .font(.system(size: 38, weight: .heavy))
                    .foregroundStyle(.white)
                Text(
                    "\(trackCount) tracks · \(albumCount) albums · \(artistCount) artists"
                )
                .font(.system(size: 12))
                .foregroundStyle(Color.white.opacity(0.85))
            }
            .padding(.horizontal, 24)
            .padding(.bottom, 22)

            Button(action: onBack) {
                Image(systemName: "chevron.left")
                    .foregroundStyle(.white)
                    .padding(8)
                    .background(Color.black.opacity(0.4))
                    .clipShape(Circle())
            }
            .buttonStyle(.plain)
            .padding(.leading, 14)
            .padding(.top, 14)
            .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        }
        .frame(height: 220)
        .clipped()
    }
}
