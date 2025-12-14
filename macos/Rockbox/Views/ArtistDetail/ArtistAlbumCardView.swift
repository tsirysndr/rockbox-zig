//
//  ArtistAlbumCardView.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 14/12/2025.
//

import SwiftUI

struct ArtistAlbumCardView: View {
    let album: Album
    @State private var isHovering = false
    
    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            // Album artwork
            ZStack {
                RoundedRectangle(cornerRadius: 6)
                    .fill(album.color.gradient)
                    .aspectRatio(1, contentMode: .fit)
                    .shadow(color: .black.opacity(0.15), radius: isHovering ? 8 : 3, y: isHovering ? 4 : 2)
                
                Image(systemName: "music.note")
                    .font(.system(size: 32))
                    .foregroundStyle(.white.opacity(0.6))
                
                // Play button on hover
                if isHovering {
                    ZStack {
                        Circle()
                            .fill(.black.opacity(0.5))
                            .frame(width: 36, height: 36)
                        
                        Image(systemName: "play.fill")
                            .font(.system(size: 14))
                            .foregroundStyle(.white)
                    }
                }
            }
            .onHover { hovering in
                withAnimation(.easeInOut(duration: 0.2)) {
                    isHovering = hovering
                }
            }
            
            // Album info
            VStack(alignment: .leading, spacing: 2) {
                Text(album.title)
                    .font(.system(size: 11, weight: .medium))
                    .lineLimit(1)
                
                Text(String(album.year))
                    .font(.system(size: 10))
                    .foregroundStyle(.secondary)
            }
        }
        .contentShape(Rectangle())
    }
}

