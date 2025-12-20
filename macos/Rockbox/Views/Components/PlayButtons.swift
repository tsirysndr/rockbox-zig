//
//  PlayButtons.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 20/12/2025.
//

import SwiftUI

struct PlayShuffleButtons: View {
    var onPlay: () -> Void
    var onShuffle: () -> Void
    var songCount: Int
    
    @State private var isHoveringPlay = false
    @State private var isHoveringShuffle = false
    
    var body: some View {
        HStack(spacing: 12) {
            Button(action: onPlay) {
                HStack(spacing: 6) {
                    Image(systemName: "play.fill")
                        .font(.system(size: 12))
                    Text("Play")
                        .font(.system(size: 13, weight: .medium))
                }
                .foregroundStyle(.white)
                .padding(.horizontal, 16)
                .padding(.vertical, 8)
                .background(
                    RoundedRectangle(cornerRadius: 6)
                        .fill(isHoveringPlay ? Color.accentColor.opacity(0.8) : Color.accentColor)
                )
            }
            .buttonStyle(.plain)
            .onHover { isHoveringPlay = $0 }
            
            Button(action: onShuffle) {
                HStack(spacing: 6) {
                    Image(systemName: "shuffle")
                        .font(.system(size: 12))
                    Text("Shuffle")
                        .font(.system(size: 13, weight: .medium))
                }
                .foregroundStyle(.primary)
                .padding(.horizontal, 16)
                .padding(.vertical, 8)
                .background(
                    RoundedRectangle(cornerRadius: 6)
                        .fill(isHoveringShuffle ? Color.secondary.opacity(0.2) : Color.secondary.opacity(0.1))
                )
            }
            .buttonStyle(.plain)
            .onHover { isHoveringShuffle = $0 }
            
            Spacer()
            
            Text("\(songCount) songs")
                .font(.system(size: 12))
                .foregroundStyle(.secondary)
        }
        .padding(.horizontal, 20)
        .padding(.vertical, 12)
    }
}
