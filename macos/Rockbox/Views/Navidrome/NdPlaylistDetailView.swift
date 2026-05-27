import SwiftUI

struct NdPlaylistDetailView: View {
    let playlist: NdPlaylist
    var onBack: () -> Void

    @State private var songs: [NdSong] = []
    @State private var isLoading = false
    @State private var errorText: String?
    @ObservedObject private var ndManager = NavidromeManager.shared
    @EnvironmentObject var player: PlayerState

    private var server: NdServer? { ndManager.activeServer }

    var body: some View {
        ScrollView {
            VStack(spacing: 0) {
                playlistHeader

                LazyVStack(spacing: 0) {
                    ForEach(Array(songs.enumerated()), id: \.element.id) { idx, song in
                        if let server {
                            NdSongRowView(
                                song: song,
                                index: idx + 1,
                                isEven: idx % 2 == 0,
                                server: server,
                                allSongs: songs,
                                showArt: false
                            )
                        }
                    }
                }
                .padding(.top, 16)
                .padding(.bottom, 40)
            }
        }
        .task(id: playlist.id) {
            guard let server else { return }
            isLoading = true
            defer { isLoading = false }
            if let detail = try? await ndGetPlaylist(server: server, playlistId: playlist.id) {
                songs = detail.songs
            }
        }
        .alert("Error", isPresented: .constant(errorText != nil)) {
            Button("OK") { errorText = nil }
        } message: {
            Text(errorText ?? "")
        }
    }

    private var playlistHeader: some View {
        HStack(alignment: .top, spacing: 24) {
            VStack(alignment: .leading, spacing: 12) {
                Button(action: onBack) {
                    HStack(spacing: 4) {
                        Image(systemName: "chevron.left").font(.system(size: 14, weight: .semibold))
                        Text("Back").font(.system(size: 13))
                    }
                    .foregroundStyle(.secondary)
                    .frame(minHeight: 34)
                    .contentShape(Rectangle())
                }
                .buttonStyle(.plain)

                RoundedRectangle(cornerRadius: 4)
                    .fill(Color.purple.opacity(0.2).gradient)
                    .frame(width: 240, height: 240)
                    .overlay {
                        if let coverId = playlist.coverArt, let url = server?.coverArtUrl(coverId: coverId, size: 500) {
                            CachedAsyncImage(url: url) { phase in
                                switch phase {
                                case .success(let img):
                                    img.resizable().aspectRatio(contentMode: .fill)
                                default:
                                    Image(systemName: "music.note.list")
                                        .font(.system(size: 60))
                                        .foregroundStyle(.white.opacity(0.6))
                                }
                            }
                        } else {
                            Image(systemName: "music.note.list")
                                .font(.system(size: 60))
                                .foregroundStyle(.white.opacity(0.6))
                        }
                    }
                    .clipShape(RoundedRectangle(cornerRadius: 4))
            }

            VStack(alignment: .leading, spacing: 8) {
                Text(playlist.name)
                    .font(.system(size: 24, weight: .bold))
                    .padding(.top, 15)

                if let comment = playlist.comment, !comment.isEmpty {
                    Text(comment)
                        .font(.system(size: 14))
                        .foregroundStyle(.secondary)
                }

                Text("\(playlist.songCount) songs")
                    .font(.system(size: 13))
                    .foregroundStyle(.tertiary)
                    .padding(.top, 4)

                Spacer()

                HStack(spacing: 12) {
                    Button(action: playPlaylist) {
                        HStack(spacing: 6) {
                            Image(systemName: "play.fill").font(.system(size: 12))
                            Text("Play").font(.system(size: 13, weight: .medium))
                        }
                        .padding(.horizontal, 20)
                        .padding(.vertical, 8)
                        .background(Color(hex: "fe09a3"))
                        .foregroundStyle(.white)
                        .cornerRadius(20)
                    }
                    .buttonStyle(.plain)

                    Button(action: shufflePlaylist) {
                        HStack(spacing: 6) {
                            Image(systemName: "shuffle").font(.system(size: 12))
                            Text("Shuffle").font(.system(size: 13, weight: .medium))
                        }
                        .padding(.horizontal, 20)
                        .padding(.vertical, 8)
                        .background(Color.black.opacity(0.05))
                        .foregroundStyle(.primary)
                        .cornerRadius(20)
                    }
                    .buttonStyle(.plain)
                }
                .disabled(songs.isEmpty)
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(.top, 28)
        }
        .padding(24)
    }

    private func playPlaylist() {
        let paths = songs.map { $0.streamUrl }
        Task {
            do {
                try await clearPlaylist()
                try await insertTracks(tracks: paths, position: 0)
                try await play(elapsed: 0)
                await player.fetchQueue()
            } catch { errorText = String(describing: error) }
        }
    }

    private func shufflePlaylist() {
        let paths = songs.shuffled().map { $0.streamUrl }
        Task {
            do {
                try await clearPlaylist()
                try await insertTracks(tracks: paths, position: 0)
                try await play(elapsed: 0)
                await player.fetchQueue()
            } catch { errorText = String(describing: error) }
        }
    }
}
