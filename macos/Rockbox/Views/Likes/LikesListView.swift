//
//  LikesListView.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 14/12/2025.
//

import SwiftUI


struct LikesListView: View {
    @State private var likedSongs: [Song] = []
    @State private var errorText: String?
    @ObservedObject var library: MusicLibrary
    
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
            .task {
                do {
                    let data = try await fetchLikedTracks()
                    likedSongs = []
                    for track in data {
                        let song = Song(cuid: track.id, title: track.title, artist: track.artist, album: track.album, albumArt: URL(string: "http://localhost:6062/covers/" + track.albumArt), duration: TimeInterval(track.length / 1000), trackNumber: Int(track.trackNumber), discNumber: Int(track.discNumber),color: .gray.opacity(0.3))
                        library.likedSongIds.insert(song.cuid)
                        likedSongs.append(song)
                    }
                    
                } catch {
                    errorText = String(describing: error)
                }
            }
            .alert("gRPC Error", isPresented: .constant(errorText != nil)) {
              Button("OK") { errorText = nil }
             } message: {
               Text(errorText ?? "")
             }

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

