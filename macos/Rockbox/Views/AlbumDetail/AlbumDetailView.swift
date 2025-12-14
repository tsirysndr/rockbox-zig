//
//  AlbumDetailView.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 14/12/2025.
//

import SwiftUI

struct AlbumDetailView: View {
    let album: Album
    @ObservedObject var library: MusicLibrary
    var onBack: () -> Void
    
    var totalDuration: TimeInterval {
        album.tracks.reduce(0) { $0 + $1.duration }
    }
    
    var body: some View {
        ScrollView {
            VStack(spacing: 0) {
                // Album header
                AlbumHeaderView(album: album, totalDuration: totalDuration, onBack: onBack)
                
                Divider()
                    .padding(.top, 20)
                
                // Track list
                LazyVStack(spacing: 0) {
                    ForEach(Array(album.tracks.enumerated()), id: \.element.id) { index, track in
                        AlbumTrackRowView(
                            track: track,
                            index: index + 1,
                            isEven: index % 2 == 0,
                            library: library
                        )
                    }
                }
            }
        }
    }
}
