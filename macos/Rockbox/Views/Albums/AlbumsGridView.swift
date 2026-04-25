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
    
    @State private var savedPlaylists: [SavedPlaylist] = []

    var body: some View {
        ScrollView {
            LazyVGrid(columns: columns, spacing: 24) {
                ForEach(albums) { album in
                    AlbumCardView(album: album, savedPlaylists: savedPlaylists) {
                        selectedAlbum = album
                    }
                }
            }
            .padding(20)
        }
        .task {
            do {
                async let albumData = fetchAlbums()
                async let playlistData = fetchSavedPlaylists()
                let (data, plData) = try await (albumData, playlistData)
                albums = data.map {
                    Album(
                        cuid: $0.id,
                        title: $0.title,
                        artist: $0.artist,
                        year: Int($0.year),
                        color: .gray.opacity(0.3),
                        cover: "http://localhost:6062/covers/" + $0.albumArt,
                        releaseDate: $0.yearString,
                        copyrightMessage: $0.copyrightMessage,
                        artistID: $0.artistID,
                        tracks: []
                    )
                }
                savedPlaylists = plData.map {
                    SavedPlaylist(
                        id: $0.id, name: $0.name,
                        description: $0.hasDescription_p ? $0.description_p : nil,
                        image: $0.hasImage ? $0.image : nil,
                        folderID: $0.hasFolderID ? $0.folderID : nil,
                        trackCount: $0.trackCount
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
}
