//
//  ArtistAlbumCardView.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 14/12/2025.
//

import SwiftUI

struct ArtistAlbumCardView: View {
    let album: Album
    var onSelect: () -> Void

    @State private var isHovering = false
    @State private var errorText: String?
    
    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            // Album artwork
            ZStack {
                RoundedRectangle(cornerRadius: 6)
                    .fill(album.color.gradient)
                    .aspectRatio(1, contentMode: .fit)
                    .overlay {
                        AsyncImage(url: URL(string: album.cover)) { phase in
                            switch phase {
                            case .empty:
                                Image(systemName: "music.note")
                                    .font(.system(size: 40))
                                    .foregroundStyle(.white.opacity(0.6))
                            case .success(let image):
                                image
                                    .resizable()
                                    .aspectRatio(contentMode: .fill)
                            case .failure:
                                Image(systemName: "music.note")
                                    .font(.system(size: 40))
                                    .foregroundStyle(.white.opacity(0.6))
                            @unknown default:
                                EmptyView()
                            }
                        }
                    }
                    .clipShape(RoundedRectangle(cornerRadius: 5))
                    .onTapGesture {
                        onSelect()
                    }

              
                
                // Play button on hover
                if isHovering {
                    Button(action: {
                        Task {
                            do {
                                try await playAlbum(albumID: album.cuid)
                            } catch {
                                errorText = String(describing: error)
                            }
                        }
                    }) {
                        ZStack {
                            Circle()
                                .fill(.black.opacity(0.5))
                                .frame(width: 36, height: 36)
                            
                            Image(systemName: "play.fill")
                                .font(.system(size: 14))
                                .foregroundStyle(.white)
                        }
                    }.buttonStyle(.borderless)
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
            .onTapGesture {
                onSelect()
            }

        }
        .contentShape(Rectangle())
    }
}

