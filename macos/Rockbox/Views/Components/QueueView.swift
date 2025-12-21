//
//  QueueView.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 20/12/2025.
//

import SwiftUI

struct QueueView: View {
    @EnvironmentObject var player: PlayerState
    @State private var showPlayingNext: Bool = true
    @State private var isHoveringClear: Bool = false
    @ObservedObject var library: MusicLibrary

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            // Header
            HStack(spacing: 0) {
                Text(!showPlayingNext ? "History" : "Playing Next")
                    .font(.headline)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .padding()

                Text("\(player.currentIndex + 1) of \(player.playlistLength)")
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .center)
                    .padding()

                Button(action: {
                    showPlayingNext.toggle()
                }) {
                    Text(showPlayingNext ? "History" : "Playing Next")
                        .padding()
                }
                .buttonStyle(.borderless)
                .frame(maxWidth: .infinity, alignment: .trailing)
            }
            
            Divider()
            
            if !player.upNext.isEmpty || !player.history.isEmpty {
                Button(action: {
                   player.clearQueue()
                }) {
                    Text("Clear")
                        .font(.system(size: 12))
                        .foregroundStyle(isHoveringClear ? .primary : .secondary)
                }
                .buttonStyle(.borderless)
                .onHover { isHoveringClear = $0 }
                .frame(maxWidth: .infinity, alignment: .center)
                .padding(.vertical, 8)
            }

            
            if player.upNext.isEmpty {
                VStack(spacing: 12) {
                    Image(systemName: "music.note.list")
                        .font(.system(size: 32))
                        .foregroundStyle(.tertiary)
                    Text("No upcoming songs")
                        .foregroundStyle(.secondary)
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                ScrollView {
                    LazyVStack(spacing: 0) {
                        ForEach(Array(showPlayingNext ? player.upNext.enumerated() : player.history.enumerated()), id: \.element.id) { index, song in
                            QueueRowView(
                                song: song,
                                index: index,
                                onTap: {
                                    player.playFromQueue(at: showPlayingNext ? player.currentIndex + 1 + index : index)
                                },
                                onRemove: {
                                    let queueIndex = showPlayingNext ? player.currentIndex + 1 + index : index
                                    player.removeFromQueue(at: queueIndex)
                                },
                                library: library,
                            )
                        }
                    }
                }
            }
        }
        .frame(minWidth: 350)
        .background(.background)
    }
}

struct QueueRowView: View {
    let song: Song
    let index: Int
    var onTap: () -> Void
    var onRemove: () -> Void
    @State private var errorText: String? = nil
    @State private var isHovering: Bool = false
    @State private var isHoveringMenu: Bool = false
    @State private var isHoveringRemove: Bool = false
    @ObservedObject var library: MusicLibrary
    @EnvironmentObject var player: PlayerState
    @EnvironmentObject var navigation: NavigationManager
    
    var body: some View {
        HStack(spacing: 12) {
            // Album art
            RoundedRectangle(cornerRadius: 4)
                .fill(song.color.gradient)
                .frame(width: 40, height: 40)
                .overlay {
                    CachedAsyncImage(url: song.albumArt) { phase in
                        switch phase {
                        case .success(let image):
                            image
                                .resizable()
                                .aspectRatio(contentMode: .fill)
                        default:
                            Image(systemName: "music.note")
                                .font(.system(size: 12))
                                .foregroundStyle(.white.opacity(0.6))
                        }
                    }
                }
                .overlay(alignment: .topLeading) {
                    if isHovering {
                        Button(action: onRemove) {
                            Image(systemName: "minus")
                                .font(.system(size: 8, weight: .bold))
                                .foregroundStyle(.white)
                                .frame(width: 14, height: 14)
                                .background(
                                    Circle()
                                        .fill(isHoveringRemove ? Color.red : Color.red.opacity(0.85))
                                        .shadow(color: .black.opacity(0.3), radius: 2, x: 0, y: 1)
                                )
                                .contentShape(Circle())
                        }
                        .buttonStyle(.plain)
                        .padding(2)
                        .onHover { isHoveringRemove = $0 }
                        .transition(.scale.combined(with: .opacity))
                    }
                }
                .clipShape(RoundedRectangle(cornerRadius: 4))
            
            VStack(alignment: .leading, spacing: 2) {
                Text(song.title)
                    .font(.system(size: 13))
                    .lineLimit(1)
                
                Text(song.artist)
                    .font(.system(size: 11))
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            
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
                            try await insertTracks(tracks: [song.path], position: Int32(PlaylistPosition.insertFirst))
                            await player.fetchQueue()
                        } catch {
                            errorText = String(describing: error)
                        }
                    }
                }) {
                    Label("Play Next", systemImage: "text.insert")
                }
                
                Button(action: {
                    Task {
                        do {
                            // try await addToQueueLast(songId: song.cuid)
                        } catch {
                            errorText = String(describing: error)
                        }
                    }
                }) {
                    Label("Add to Playlist", systemImage: "text.append")
                }
                
                Button(action: {
                    Task {
                        do {
                            try await insertTracks(tracks: [song.path], position: Int32(PlaylistPosition.insertLast))
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
                    library.toggleLike(song)
                }) {
                    Label(library.isLiked(song) ? "Remove from Liked" : "Add to Liked",
                          systemImage: library.isLiked(song) ? "heart.slash" : "heart")
                }
                
                Divider()
                
                Button(action: {
                    Task {
                        await navigation.goToAlbum(byId: song.albumID)
                    }
                }) {
                    Label("Go to Album", systemImage: "square.stack")
                }
                
                Button(action: {
                    Task {
                        await navigation.goToArtist(byId: song.artistID)
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
                .frame(width: 20, alignment: .center)
                .opacity(isHovering ? 1 : 0)
                .onHover { hovering in
                    isHoveringMenu = hovering
            }
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 8)
        .background(isHovering ? Color.secondary.opacity(0.1) : Color.clear)
        .contentShape(Rectangle())
        .onTapGesture {
            onTap()
        }
        .onHover { isHovering = $0 }
    }
}

