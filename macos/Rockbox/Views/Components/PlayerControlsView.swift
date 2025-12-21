//
//  PlayerControlsView.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 14/12/2025.
//

import SwiftUI

struct PlayerControlsView: View {
    @EnvironmentObject var player: PlayerState
    @EnvironmentObject var navigation: NavigationManager
    @State private var isHoveringProgress = false
    @State private var isHoveringTrackInfo = false
    @State private var isHoveringQueue = false
    @State private var isHoveringMenu = false
    @State private var isHoveringShuffle = false
    @State private var isHoveringRepeat = false
    @State private var errorText: String? = nil
    @ObservedObject var library: MusicLibrary
    @Binding var showQueue: Bool  // Add this binding
    
    
    var body: some View {
        HStack(spacing: 0) {
            // Playback controls (left)
            HStack(alignment: .center, spacing: 16) {
                Button(action: {
                    player.toggleShuffle()
                }) {
                    Image(systemName: "shuffle")
                        .font(.system(size: 14))
                        .foregroundStyle(player.isShuffleEnabled ? Color(hex: "fe09a3") : (isHoveringShuffle ? .primary : .secondary))
                }
                .buttonStyle(.plain)
                .onHover { isHoveringShuffle = $0 }
                
                Button(action: { player.playPreviousTrack() }) {
                    Image(systemName: "backward.fill")
                        .font(.system(size: 16))
                }
                .buttonStyle(.plain)
                
                Button(action: {
                    player.playOrPause()
                }) {
                    Image(systemName: player.isPlaying ? "pause.fill" : "play.fill")
                        .font(.system(size: 24))
                }
                .buttonStyle(.plain)
                
                Button(action: { player.playNextTrack() }) {
                    Image(systemName: "forward.fill")
                        .font(.system(size: 16))
                }
                .buttonStyle(.plain)
                
                Button(action: {
                    player.toggleRepeat()
                }) {
                    ZStack {
                        Image(systemName: player.repeatMode == .one ? "repeat.1" : "repeat")
                            .font(.system(size: 14))
                            .foregroundStyle(player.repeatMode != .off ? Color(hex: "fe09a3") : (isHoveringRepeat ? .primary : .secondary))
                    }
                }
                .buttonStyle(.plain)
                .onHover { isHoveringRepeat = $0 }
            }
            .foregroundStyle(.primary)
            .frame(maxWidth: 280)
            
            // Track info with artwork and progress (center)
            HStack(spacing: 10) {
                // Album artwork
                RoundedRectangle(cornerRadius: 4)
                    .fill(player.currentTrack.color.gradient)
                    .frame(width: 44, height: 44)
                    .overlay {
                        AsyncImage(url: player.currentTrack.albumArt) { phase in
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
                    .clipShape(RoundedRectangle(cornerRadius: 4))
                
                VStack(alignment: .leading, spacing: 0) {
                    // Track metadata with heart button
                    HStack(spacing: 8) {
                        Spacer()
                        
                        VStack(alignment: .center, spacing: 1) {
                            Text(player.currentTrack.title)
                                .font(.system(size: 12, weight: .medium))
                                .lineLimit(1)
                            Text("\(player.currentTrack.artist) — \(player.currentTrack.album)")
                                .font(.system(size: 10))
                                .foregroundStyle(.secondary)
                                .lineLimit(1)
                        }
                        
                        // Heart button
                        Button(action: {
                            withAnimation(.easeInOut(duration: 0.2)) {
                                library.toggleLike(player.currentTrack)
                            }
                        }) {
                            Image(systemName: library.isLiked(player.currentTrack) ? "heart.fill" : "heart")
                                .font(.system(size: 12))
                                .foregroundStyle(library.isLiked(player.currentTrack) ? Color(hex: "fe09a3") : .secondary)
                        }
                        .buttonStyle(.plain)
                        .opacity(isHoveringTrackInfo || library.isLiked(player.currentTrack) ? 1 : 0)
                        
                        // Context menu button
                        Menu {
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
                            
                            Divider()
                            
                            Button(action: {
                                Task {
                                    await navigation.goToAlbum(byId: player.currentTrack.albumID)
                                }
                            }) {
                                Label("Go to Album", systemImage: "square.stack")
                            }
                            
                            Button(action: {
                                Task {
                                    await navigation.goToArtist(byId: player.currentTrack.artistID)
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
                            .opacity(isHoveringTrackInfo ? 1 : 0)
                            .onHover { hovering in
                                isHoveringMenu = hovering
                        }
                        
                        Spacer()
                    }
                    
                    Spacer()
                        .frame(height: 6)
                    
                    // Progress bar with times
                    HStack(spacing: 3) {
                        Text(formatTime(player.currentTime))
                            .font(.system(size: 9, weight: .medium).monospacedDigit())
                            .foregroundStyle(.tertiary)
                            .frame(width: 32, alignment: .trailing)
                        
                        GeometryReader { geometry in
                            ZStack(alignment: .leading) {
                                Capsule()
                                    .fill(.quaternary)
                                    .frame(height: isHoveringProgress ? 6 : 3)
                                
                                Capsule()
                                    .fill(.primary.opacity(0.8))
                                    .frame(width: geometry.size.width * player.progress, height: isHoveringProgress ? 6 : 3)
                            }
                            .frame(maxHeight: .infinity)
                            .contentShape(Rectangle())
                            .gesture(
                                DragGesture(minimumDistance: 0)
                                    .onChanged { value in
                                        let progress = max(0, min(1, value.location.x / geometry.size.width))
                                        player.progress = progress
                                    }
                                    .onEnded { _ in
                                        player.seek(position: Int64(player.currentTime * 1000))
                                    }
                            )
                            .onHover { hovering in
                                withAnimation(.easeInOut(duration: 0.15)) {
                                    isHoveringProgress = hovering
                                }
                            }
                        }
                        .frame(height: 10)
                        
                        Text(formatTime(player.duration))
                            .font(.system(size: 9, weight: .medium).monospacedDigit())
                            .foregroundStyle(.tertiary)
                            .frame(width: 32, alignment: .leading)
                    }
                }
                .frame(maxWidth: 800)
            }
            .frame(maxWidth: .infinity)
            .onHover { hovering in
                withAnimation(.easeInOut(duration: 0.15)) {
                    isHoveringTrackInfo = hovering
                }
            }
            
            Button(action: {
                withAnimation(.easeInOut(duration: 0.2)) {
                    showQueue.toggle()
                }
            }) {
                Image(systemName: "list.bullet")
                    .font(.system(size: 14))
                    .foregroundStyle(showQueue ? .primary : .secondary)
                    .frame(width: 32, height: 32)
                    .background(
                        RoundedRectangle(cornerRadius: 6)
                            .fill(isHoveringQueue || showQueue ? Color.secondary.opacity(0.15) : Color.clear)
                    )
                    .contentShape(Rectangle())
            }
            .buttonStyle(.plain)
            .onHover { hovering in
                withAnimation(.easeInOut(duration: 0.1)) {
                    isHoveringQueue = hovering
                }
            }
            .frame(width: 60)
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 10)
        .background(.bar)
    }
    
    private func formatTime(_ time: TimeInterval) -> String {
        let minutes = Int(time) / 60
        let seconds = Int(time) % 60
        return String(format: "%d:%02d", minutes, seconds)
    }
}
