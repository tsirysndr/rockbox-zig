//
//  AlbumHeaderView.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 14/12/2025.
//

import SwiftUI

struct AlbumHeaderView: View {
    let album: Album
    let totalDuration: TimeInterval
    let trackCount: Int
    var onBack: () -> Void
    @State private var errorText: String?
    
    var body: some View {
        HStack(alignment: .top, spacing: 24) {
            // Back button and album cover
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
                
                // Album cover
                RoundedRectangle(cornerRadius: 4)
                    .fill(album.color.gradient)
                    .frame(width: 240, height: 240)
                    .overlay {
                        AsyncImage(url: URL(string: album.cover)) { phase in
                            switch phase {
                            case .empty:
                                Image(systemName: "music.note")
                                    .font(.system(size: 60))
                                    .foregroundStyle(.white.opacity(0.6))
                            case .success(let image):
                                image
                                    .resizable()
                                    .aspectRatio(contentMode: .fill)
                            case .failure:
                                Image(systemName: "music.note")
                                    .font(.system(size: 60))
                                    .foregroundStyle(.white.opacity(0.6))
                            @unknown default:
                                EmptyView()
                            }
                        }
                    }
                    .clipShape(RoundedRectangle(cornerRadius: 4))
            }
            
            // Album info
            VStack(alignment: .leading, spacing: 8) {
                Text(album.title)
                    .font(.system(size: 24, weight: .bold))
                
                Text(album.artist)
                    .font(.system(size: 16))
                    .foregroundStyle(.secondary)
                
                Text("\(trackCount) songs · \(album.year.formatted(.number.grouping(.never))) · \(formatDuration(totalDuration))")
                    .font(.system(size: 13))
                    .foregroundStyle(.tertiary)
                    .padding(.top, 4)
                
                Spacer()
                
                // Action buttons
                HStack(spacing: 12) {
                    Button(action: {
                        Task {
                            do {
                                try await playAlbum(albumID: album.cuid)
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
                            try await playAlbum(albumID: album.cuid, shuffle: true)
                        } catch {
                            errorText = String(describing: error)
                        }
                    } }) {
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
            .padding(.top, 28) // Align with album cover top
        }
        .padding(24)
    }
    
    private func formatDuration(_ duration: TimeInterval) -> String {
        let totalMinutes = Int(duration) / 60
        if totalMinutes >= 60 {
            let hours = totalMinutes / 60
            let minutes = totalMinutes % 60
            return "\(hours) hr \(minutes) min"
        }
        return "\(totalMinutes) min"
    }
}

