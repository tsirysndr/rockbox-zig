import SwiftUI

enum NdSection: String, CaseIterable {
    case albums = "Albums"
    case artists = "Artists"
    case songs = "Songs"
    case liked = "Liked"
    case playlists = "Playlists"
}

struct NdLibraryView: View {
    var onAlbumSelected: (NdAlbum) -> Void
    var onArtistSelected: (NdArtist) -> Void
    var onPlaylistSelected: (NdPlaylist) -> Void

    @ObservedObject private var ndManager = NavidromeManager.shared
    @State private var section: NdSection = .albums
    @State private var showServers = false

    var body: some View {
        VStack(spacing: 0) {
            // Header bar: server name + manage button
            HStack {
                if let server = ndManager.activeServer {
                    Text(server.label)
                        .font(.system(size: 13, weight: .semibold))
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                } else {
                    Text("No server")
                        .font(.system(size: 13))
                        .foregroundStyle(.secondary)
                }
                Spacer()
                Button(action: { showServers = true }) {
                    Image(systemName: "server.rack")
                        .font(.system(size: 13))
                        .foregroundStyle(.secondary)
                }
                .buttonStyle(.plain)
                .help("Manage Navidrome servers")
            }
            .padding(.horizontal, 16)
            .padding(.vertical, 8)

            Divider()

            if ndManager.activeServer == nil {
                ndEmptyState
            } else {
                Picker("", selection: $section) {
                    ForEach(NdSection.allCases, id: \.self) { s in
                        Text(s.rawValue).tag(s)
                    }
                }
                .pickerStyle(.segmented)
                .padding(.horizontal, 20)
                .padding(.vertical, 10)

                Divider()

                sectionContent
            }
        }
        .sheet(isPresented: $showServers) {
            NdServerManagerView(onDismiss: { showServers = false })
        }
    }

    @ViewBuilder
    private var sectionContent: some View {
        if let server = ndManager.activeServer {
            switch section {
            case .albums:
                NdAlbumsGridView(server: server, onAlbumSelected: onAlbumSelected)
            case .artists:
                NdArtistsGridView(server: server, onArtistSelected: onArtistSelected)
            case .songs:
                NdSongsListView(server: server)
            case .liked:
                NdLikedListView(server: server)
            case .playlists:
                NdPlaylistsListView(server: server, onPlaylistSelected: onPlaylistSelected)
            }
        }
    }

    private var ndEmptyState: some View {
        VStack(spacing: 16) {
            Image(systemName: "music.note.house")
                .font(.system(size: 48))
                .foregroundStyle(.secondary)
            Text("No Navidrome server connected")
                .font(.system(size: 16, weight: .medium))
            Text("Add a Navidrome or Subsonic-compatible server to browse your music.")
                .font(.system(size: 13))
                .foregroundStyle(.secondary)
                .multilineTextAlignment(.center)
            Button("Add Server…") { showServers = true }
                .buttonStyle(.borderedProminent)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .padding(40)
    }
}

// MARK: - Albums grid

private struct NdAlbumsGridView: View {
    let server: NdServer
    var onAlbumSelected: (NdAlbum) -> Void

    @State private var albums: [NdAlbum] = []
    @State private var isLoading = false
    @State private var errorText: String?

    private let columns = [GridItem(.adaptive(minimum: 170, maximum: 230), spacing: 20)]

    var body: some View {
        ScrollView {
            LazyVGrid(columns: columns, spacing: 24) {
                ForEach(albums) { album in
                    NdAlbumCardView(album: album, server: server)
                        .onTapGesture { onAlbumSelected(album) }
                }
            }
            .padding(20)
        }
        .overlay {
            if isLoading { ProgressView() }
        }
        .task(id: server.id) {
            isLoading = true
            defer { isLoading = false }
            albums = (try? await ndGetAlbums(server: server)) ?? []
        }
    }
}

// MARK: - Artists grid

private struct NdArtistsGridView: View {
    let server: NdServer
    var onArtistSelected: (NdArtist) -> Void

    @State private var artists: [NdArtist] = []
    @State private var isLoading = false

    private let columns = [GridItem(.adaptive(minimum: 150, maximum: 180), spacing: 20)]

    var body: some View {
        ScrollView {
            LazyVGrid(columns: columns, spacing: 24) {
                ForEach(artists) { artist in
                    NdArtistCardView(artist: artist, server: server)
                        .onTapGesture { onArtistSelected(artist) }
                }
            }
            .padding(20)
        }
        .overlay {
            if isLoading { ProgressView() }
        }
        .task(id: server.id) {
            isLoading = true
            defer { isLoading = false }
            artists = (try? await ndGetArtists(server: server)) ?? []
        }
    }
}

// MARK: - Songs list

private struct NdSongsListView: View {
    let server: NdServer

    private let pageSize = 100
    @State private var songs: [NdSong] = []
    @State private var isLoading = false
    @State private var isLoadingMore = false
    @State private var hasMore = true

    var body: some View {
        ScrollView {
            LazyVStack(spacing: 0) {
                ForEach(Array(songs.enumerated()), id: \.element.id) { idx, song in
                    NdSongRowView(song: song, index: idx + 1, isEven: idx % 2 == 0, server: server, allSongs: songs)
                }
                // Infinite-scroll sentinel
                if hasMore {
                    Color.clear
                        .frame(height: 1)
                        .onAppear { loadMore() }
                    if isLoadingMore {
                        HStack {
                            Spacer()
                            ProgressView().padding(.vertical, 12)
                            Spacer()
                        }
                    }
                }
            }
        }
        .overlay {
            if isLoading { ProgressView() }
        }
        .task(id: server.id) {
            isLoading = true
            songs = []
            hasMore = true
            defer { isLoading = false }
            let page = (try? await ndGetSongs(server: server, count: pageSize, offset: 0)) ?? []
            songs = page
            hasMore = page.count == pageSize
        }
    }

    private func loadMore() {
        guard !isLoadingMore, hasMore else { return }
        isLoadingMore = true
        let offset = songs.count
        Task {
            let page = (try? await ndGetSongs(server: server, count: pageSize, offset: offset)) ?? []
            // Merge: keep existing sorted list, append new unique entries, re-sort
            let existingIds = Set(songs.map { $0.id })
            let newSongs = page.filter { !existingIds.contains($0.id) }
            songs = (songs + newSongs).sorted {
                $0.title.localizedCaseInsensitiveCompare($1.title) == .orderedAscending
            }
            hasMore = page.count == pageSize
            isLoadingMore = false
        }
    }
}

// MARK: - Liked list

private struct NdLikedListView: View {
    let server: NdServer

    @State private var songs: [NdSong] = []
    @State private var isLoading = false
    @ObservedObject private var ndManager = NavidromeManager.shared

    var body: some View {
        ScrollView {
            LazyVStack(spacing: 0) {
                ForEach(Array(songs.enumerated()), id: \.element.id) { idx, song in
                    NdSongRowView(song: song, index: idx + 1, isEven: idx % 2 == 0, server: server, allSongs: songs)
                }
            }
        }
        .overlay {
            if isLoading { ProgressView() }
        }
        .task(id: server.id) {
            isLoading = true
            defer { isLoading = false }
            songs = (try? await ndGetStarred(server: server)) ?? []
        }
        .onChange(of: ndManager.starredIds) {
            if songs.isEmpty { return }
            songs = songs.filter { ndManager.starredIds.contains($0.id) }
        }
    }
}

// MARK: - Playlists list

private struct NdPlaylistsListView: View {
    let server: NdServer
    var onPlaylistSelected: (NdPlaylist) -> Void

    @State private var playlists: [NdPlaylist] = []
    @State private var isLoading = false

    private let columns = [GridItem(.adaptive(minimum: 170, maximum: 230), spacing: 20)]

    var body: some View {
        ScrollView {
            LazyVGrid(columns: columns, spacing: 24) {
                ForEach(playlists) { pl in
                    NdPlaylistCardView(playlist: pl, server: server)
                        .onTapGesture { onPlaylistSelected(pl) }
                }
            }
            .padding(20)
        }
        .overlay {
            if isLoading { ProgressView() }
        }
        .task(id: server.id) {
            isLoading = true
            defer { isLoading = false }
            playlists = (try? await ndGetPlaylists(server: server)) ?? []
        }
    }
}

// MARK: - Card views

struct NdAlbumCardView: View {
    let album: NdAlbum
    let server: NdServer
    @State private var isHovering = false

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            RoundedRectangle(cornerRadius: 6)
                .fill(Color.gray.opacity(0.2).gradient)
                .aspectRatio(1, contentMode: .fit)
                .overlay {
                    if let coverId = album.coverArt, let url = server.coverArtUrl(coverId: coverId, size: 300) {
                        CachedAsyncImage(url: url) { phase in
                            switch phase {
                            case .success(let img):
                                img.resizable().aspectRatio(contentMode: .fill)
                            default:
                                Image(systemName: "music.note")
                                    .font(.system(size: 32))
                                    .foregroundStyle(.white.opacity(0.5))
                            }
                        }
                    } else {
                        Image(systemName: "music.note")
                            .font(.system(size: 32))
                            .foregroundStyle(.white.opacity(0.5))
                    }
                }
                .clipShape(RoundedRectangle(cornerRadius: 6))
                .scaleEffect(isHovering ? 1.02 : 1)
                .animation(.easeInOut(duration: 0.15), value: isHovering)

            Text(album.name)
                .font(.system(size: 13, weight: .semibold))
                .lineLimit(1)
            Text(album.artist)
                .font(.system(size: 11))
                .foregroundStyle(.secondary)
                .lineLimit(1)
            if let year = album.year {
                Text(String(year))
                    .font(.system(size: 11))
                    .foregroundStyle(.tertiary)
            }
        }
        .onHover { isHovering = $0 }
    }
}

struct NdArtistCardView: View {
    let artist: NdArtist
    let server: NdServer
    @State private var isHovering = false

    var body: some View {
        VStack(alignment: .center, spacing: 8) {
            Circle()
                .fill(Color.gray.opacity(0.2).gradient)
                .aspectRatio(1, contentMode: .fit)
                .overlay {
                    if let coverId = artist.coverArt, let url = server.coverArtUrl(coverId: coverId, size: 300) {
                        CachedAsyncImage(url: url) { phase in
                            switch phase {
                            case .success(let img):
                                img.resizable().aspectRatio(contentMode: .fill)
                            default:
                                Image(systemName: "music.mic")
                                    .font(.system(size: 32))
                                    .foregroundStyle(.white.opacity(0.5))
                            }
                        }
                    } else {
                        Image(systemName: "music.mic")
                            .font(.system(size: 32))
                            .foregroundStyle(.white.opacity(0.5))
                    }
                }
                .clipShape(Circle())
                .scaleEffect(isHovering ? 1.02 : 1)
                .animation(.easeInOut(duration: 0.15), value: isHovering)

            Text(artist.name)
                .font(.system(size: 13, weight: .semibold))
                .lineLimit(1)
                .multilineTextAlignment(.center)
        }
        .onHover { isHovering = $0 }
    }
}

struct NdPlaylistCardView: View {
    let playlist: NdPlaylist
    let server: NdServer
    @State private var isHovering = false

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            RoundedRectangle(cornerRadius: 6)
                .fill(Color.purple.opacity(0.2).gradient)
                .aspectRatio(1, contentMode: .fit)
                .overlay {
                    if let coverId = playlist.coverArt, let url = server.coverArtUrl(coverId: coverId, size: 300) {
                        CachedAsyncImage(url: url) { phase in
                            switch phase {
                            case .success(let img):
                                img.resizable().aspectRatio(contentMode: .fill)
                            default:
                                Image(systemName: "music.note.list")
                                    .font(.system(size: 32))
                                    .foregroundStyle(.white.opacity(0.5))
                            }
                        }
                    } else {
                        Image(systemName: "music.note.list")
                            .font(.system(size: 32))
                            .foregroundStyle(.white.opacity(0.5))
                    }
                }
                .clipShape(RoundedRectangle(cornerRadius: 6))
                .scaleEffect(isHovering ? 1.02 : 1)
                .animation(.easeInOut(duration: 0.15), value: isHovering)

            Text(playlist.name)
                .font(.system(size: 13, weight: .semibold))
                .lineLimit(1)
            Text("\(playlist.songCount) songs")
                .font(.system(size: 11))
                .foregroundStyle(.secondary)
        }
        .onHover { isHovering = $0 }
    }
}
