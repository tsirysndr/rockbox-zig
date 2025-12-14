//
//  SongsListView.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 14/12/2025.
//

import SwiftUI

struct SongsListView: View {
    @ObservedObject var library: MusicLibrary
    
    var body: some View {
        ScrollView {
            LazyVStack(spacing: 0) {
                // Header row
                SongHeaderRow(showLike: true)
                
                Divider()
                
                // Song rows
                ForEach(Array(sampleSongs.enumerated()), id: \.element.id) { index, song in
                    SongRowView(song: song, index: index + 1, isEven: index % 2 == 0, showLike: true, library: library)
                }
            }
        }
    }
}
