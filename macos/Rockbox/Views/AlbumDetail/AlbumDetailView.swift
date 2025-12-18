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
    
    var hasMultipleDiscs: Bool {
        tracks.contains { $0.discNumber > 1 }
    }
    
    var tracksByDisc: [Int: [Song]] {
        Dictionary(grouping: tracks, by: { $0.discNumber })
    }
    
    var body: some View {
        ScrollView {
            VStack(spacing: 0) {
                // Album header
                AlbumHeaderView(album: album, totalDuration: totalDuration, trackCount: tracks.count, onBack: onBack)
                
                // Track list
                LazyVStack(spacing: 0) {
                    if hasMultipleDiscs {
                        ForEach(tracksByDisc.keys.sorted(), id: \.self) { discNumber in
                            // Volume header
                            HStack {
                                Text("Disc \(discNumber)")
                                    .font(.headline)
                                    .foregroundColor(.secondary)
                                Spacer()
                            }
                            .padding(.horizontal)
                            .padding(.top, discNumber == 1 ? 20 : 30)
                            .padding(.bottom, 10)
                            
                            // Tracks for this disc
                            ForEach(Array(tracksByDisc[discNumber]!.enumerated()), id: \.element.id) { index, track in
                                AlbumTrackRowView(
                                    track: track,
                                    index: track.trackNumber,
                                    isEven: index % 2 == 0,
                                    library: library
                                )
                            }
                        }
                    } else {
                        ForEach(Array(tracks.enumerated()), id: \.element.id) { index, track in
                            AlbumTrackRowView(
                                track: track,
                                index: track.trackNumber,
                                isEven: index % 2 == 0,
                                library: library
                            )
                        }
                    }
                }
                .padding(.top, hasMultipleDiscs ? 0 : 20)
                .padding(.bottom, 50)
                
                VStack(alignment: .leading, spacing: 4) {
                    Text(album.releaseDate != nil ?  formatReleaseDate(album.releaseDate.unsafelyUnwrapped): String())
                        .foregroundStyle(.secondary)
                    
                    Text(album.copyrightMessage ?? String())
                        .foregroundStyle(.secondary)
                }
                .frame(maxWidth: .infinity, alignment: .leading)
                .padding(.horizontal)
                .padding(.bottom, 30)
                .padding(.leading, 8)
            }
        }
        .task {
            do {
                let data = try await fetchAlbum(id: album.cuid)
                for track in data.tracks {
                    tracks.append(Song(
                        cuid: track.id,
                        title: track.title,
                        artist: track.artist,
                        album: track.album,
                        albumArt: URL(string: "http://localhost:6062/covers/" + track.albumArt),
                        duration: TimeInterval(track.length / 1000),
                        trackNumber: Int(track.trackNumber),
                        discNumber: Int(track.discNumber),
                        color: .gray.opacity(0.3)
                    ))
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
    
    private func formatReleaseDate(_ dateString: String?) -> String {
        guard let dateString = dateString else { return "" }
        
        let inputFormatter = DateFormatter()
        inputFormatter.dateFormat = "yyyy-MM-dd"
        
        guard let date = inputFormatter.date(from: dateString) else { return dateString }
        
        let outputFormatter = DateFormatter()
        outputFormatter.dateStyle = .long
        
        return outputFormatter.string(from: date)
    }
}
