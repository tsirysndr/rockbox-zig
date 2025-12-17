//
//  PlayerControlsView.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 14/12/2025.
//

import SwiftUI

struct PlayerControlsView: View {
    @EnvironmentObject var player: PlayerState
    @State private var isHoveringProgress = false
    @State private var isHoveringTrackInfo = false
    @State private var isCurrentTrackLiked = false
    
    var body: some View {
        HStack(spacing: 0) {
            // Playback controls (left, but centered in its space)
            HStack( alignment: .center, spacing: 16) {
                Button(action: { /* previous */ }) {
                    Image(systemName: "backward.fill")
                        .font(.system(size: 13))
                }
                .buttonStyle(.plain)
                
                Button(action: { player.isPlaying.toggle() }) {
                    Image(systemName: player.isPlaying ? "pause.fill" : "play.fill")
                        .font(.system(size: 16))
                }
                .buttonStyle(.plain)
                
                Button(action: { /* next */ }) {
                    Image(systemName: "forward.fill")
                        .font(.system(size: 13))
                }
                .buttonStyle(.plain)
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
                    .clipShape(RoundedRectangle(cornerRadius: 0))
              
                
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
                        
                        // Heart button (shows on hover or when liked)
                        Button(action: {
                            withAnimation(.easeInOut(duration: 0.2)) {
                                isCurrentTrackLiked.toggle()
                            }
                        }) {
                            Image(systemName: isCurrentTrackLiked ? "heart.fill" : "heart")
                                .font(.system(size: 12))
                                .foregroundStyle(isCurrentTrackLiked ? Color(hex: "fe09a3") : .secondary)
                        }
                        .buttonStyle(.plain)
                        .opacity(isHoveringTrackInfo || isCurrentTrackLiked ? 1 : 0)
                        
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
                                // Track background
                                Capsule()
                                    .fill(.quaternary)
                                    .frame(height: isHoveringProgress ? 6 : 3)
                                
                                // Progress fill
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
            
            // Volume (right)
            HStack(spacing: 8) {
                Image(systemName: "speaker.fill")
                    .font(.system(size: 10))
                    .foregroundStyle(.secondary)
                
                Slider(value: .constant(0.7))
                    .frame(width: 80)
                
                Image(systemName: "speaker.wave.3.fill")
                    .font(.system(size: 10))
                    .foregroundStyle(.secondary)
            }
            .frame(width: 120)
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

