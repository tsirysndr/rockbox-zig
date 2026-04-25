//
//  AlbumTrackView.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 14/12/2025.
//

import SwiftUI

struct AlbumTrackRowView: View {
    let track: Song
    let index: Int
    let isEven: Bool
    let albumID: String
    @State private var errorText: String?
    @State private var savedPlaylists: [SavedPlaylist] = []
    @State private var isHoveringMenu: Bool = false
    @ObservedObject var library: MusicLibrary
    @EnvironmentObject var player: PlayerState
    @EnvironmentObject var navigation: NavigationManager
    
    @State private var isHovering = false
    
    var body: some View {
        HStack(spacing: 12) {
            // Track number or play button
            ZStack {
                Text("\(index)")
                    .opacity(isHovering ? 0 : 1)
                
                Button(action: {
                    Task {
                        do {
                            try await playAlbum(albumID: albumID, position: Int32(index) - 1)
                            await player.fetchQueue()
                        } catch {
                            errorText = String(describing: error)
                        }
                    }
                }) {
                    Image(systemName: "play.fill")
                        .font(.system(size: 10))
                        .opacity(isHovering ? 1 : 0)
                }
                .buttonStyle(.plain)
            }
            .frame(width: 30, alignment: .center)
            .foregroundStyle(.secondary)
            
            // Title
            Text(track.title)
                .lineLimit(1)
                .frame(maxWidth: .infinity, alignment: .leading)
            
            // Duration
            Text(formatDuration(track.duration))
                .foregroundStyle(.secondary)
                .frame(width: 50, alignment: .trailing)
            
            // Like button
            Button(action: {
                withAnimation(.easeInOut(duration: 0.2)) {
                    library.toggleLike(track)
                }
            }) {
                Image(systemName: library.isLiked(track) ? "heart.fill" : "heart")
                    .font(.system(size: 14))
                    .foregroundStyle(library.isLiked(track) ? Color(hex: "fe09a3") : .secondary)
            }
            .buttonStyle(.plain)
            .frame(width: 40, alignment: .center)
            
            
            /*
             Play Next
             Add to Playlist
             Play Last
             Add Shuffled
             */
            // Context menu button
            Menu {
                Button(action: {
                    Task {
                        do {
                          try await insertTracks(tracks: [track.path], position: Int32(PlaylistPosition.insertFirst))
                            await player.fetchQueue()
                        } catch {
                            errorText = String(describing: error)
                        }
                    }
                }) {
                    Label("Play Next", systemImage: "text.insert")
                }
                
                Menu {
                    Button(action: {
                        Task {
                            do {
                                let pl = try await createSavedPlaylist(
                                    name: track.title, trackIDs: [track.cuid])
                                let saved = SavedPlaylist(
                                    id: pl.id, name: pl.name,
                                    description: nil, image: nil, folderID: nil,
                                    trackCount: pl.trackCount)
                                savedPlaylists.append(saved)
                            } catch {
                                errorText = String(describing: error)
                            }
                        }
                    }) {
                        Label("New Playlist...", systemImage: "plus")
                    }
                    if !savedPlaylists.isEmpty {
                        Divider()
                        ForEach(savedPlaylists) { pl in
                            Button(action: {
                                Task {
                                    do {
                                        try await addTracksToSavedPlaylist(
                                            playlistID: pl.id, trackIDs: [track.cuid])
                                    } catch {
                                        errorText = String(describing: error)
                                    }
                                }
                            }) {
                                Label(pl.name, systemImage: "music.note.list")
                            }
                        }
                    }
                } label: {
                    Label("Add to Playlist", systemImage: "text.append")
                }
                .menuStyle(.borderlessButton)
                .menuIndicator(.hidden)
                
                Button(action: {
                    Task {
                        do {
                            try await insertTracks(tracks: [track.path], position: Int32(PlaylistPosition.insertLast))
                            await player.fetchQueue()
                        } catch {
                            errorText = String(describing: error)
                        }
                    }
                }) {
                    Label("Play Last", systemImage: "text.append")
                }
                
                Divider()
                
                Button(action: {
                    library.toggleLike(track)
                }) {
                    Label(library.isLiked(track) ? "Remove from Liked" : "Add to Liked",
                          systemImage: library.isLiked(track) ? "heart.slash" : "heart")
                }
                
                Divider()
                
                Button(action: {
                    Task {
                        await navigation.goToArtist(byId: track.artistID)
                    }
                }) {
                    Label("Go to Artist", systemImage: "music.mic")
                    
                }
            } label: {
                Image(systemName: "ellipsis")
                    .font(.system(size: 14))
                    .foregroundStyle(isHoveringMenu ? .primary : .secondary)
                    .frame(width: 32, height: 32)
                    .contentShape(Rectangle())
                }
                .menuStyle(.borderlessButton)
                .menuIndicator(.hidden)
                .frame(width: 40, alignment: .center)
                .opacity(isHovering ? 1 : 0)
                .onHover { hovering in
                    isHoveringMenu = hovering
            }
        }
        .font(.system(size: 12))
        .padding(.horizontal, 24)
        .padding(.vertical, 10)
        .background(isHovering ? Color.black.opacity(0.05) : (isEven ? Color.black.opacity(0.02) : Color.clear))
        .onHover { hovering in
            withAnimation(.easeInOut(duration: 0.15)) {
                isHovering = hovering
            }
        }
        .task {
            if savedPlaylists.isEmpty {
                if let data = try? await fetchSavedPlaylists() {
                    savedPlaylists = data.map {
                        SavedPlaylist(
                            id: $0.id, name: $0.name,
                            description: $0.hasDescription_p ? $0.description_p : nil,
                            image: $0.hasImage ? $0.image : nil,
                            folderID: $0.hasFolderID ? $0.folderID : nil,
                            trackCount: $0.trackCount
                        )
                    }
                }
            }
        }
    }
    
    private func formatDuration(_ duration: TimeInterval) -> String {
        let minutes = Int(duration) / 60
        let seconds = Int(duration) % 60
        return String(format: "%d:%02d", minutes, seconds)
    }
}
