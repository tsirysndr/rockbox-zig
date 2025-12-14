//
//  LikesListView.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 14/12/2025.
//

import SwiftUI


struct LikesListView: View {
    @ObservedObject var library: MusicLibrary
    
    var likedSongs: [Song] {
        library.likedSongs(from: sampleSongs)
    }
    
    var body: some View {
        if likedSongs.isEmpty {
            VStack(spacing: 12) {
                Image(systemName: "heart.slash")
                    .font(.system(size: 48))
                    .foregroundStyle(.tertiary)
                
                Text("No liked songs yet")
                    .font(.title3)
                    .foregroundStyle(.secondary)
                
                Text("Tap the heart icon on any song to add it here")
                    .font(.subheadline)
                    .foregroundStyle(.tertiary)
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)
        } else {
            ScrollView {
                LazyVStack(spacing: 0) {
                    // Header row
                    SongHeaderRow(showLike: true)
                    
                    Divider()
                    
                    // Liked song rows
                    ForEach(Array(likedSongs.enumerated()), id: \.element.id) { index, song in
                        SongRowView(song: song, index: index + 1, isEven: index % 2 == 0, showLike: true, library: library)
                    }
                }
            }
        }
    }
}

