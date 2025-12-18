//
//  ArtistHeaderView.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 14/12/2025.
//

import SwiftUI

struct ArtistHeaderView: View {
    let artist: Artist
    let trackCount: Int
    let albumCount: Int
    var onBack: () -> Void
    @State private var errorText: String?

    
    var body: some View {
        HStack(alignment: .top, spacing: 24) {
            // Back button and artist image
            VStack(alignment: .leading, spacing: 12) {
                // Back button
                Button(action: onBack) {
                    HStack(spacing: 4) {
                        Image(systemName: "chevron.left")
                            .font(.system(size: 14, weight: .semibold))
                        Text("Back")
                            .font(.system(size: 13))
                    }
                    .foregroundStyle(.secondary)
                }
                .buttonStyle(.plain)
                
                // Artist image (circular)
                Circle()
                    .fill(artist.color.gradient)
                    .frame(width: 200, height: 200)
                    .overlay {
                        AsyncImage(url: URL(string: artist.image ?? "")) { phase in
                            switch phase {
                            case .empty:
                                Image(systemName: "music.mic")
                                    .font(.system(size: 60))
                                    .foregroundStyle(.white.opacity(0.6))
                            case .success(let image):
                                image
                                    .resizable()
                                    .aspectRatio(contentMode: .fill)
                            case .failure:
                                Image(systemName: "music.mic")
                                    .font(.system(size: 40))
                                    .foregroundStyle(.white.opacity(0.6))
                            @unknown default:
                                EmptyView()
                            }
                        }
                    }
                    .clipShape(Circle())
                    .shadow(color: .black.opacity(artist.image != nil ? 0.0 : 0.2), radius: 10, y: 5)
            }
            
            // Artist info
            VStack(alignment: .leading, spacing: 8) {
                Text(artist.name)
                    .font(.system(size: 28, weight: .bold))
                
                Text("\(albumCount) albums · \(trackCount) songs")
                    .font(.system(size: 13))
                    .foregroundStyle(.tertiary)
                    .padding(.top, 4)
                
                Spacer()
                
                // Action buttons
                HStack(spacing: 12) {
                    Button(action: {
                        Task {
                            do {
                                try await playArtistTracks(artistID: artist.cuid)
                            } catch {
                                errorText = String(describing: error)
                            }
                        }
                    }) {
                        HStack(spacing: 6) {
                            Image(systemName: "play.fill")
                                .font(.system(size: 12))
                            Text("Play")
                                .font(.system(size: 13, weight: .medium))
                        }
                        .padding(.horizontal, 20)
                        .padding(.vertical, 8)
                        .background(Color(hex: "fe09a3"))
                        .foregroundStyle(.white)
                        .cornerRadius(20)
                    }
                    .buttonStyle(.plain)
                    
                    Button(action: {
                        Task {
                            do {
                                try await playArtistTracks(artistID: artist.cuid, shuffle: true)
                            } catch {
                                errorText = String(describing: error)
                            }
                        }
                    }) {
                        HStack(spacing: 6) {
                            Image(systemName: "shuffle")
                                .font(.system(size: 12))
                            Text("Shuffle")
                                .font(.system(size: 13, weight: .medium))
                        }
                        .padding(.horizontal, 20)
                        .padding(.vertical, 8)
                        .background(Color.black.opacity(0.05))
                        .foregroundStyle(.primary)
                        .cornerRadius(20)
                    }
                    .buttonStyle(.plain)
                }
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(.top, 28)
        }
        .padding(24)
    }
}

