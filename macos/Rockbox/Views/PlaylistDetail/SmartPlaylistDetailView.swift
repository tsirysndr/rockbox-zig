//
//  SmartPlaylistDetailView.swift
//  Rockbox
//

import SwiftUI

struct SmartPlaylistDetailView: View {
    let playlist: SmartPlaylist
    @ObservedObject var library: MusicLibrary
    var onBack: () -> Void

    @EnvironmentObject var player: PlayerState
    @State private var tracks: [Song] = []
    @State private var errorText: String?

    var totalDuration: TimeInterval {
        tracks.reduce(0) { $0 + $1.duration }
    }

    var body: some View {
        ScrollView {
            VStack(spacing: 0) {
                // Header
                ZStack(alignment: .topLeading) {
                    LinearGradient(
                        colors: [Color.purple.opacity(0.35), Color.clear],
                        startPoint: .top, endPoint: .bottom
                    )
                    .frame(height: 220)

                    VStack(alignment: .leading, spacing: 0) {
                        Button(action: onBack) {
                            HStack(spacing: 4) {
                                Image(systemName: "chevron.left")
                                Text("Back")
                            }
                            .font(.system(size: 13))
                            .foregroundStyle(.primary)
                        }
                        .buttonStyle(.plain)
                        .padding(.top, 16)
                        .padding(.horizontal, 24)

                        HStack(alignment: .bottom, spacing: 20) {
                            RoundedRectangle(cornerRadius: 8)
                                .fill(Color.purple.opacity(0.25).gradient)
                                .frame(width: 120, height: 120)
                                .overlay {
                                    Image(systemName: playlist.isSystem ? "sparkles" : "wand.and.stars")
                                        .font(.system(size: 40))
                                        .foregroundStyle(.white.opacity(0.7))
                                }
                                .clipShape(RoundedRectangle(cornerRadius: 8))

                            VStack(alignment: .leading, spacing: 6) {
                                Text(playlist.isSystem ? "SMART PLAYLIST · SYSTEM" : "SMART PLAYLIST")
                                    .font(.system(size: 11, weight: .semibold))
                                    .foregroundStyle(.secondary)
                                Text(playlist.name)
                                    .font(.system(size: 24, weight: .bold))
                                    .lineLimit(2)
                                if let desc = playlist.description {
                                    Text(desc)
                                        .font(.system(size: 12))
                                        .foregroundStyle(.secondary)
                                        .lineLimit(2)
                                }
                                Text("\(tracks.count) tracks · \(formatDuration(totalDuration))")
                                    .font(.system(size: 12))
                                    .foregroundStyle(.secondary)
                            }
                            Spacer()
                        }
                        .padding(.horizontal, 24)
                        .padding(.top, 12)

                        HStack(spacing: 12) {
                            Button(action: {
                                Task {
                                    do {
                                        try await playSmartPlaylist(id: playlist.id)
                                        await player.fetchQueue()
                                    } catch {
                                        errorText = String(describing: error)
                                    }
                                }
                            }) {
                                Label("Play", systemImage: "play.fill")
                                    .font(.system(size: 13, weight: .semibold))
                                    .padding(.horizontal, 20)
                                    .padding(.vertical, 8)
                                    .background(Color.purple.opacity(0.8))
                                    .foregroundStyle(.white)
                                    .clipShape(Capsule())
                            }
                            .buttonStyle(.plain)
                        }
                        .padding(.horizontal, 24)
                        .padding(.top, 16)
                        .padding(.bottom, 20)
                    }
                }

                SongHeaderRow()
                Divider()

                LazyVStack(spacing: 0) {
                    ForEach(Array(tracks.enumerated()), id: \.element.cuid) { index, song in
                        SongRowView(
                            song: song,
                            index: index + 1,
                            isEven: index % 2 == 0,
                            showLike: true,
                            library: library
                        )
                    }
                }
            }
        }
        .task {
            await loadTracks()
        }
        .alert("Error", isPresented: .constant(errorText != nil)) {
            Button("OK") { errorText = nil }
        } message: {
            Text(errorText ?? "")
        }
    }

    private func loadTracks() async {
        do {
            let trackIDs = try await fetchSmartPlaylistTracks(id: playlist.id)
            var loaded: [Song] = []
            for id in trackIDs {
                if let track = try? await fetchTrack(id: id) {
                    loaded.append(Song(
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
                    ))
                }
            }
            tracks = loaded
        } catch {
            errorText = String(describing: error)
        }
    }

    private func formatDuration(_ d: TimeInterval) -> String {
        let totalMin = Int(d) / 60
        let h = totalMin / 60
        let m = totalMin % 60
        return h > 0 ? "\(h) hr \(m) min" : "\(m) min"
    }
}
