//
//  SongsListView.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 14/12/2025.
//

import SwiftUI

struct SongsListView: View {
    @State private var songs: [Song] = []
    @State private var errorText: String?
    @ObservedObject var library: MusicLibrary
    
    var body: some View {
        ScrollView {
            LazyVStack(spacing: 0) {
                // Header row
                SongHeaderRow(showLike: true)
                
                Divider()
                
                // Song rows
                ForEach(Array(songs.enumerated()), id: \.element.id) { index, song in
                    SongRowView(song: song, index: index + 1, isEven: index % 2 == 0, showLike: true, library: library)
                }
            }
        }
        .task {
            do {
                let data = try await fetchTracks()
                songs = []
                for track in data {
                    songs.append(Song(cuid: track.id, title: track.title, artist: track.artist, album: track.album, albumArt: URL(string: "http://localhost:6062/covers/" + track.albumArt), duration: TimeInterval(track.length / 1000), color: .gray.opacity(0.3)))
                }
                
                let likes = try await fetchLikedTracks()
                for track in likes {
                    let song = Song(cuid: track.id, title: track.title, artist: track.artist, album: track.album, albumArt: URL(string: "http://localhost:6062/covers/" + track.albumArt), duration: TimeInterval(track.length / 1000), color: .gray.opacity(0.3))
                    library.likedSongIds.insert(song.cuid)
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

    }
}
