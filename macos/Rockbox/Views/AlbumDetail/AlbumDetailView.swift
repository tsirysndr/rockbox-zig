//
//  AlbumDetailView.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 14/12/2025.
//

import SwiftUI

struct AlbumDetailView: View {
    let album: Album
    @State private var tracks: [Song] = []
    @State private var errorText: String?
    @ObservedObject var library: MusicLibrary
    var onBack: () -> Void
    
    var totalDuration: TimeInterval {
        tracks.reduce(0) { $0 + $1.duration }
    }
    
    var body: some View {
        ScrollView {
            VStack(spacing: 0) {
                // Album header
                AlbumHeaderView(album: album, totalDuration: totalDuration, trackCount: tracks.count,  onBack: onBack)
                    
                
                // Track list
                LazyVStack(spacing: 0) {
                    ForEach(Array(tracks.enumerated()), id: \.element.id) { index, track in
                        AlbumTrackRowView(
                            track: track,
                            index: index + 1,
                            isEven: index % 2 == 0,
                            library: library
                        )
                    }
                }
                .padding(.top, 20)
                .padding(.bottom, 50)
            }
        }
        .task {
            do {
                let data = try await fetchAlbum(id: album.cuid)
                for track in data.tracks {
                    tracks.append(Song(cuid: track.id, title: track.title, artist: track.artist, album: track.album, albumArt: URL(string: "http://localhost:6062/covers/" + track.albumArt), duration: TimeInterval(track.length / 1000), color: .gray.opacity(0.3)))
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
