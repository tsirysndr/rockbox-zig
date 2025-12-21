//
//  AlbumsGridView.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 14/12/2025.
//

import SwiftUI

struct AlbumsGridView: View {
    @State private var albums: [Album] = []
    @State private var errorText: String?
    @Binding var selectedAlbum: Album?
    
    private let columns = [
        GridItem(.adaptive(minimum: 170, maximum: 230), spacing: 20)
    ]
    
    var body: some View {
        ScrollView {
            LazyVGrid(columns: columns, spacing: 24) {
                ForEach(albums) { album in
                    AlbumCardView(album: album, playlists: []) {
                        selectedAlbum = album
                    }
                }
            }
            .padding(20)
        }
        .task {
            do {
                let data = try await fetchAlbums()
                albums = []
                for album in data {
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
