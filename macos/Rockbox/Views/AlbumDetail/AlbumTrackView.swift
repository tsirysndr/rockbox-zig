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
    @ObservedObject var library: MusicLibrary
    
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
