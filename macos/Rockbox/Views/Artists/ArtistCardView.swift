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
    @State private var isHoveringPlayButton = false
    @State private var errorText: String? = nil
    
    var body: some View {
        VStack(spacing: 8) {
            // Artist artwork (circular)
            ZStack {
                Circle()
                    .fill(artist.color.gradient)
                    .aspectRatio(1, contentMode: .fit)
                    .overlay {
                        AsyncImage(url: URL(string: artist.image ?? "")) { phase in
                            switch phase {
                            case .empty:
                                Image(systemName: "music.mic")
                                    .font(.system(size: 40))
                                    .foregroundStyle(.white.opacity(0.8))
                            case .success(let image):
                                image
                                    .resizable()
                                    .aspectRatio(contentMode: .fill)
                            case .failure:
                                Image(systemName: "music.mic")
                                    .font(.system(size: 40))
                                    .foregroundStyle(.white.opacity(0.8))
                            @unknown default:
                                EmptyView()
                            }
                        }
                    }
                    .clipShape(Circle())
                    .shadow(color: .black.opacity(artist.image != nil ? 0.0 : 0.2), radius: 10, y: 5)
                    
                
                // Play button on hover
                if isHovering {
                    Button(action: {
                        Task {
                            do {
                                try await playArtistTracks(artistID: artist.cuid)
                            } catch {
                                errorText = String(describing: error)
                            }
                        }
                    }) {
                        ZStack {
                            Circle()
                                .fill(isHoveringPlayButton ? Color(hex: "fe09a3") : .white.opacity(0.3))
                                .frame(width: 44, height: 44)
                            
                            Image(systemName: "play.fill")
                                .font(.system(size: 18))
                                .foregroundStyle(.white)
                        }
                    }
                    .buttonStyle(.borderless)
                    .onHover { hovering in
                        withAnimation(.easeInOut(duration: 0.15)) {
                            isHoveringPlayButton = hovering
                        }
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

