//
//  ArtistTrackRow.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 14/12/2025.
//

import SwiftUI

struct ArtistTrackRowView: View {
    let track: Song
    let artist: Artist
    let index: Int
    let isEven: Bool
    @ObservedObject var library: MusicLibrary
    @State private var errorText: String? = nil
    
    @State private var isHovering: Bool = false
    @State private var isHoveringMenu: Bool = false
    @EnvironmentObject var player: PlayerState
    @EnvironmentObject var navigation: NavigationManager
    
    var body: some View {
        HStack(spacing: 12) {
            // Track number or play button
            ZStack {
                Text("\(index)")
                    .opacity(isHovering ? 0 : 1)
                Button(action: {
                    Task {
                        do {
                            try await playArtistTracks(artistID: artist.cuid, position: Int32(index) - 1)
                        } catch {
                            errorText = String(describing: error)
                        }
                    }
                }) {
                    Image(systemName: "play.fill")
                        .font(.system(size: 10))
                        .opacity(isHovering ? 1 : 0)
                }.buttonStyle(.plain)
            }
            .frame(width: 30, alignment: .center)
            .foregroundStyle(.secondary)
            
            // Album artwork
            RoundedRectangle(cornerRadius: 4)
                .fill(track.color.gradient)
                .frame(width: 36, height: 36)
                .overlay {
                    AsyncImage(url: track.albumArt) { phase in
                        switch phase {
                        case .empty:
                            Image(systemName: "music.note")
                                .font(.system(size: 14))
                                .foregroundStyle(.white.opacity(0.8))
                        case .success(let image):
                            image
                                .resizable()
                                .aspectRatio(contentMode: .fill)
                        case .failure:
                            Image(systemName: "music.note")
                                .font(.system(size: 14))
                                .foregroundStyle(.white.opacity(0.8))
                        @unknown default:
                            EmptyView()
                        }
                    }
                }
                .clipShape(RoundedRectangle(cornerRadius: 0))
            
            // Title and album
            VStack(alignment: .leading, spacing: 2) {
                Text(track.title)
                    .font(.system(size: 12, weight: .medium))
                    .lineLimit(1)
                
                Text(track.album)
                    .font(.system(size: 10))
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            
            // Duration
            Text(formatDuration(track.duration))
                .font(.system(size: 11))
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
                
                Button(action: {
                    Task {
                        do {
                            // Add to Playlist
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
                        await navigation.goToAlbum(byId: track.albumID)
                    }
                }) {
                    Label("Go to Album", systemImage: "square.stack")
                }
                
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
    }
    
    private func formatDuration(_ duration: TimeInterval) -> String {
        let minutes = Int(duration) / 60
        let seconds = Int(duration) % 60
        return String(format: "%d:%02d", minutes, seconds)
    }
}

