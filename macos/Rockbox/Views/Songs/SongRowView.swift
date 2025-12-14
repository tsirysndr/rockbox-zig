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
    @ObservedObject var library: MusicLibrary
    
    @State private var isHovering = false
    
    var body: some View {
        HStack(spacing: 12) {
            // Track number or play button
            ZStack {
                Text("\(index)")
                    .opacity(isHovering ? 0 : 1)
                
                Image(systemName: "play.fill")
                    .font(.system(size: 10))
                    .opacity(isHovering ? 1 : 0)
            }
            .frame(width: 30, alignment: .center)
            
            // Title with artwork
            HStack(spacing: 10) {
                RoundedRectangle(cornerRadius: 0)
                    .fill(song.color.gradient)
                    .frame(width: 36, height: 36)
                    .overlay {
                        AsyncImage(url: song.albumArt) { phase in
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
                        .foregroundStyle(library.isLiked(song) ? Color(hex:"#fe09a3") : .secondary)
                }
                .buttonStyle(.plain)
                .frame(width: 40, alignment: .center)
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
    }
    
    private func formatDuration(_ duration: TimeInterval) -> String {
        let minutes = Int(duration) / 60
        let seconds = Int(duration) % 60
        return String(format: "%d:%02d", minutes, seconds)
    }
}


