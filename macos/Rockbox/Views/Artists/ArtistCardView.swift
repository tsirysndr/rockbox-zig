//
//  ArtistCardView.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 14/12/2025.
//

import SwiftUI

struct ArtistCardView: View {
    let artist: Artist
    @State private var isHovering = false
    
    var body: some View {
        VStack(spacing: 8) {
            // Artist artwork (circular)
            ZStack {
                Circle()
                    .fill(artist.color.gradient)
                    .aspectRatio(1, contentMode: .fit)
                    .shadow(color: .black.opacity(0.2), radius: isHovering ? 10 : 4, y: isHovering ? 6 : 2)
                
                Image(systemName: "music.mic")
                    .font(.system(size: 40))
                    .foregroundStyle(.white.opacity(0.6))
                
                // Play button on hover
                if isHovering {
                    ZStack {
                        Circle()
                            .fill(.black.opacity(0.5))
                            .frame(width: 44, height: 44)
                        
                        Image(systemName: "play.fill")
                            .font(.system(size: 18))
                            .foregroundStyle(.white)
                    }
                }
            }
            .onHover { hovering in
                withAnimation(.easeInOut(duration: 0.2)) {
                    isHovering = hovering
                }
            }
            
            // Artist info (centered)
            VStack(spacing: 2) {
                Text(artist.name)
                    .font(.system(size: 12, weight: .medium))
                    .lineLimit(1)
                
                Text(artist.genre)
                    .font(.system(size: 11))
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
            }
        }
        .contentShape(Rectangle())
    }
}

