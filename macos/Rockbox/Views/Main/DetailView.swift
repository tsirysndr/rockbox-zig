//
//  DetailView.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 14/12/2025.
//

import SwiftUI

struct DetailView: View {
    let selection: SidebarItem?
    @ObservedObject var player: PlayerState
    @ObservedObject var library: MusicLibrary
    @State private var selectedAlbum: Album? = nil
    @State private var selectedArtist: Artist? = nil
    @Binding var showQueue: Bool

    
    var body: some View {
        VStack(spacing: 0) {
            // Main content area
            Group {
                if let album = selectedAlbum {
                    AlbumDetailView(album: album, library: library, onBack: {
                        selectedAlbum = nil
                    })
                } else if let artist = selectedArtist {
                    ArtistDetailView(
                        artist: artist,
                        library: library,
                        onBack: { selectedArtist = nil },
                        onAlbumSelected: { album in selectedAlbum = album }
                    )
                } else if let selection {
                    switch selection {
                    case .albums:
                        AlbumsGridView(selectedAlbum: $selectedAlbum)
                    case .artists:
                        ArtistsGridView(selectedArtist: $selectedArtist)
                    case .songs:
                        SongsListView(library: library)
                    case .likes:
                        LikesListView(library: library)
                    case .files:
                        FilesListView()
                    }
                } else {
                    Text("Select an item")
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity, maxHeight: .infinity)
                }
            }
            .background(.white)
            
            Divider()
            
            // Player controls
            PlayerControlsView(library: library, showQueue: $showQueue)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .onChange(of: selection) {
            selectedAlbum = nil
            selectedArtist = nil
        }
        .task {
            do {
                let likes = try await fetchLikedTracks()
                for track in likes {
                    let song = Song(cuid: track.id, title: track.title, artist: track.artist, album: track.album, albumArt: URL(string: "http://localhost:6062/covers/" + track.albumArt), duration: TimeInterval(track.length / 1000), trackNumber: Int(track.trackNumber), discNumber: Int(track.discNumber), color: .gray.opacity(0.3))
                    library.likedSongIds.insert(song.cuid)
                }
            } catch {
                // do nothing on error
            }
        }
    }
}

