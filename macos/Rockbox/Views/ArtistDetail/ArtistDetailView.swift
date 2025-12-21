//
//  ArtistDetailsView.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 14/12/2025.
//

import SwiftUI

// MARK: - Artist Detail View

struct ArtistDetailView: View {
    let artist: Artist
    @State private var tracks: [Song] = []
    @State private var albums: [Album] = []
    @State private var errorText: String?
    @ObservedObject var library: MusicLibrary
    var onBack: () -> Void
    var onAlbumSelected: (Album) -> Void
    
    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 0) {
                // Artist header
                ArtistHeaderView(
                    artist: artist,
                    trackCount: tracks.count,
                    albumCount: albums.count,
                    onBack: onBack
                )
                
                Divider()
                    .padding(.top, 20)
                
                // Tracks section
                if !tracks.isEmpty {
                    SectionHeaderView(title: "Songs", count: tracks.count)
                    
                    LazyVStack(spacing: 0) {
                        ForEach(Array(tracks.enumerated()), id: \.element.id) { index, track in
                            ArtistTrackRowView(
                                track: track,
                                artist: artist,
                                index: index + 1,
                                isEven: index % 2 == 0,
                                library: library
                            )
                        }
                    }
                }
                
                // Albums section
                if !albums.isEmpty {
                    SectionHeaderView(title: "Albums", count: albums.count)
                        .padding(.top, 24)
                    
                    ArtistAlbumsGridView(albums: albums, onAlbumSelected: onAlbumSelected)
                }
            }
        }
        .task {
            do {
                let data = try await fetchArtist(id: artist.cuid)
                for album in data.albums {
                    albums.append(Album(
                        cuid: album.id,
                        title: album.title,
                        artist: album.artist,
                        year: Int(album.year),
                        color: .gray.opacity(0.3),
                        cover: "http://localhost:6062/covers/" + album.albumArt,
                        releaseDate: album.yearString,
                        copyrightMessage: album.copyrightMessage,
                        tracks: []
                    ))
                }
                
                for track in data.tracks {
                    tracks.append(Song(cuid: track.id, path: track.path, title: track.title, artist: track.artist, album: track.album, albumArt: URL(string: "http://localhost:6062/covers/" + track.albumArt), duration: TimeInterval(track.length / 1000),trackNumber: Int(track.trackNumber), discNumber: Int(track.discNumber), color: .gray.opacity(0.3)))
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
}

struct SectionHeaderView: View {
    let title: String
    let count: Int
    
    var body: some View {
        HStack {
            Text(title)
                .font(.system(size: 18, weight: .bold))
            
            Text("\(count)")
                .font(.system(size: 14))
                .foregroundStyle(.secondary)
            
            Spacer()
        }
        .padding(.horizontal, 24)
        .padding(.vertical, 12)
    }
}

