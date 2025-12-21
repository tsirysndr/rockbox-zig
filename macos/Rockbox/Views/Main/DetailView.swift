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
    @EnvironmentObject var navigation: NavigationManager
    @Binding var showQueue: Bool
    
    var body: some View {
        VStack(spacing: 0) {
            // Main content area
            contentView
                .background(.white)
            
            Divider()
            
            // Player controls
            PlayerControlsView(library: library, showQueue: $showQueue)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .onChange(of: selection) {
            navigation.reset()
        }
        .task {
            do {
                let likes = try await fetchLikedTracks()
                for track in likes {
                    let song = Song(
                        cuid: track.id,
                        path: track.path,
                        title: track.title,
                        artist: track.artist,
                        album: track.album,
                        albumArt: URL(string: "http://localhost:6062/covers/" + track.albumArt),
                        duration: TimeInterval(track.length / 1000),
                        trackNumber: Int(track.trackNumber),
                        discNumber: Int(track.discNumber),
                        albumID: track.albumID,
                        artistID: track.artistID,
                        color: .gray.opacity(0.3))
                    library.likedSongIds.insert(song.cuid)
                }
            } catch {
                // do nothing on error
            }
        }
    }
    
    @ViewBuilder
    private var contentView: some View {
        if let album = navigation.selectedAlbum {
            AlbumDetailView(album: album, library: library, onBack: {
                navigation.selectedAlbum = nil
            })
        } else if let artist = navigation.selectedArtist {
            ArtistDetailView(
                artist: artist,
                library: library,
                onBack: { navigation.selectedArtist = nil },
                onAlbumSelected: { album in navigation.goToAlbum(album) }
            )
        } else if let selection {
            selectionView(for: selection)
        } else {
            Text("Select an item")
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, maxHeight: .infinity)
        }
    }
    
    @ViewBuilder
    private func selectionView(for selection: SidebarItem) -> some View {
        switch selection {
        case .albums:
            AlbumsGridView(selectedAlbum: $navigation.selectedAlbum)
        case .artists:
            ArtistsGridView(selectedArtist: $navigation.selectedArtist)
        case .songs:
            SongsListView(library: library)
        case .likes:
            LikesListView(library: library)
        case .files:
            FilesListView()
        }
    }
}
