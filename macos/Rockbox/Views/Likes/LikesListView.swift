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
    @State private var isLoading = true
    @ObservedObject var library: MusicLibrary
    
    var body: some View {
        VStack(spacing: 0) {
            if isLoading {
                ProgressView()
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else if likedSongs.isEmpty {
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
                // Song list
                ScrollView {
                    LazyVStack(spacing: 0) {
                        // Header row
                        SongHeaderRow(showLike: true)
                        
                        Divider()
                        
                        // Liked song rows
                        ForEach(Array(likedSongs.enumerated()), id: \.element.id) { index, song in
                            SongRowView(
                                song: song,
                                index: index + 1,
                                isEven: index % 2 == 0,
                                showLike: true,
                                isLikesScreen: true,
                                library: library
                            )
                        }
                    }
                }
            }
        }
        .toolbar {
            ToolbarItemGroup(placement: .automatic) {
                if !likedSongs.isEmpty {
                    Button(action: { Task {
                        do {
                            try await playLikedTracks(shuffle: false)
                        } catch {
                            errorText = String(describing: error)
                        }
                    } }) {
                        Label("Play", systemImage: "play.fill")
                    }
                    
                    Button(action: { Task {
                        do {
                            try await playLikedTracks(shuffle: true)
                        } catch {
                            errorText = String(describing: error)
                        }
                    } }) {
                        Label("Shuffle", systemImage: "shuffle")
                    }
                    
                    Spacer()
                    
                    Text("\(likedSongs.count) songs")
                        .foregroundStyle(.secondary)
                }
            }
        }
        .toolbarBackground(.ultraThinMaterial, for: .windowToolbar)
        .task {
            do {
                let data = try await fetchLikedTracks()
                likedSongs = []
                for track in data {
                    let song = Song(
                        cuid: track.id,
                        path: track.path,
                        title: track.title,
                        artist: track.artist,
                        album: track.album,
                        albumArt: URL(string: "http://localhost:6062/covers/" + track.albumArt),
                        duration: TimeInterval(track.length / 1000),
                        trackNumber: Int(track.trackNumber),
                        discNumber: Int(track.discNumber),
                        albumID: track.albumID,
                        artistID: track.artistID,
                        color: .gray.opacity(0.3)
                    )
                    library.likedSongIds.insert(song.cuid)
                    likedSongs.append(song)
                }
                isLoading = false
            } catch {
                errorText = String(describing: error)
                isLoading = false
            }
        }
        .alert("gRPC Error", isPresented: .constant(errorText != nil)) {
            Button("OK") { errorText = nil }
        } message: {
            Text(errorText ?? "")
        }
    }
}
