//
//  QueueView.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 20/12/2025.
//

import SwiftUI

struct QueueView: View {
    @EnvironmentObject var player: PlayerState
    @State private var showPlayingNext: Bool = true

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            // Header
            HStack(spacing: 0) {
                Text(!showPlayingNext ? "History" : "Playing Next")
                    .font(.headline)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .padding()

                Text("\(player.currentIndex + 1) of \(player.playlistLength)")
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .center)
                    .padding()

                Button(action: {
                    showPlayingNext.toggle()
                }) {
                    Text(showPlayingNext ? "History" : "Playing Next")
                        .padding()
                }
                .buttonStyle(.borderless)
                .frame(maxWidth: .infinity, alignment: .trailing)
            }
            
            Divider()
            
            if player.upNext.isEmpty {
                VStack(spacing: 12) {
                    Image(systemName: "music.note.list")
                        .font(.system(size: 32))
                        .foregroundStyle(.tertiary)
                    Text("No upcoming songs")
                        .foregroundStyle(.secondary)
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                ScrollView {
                    LazyVStack(spacing: 0) {
                        ForEach(Array(showPlayingNext ? player.upNext.enumerated() : player.history.enumerated()), id: \.element.id) { index, song in
                            QueueRowView(
                                song: song,
                                index: index,
                                onTap: {
                                    player.playFromQueue(at: showPlayingNext ? player.currentIndex + 1 + index : index)
                                }
                            )
                        }
                    }
                }
            }
        }
        .frame(minWidth: 350)
        .background(.background)
    }
}

struct QueueRowView: View {
    let song: Song
    let index: Int
    var onTap: () -> Void
    @State private var isHovering = false
    
    var body: some View {
        HStack(spacing: 12) {
            // Album art
            RoundedRectangle(cornerRadius: 4)
                .fill(song.color.gradient)
                .frame(width: 40, height: 40)
                .overlay {
                    CachedAsyncImage(url: song.albumArt) { phase in
                        switch phase {
                        case .success(let image):
                            image
                                .resizable()
                                .aspectRatio(contentMode: .fill)
                        default:
                            Image(systemName: "music.note")
                                .font(.system(size: 12))
                                .foregroundStyle(.white.opacity(0.6))
                        }
                    }
                }
                .clipShape(RoundedRectangle(cornerRadius: 4))
            
            VStack(alignment: .leading, spacing: 2) {
                Text(song.title)
                    .font(.system(size: 13))
                    .lineLimit(1)
                
                Text(song.artist)
                    .font(.system(size: 11))
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
            }
            
            Spacer()
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 8)
        .background(isHovering ? Color.secondary.opacity(0.1) : Color.clear)
        .contentShape(Rectangle())
        .onTapGesture {
            onTap()
        }
        .onHover { isHovering = $0 }
    }
}

