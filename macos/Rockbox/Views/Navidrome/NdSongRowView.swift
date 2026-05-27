import SwiftUI

struct NdSongRowView: View {
    let song: NdSong
    let index: Int
    let isEven: Bool
    let server: NdServer
    let allSongs: [NdSong]
    var showArt: Bool = true

    @State private var isHovering = false
    @State private var isHoveringMenu = false
    @State private var errorText: String?
    @ObservedObject private var ndManager = NavidromeManager.shared
    @EnvironmentObject var player: PlayerState
    @EnvironmentObject var navigation: NavigationManager

    var isStarred: Bool { ndManager.starredIds.contains(song.id) }

    var body: some View {
        HStack(spacing: 12) {
            // Track number / play button
            ZStack {
                Text("\(index)")
                    .opacity(isHovering ? 0 : 1)
                Button(action: playSong) {
                    Image(systemName: "play.fill")
                        .font(.system(size: 10))
                }
                .opacity(isHovering ? 1 : 0)
                .buttonStyle(.plain)
            }
            .frame(width: 30, alignment: .center)

            // Title (+ optional art)
            HStack(spacing: 10) {
                if showArt {
                    RoundedRectangle(cornerRadius: 0)
                        .fill(Color.gray.opacity(0.2).gradient)
                        .frame(width: 36, height: 36)
                        .overlay {
                            if let coverId = song.coverArt,
                               let url = server.coverArtUrl(coverId: coverId, size: 60) {
                                CachedAsyncImage(url: url) { phase in
                                    switch phase {
                                    case .success(let img):
                                        img.resizable().aspectRatio(contentMode: .fill)
                                    default:
                                        Image(systemName: "music.note")
                                            .font(.system(size: 14))
                                            .foregroundStyle(.white.opacity(0.8))
                                    }
                                }
                            } else {
                                Image(systemName: "music.note")
                                    .font(.system(size: 14))
                                    .foregroundStyle(.white.opacity(0.8))
                            }
                        }
                        .clipShape(RoundedRectangle(cornerRadius: 0))
                }
                Text(song.title).lineLimit(1)
            }
            .frame(maxWidth: .infinity, alignment: .leading)

            // Artist
            Text(song.artist)
                .foregroundStyle(.secondary)
                .lineLimit(1)
                .frame(width: 150, alignment: .leading)

            // Album
            Text(song.album)
                .foregroundStyle(.secondary)
                .lineLimit(1)
                .frame(width: 180, alignment: .leading)

            // Duration
            Text(formatDuration(TimeInterval(song.duration)))
                .foregroundStyle(.secondary)
                .frame(width: 50, alignment: .center)

            // Star button (always visible)
            Button(action: { ndManager.toggleStar(songId: song.id) }) {
                Image(systemName: isStarred ? "heart.fill" : "heart")
                    .font(.system(size: 14))
                    .foregroundStyle(isStarred ? Color(hex: "#fe09a3") : .secondary)
            }
            .buttonStyle(.plain)
            .frame(width: 40, alignment: .center)

            // Context menu (ellipsis on hover)
            Menu {
                Button(action: {
                    Task {
                        do {
                            try await insertTracks(tracks: [song.streamUrl], position: Int32(PlaylistPosition.insertFirst))
                            await player.fetchQueue()
                        } catch { errorText = String(describing: error) }
                    }
                }) {
                    Label("Play Next", systemImage: "text.insert")
                }

                Button(action: {
                    Task {
                        do {
                            try await insertTracks(tracks: [song.streamUrl], position: Int32(PlaylistPosition.insertLast))
                            await player.fetchQueue()
                        } catch { errorText = String(describing: error) }
                    }
                }) {
                    Label("Play Last", systemImage: "text.append")
                }

                Divider()

                Button(action: { ndManager.toggleStar(songId: song.id) }) {
                    Label(
                        isStarred ? "Remove from Liked" : "Add to Liked",
                        systemImage: isStarred ? "heart.slash" : "heart"
                    )
                }

                Divider()

                Button(action: {
                    Task {
                        guard !song.albumId.isEmpty,
                              let detail = try? await ndGetAlbum(server: server, albumId: song.albumId)
                        else { return }
                        navigation.goToNdAlbum(detail.album)
                    }
                }) {
                    Label("Go to Album", systemImage: "square.stack")
                }
                .disabled(song.albumId.isEmpty)

                Button(action: {
                    Task {
                        guard !song.artistId.isEmpty,
                              let detail = try? await ndGetArtist(server: server, artistId: song.artistId)
                        else { return }
                        navigation.goToNdArtist(detail.artist)
                    }
                }) {
                    Label("Go to Artist", systemImage: "music.mic")
                }
                .disabled(song.artistId.isEmpty)
            } label: {
                Image(systemName: "ellipsis")
                    .font(.system(size: 14))
                    .foregroundStyle(isHoveringMenu ? .primary : .secondary)
                    .frame(width: 32, height: 32)
                    .contentShape(Rectangle())
            }
            .menuStyle(.borderlessButton)
            .menuIndicator(.hidden)
            .frame(width: 40, alignment: .center)
            .opacity(isHovering ? 1 : 0)
            .onHover { isHoveringMenu = $0 }
        }
        .font(.system(size: 12))
        .padding(.horizontal, 16)
        .padding(.vertical, 8)
        .background(isHovering ? Color.black.opacity(0.05) : (isEven ? Color.black.opacity(0.02) : Color.clear))
        .onHover { hovering in
            withAnimation(.easeInOut(duration: 0.15)) { isHovering = hovering }
        }
        .alert("Error", isPresented: .constant(errorText != nil)) {
            Button("OK") { errorText = nil }
        } message: {
            Text(errorText ?? "")
        }
    }

    private func playSong() {
        Task {
            do {
                try await playTrack(path: song.streamUrl)
                await player.fetchQueue()
            } catch {
                errorText = String(describing: error)
            }
        }
    }

    private func formatDuration(_ duration: TimeInterval) -> String {
        let minutes = Int(duration) / 60
        let seconds = Int(duration) % 60
        return String(format: "%d:%02d", minutes, seconds)
    }
}
