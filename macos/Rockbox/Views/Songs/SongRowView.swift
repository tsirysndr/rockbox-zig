//
//  SongRowView.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 14/12/2025.
//

import SwiftUI

struct SongRowView: View {
    let song: Song
    let index: Int
    let isEven: Bool
    var showLike: Bool = false
    var isLikesScreen: Bool = false
    @State private var errorText: String?
    @ObservedObject var library: MusicLibrary
    @EnvironmentObject var player: PlayerState
    
    @State private var isHovering = false
    @State private var isHoveringMenu = false
    
    var body: some View {
        HStack(spacing: 12) {
            // Track number or play button
            ZStack {
                Text("\(index)")
                    .opacity(isHovering ? 0 : 1)
                Button(action: {
                    Task {
                        do {
                            if isLikesScreen {
                                try await playLikedTracks(position: Int32(index) - 1)
                                return
                            }
                            try await playAllTracks(position: Int32(index) - 1)
                        } catch {
                            errorText = String(describing: error)
                        }
                    }
                }) {
                    Image(systemName: "play.fill")
                        .font(.system(size: 10))
                        .opacity(isHovering ? 1 : 0)
                }
                .opacity(isHovering ? 1 : 0)
                .buttonStyle(.plain)
            }
            .frame(width: 30, alignment: .center)
            
            // Title with artwork
            HStack(spacing: 10) {
                RoundedRectangle(cornerRadius: 0)
                    .fill(song.color.gradient)
                    .frame(width: 36, height: 36)
                    .overlay {
                        CachedAsyncImage(url: song.albumArt) { phase in
                            switch phase {
                            case .success(let image):
                                image
                                    .resizable()
                                    .aspectRatio(contentMode: .fill)
                            default:
                                Image(systemName: "music.note")
                                    .font(.system(size: 14))
                                    .foregroundStyle(.white.opacity(0.8))
                            }
                        }
                    }
                    .clipShape(RoundedRectangle(cornerRadius: 0))
                
                Text(song.title)
                    .lineLimit(1)
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            
            // Artist
            Text(song.artist)
                .foregroundStyle(.secondary)
                .lineLimit(1)
                .frame(width: 150, alignment: .leading)
            
            // Album
            Text(song.album)
                .foregroundStyle(.secondary)
                .lineLimit(1)
                .frame(width: 180, alignment: .leading)
            
            // Duration
            Text(formatDuration(song.duration))
                .foregroundStyle(.secondary)
                .frame(width: 50, alignment: .center)
            
            // Like button
            if showLike {
                Button(action: {
                    withAnimation(.easeInOut(duration: 0.2)) {
                        library.toggleLike(song)
                    }
                }) {
                    Image(systemName: library.isLiked(song) ? "heart.fill" : "heart")
                        .font(.system(size: 14))
                        .foregroundStyle(library.isLiked(song) ? Color(hex: "#fe09a3") : .secondary)
                }
                .buttonStyle(.plain)
                .frame(width: 40, alignment: .center)
            }
            
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
                            // Add to playlist
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
                    // Go to album action
                }) {
                    Label("Go to Album", systemImage: "square.stack")
                }
                
                Button(action: {
                    // Go to artist action
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
        .padding(.horizontal, 16)
        .padding(.vertical, 8)
        .background(isHovering ? Color.black.opacity(0.05) : (isEven ? Color.black.opacity(0.02) : Color.clear))
        .onHover { hovering in
            withAnimation(.easeInOut(duration: 0.15)) {
                isHovering = hovering
            }
        }
        .alert("Error", isPresented: .constant(errorText != nil)) {
            Button("OK") { errorText = nil }
        } message: {
            Text(errorText ?? "")
        }
    }
    
    private func formatDuration(_ duration: TimeInterval) -> String {
        let minutes = Int(duration) / 60
        let seconds = Int(duration) % 60
        return String(format: "%d:%02d", minutes, seconds)
    }
}
