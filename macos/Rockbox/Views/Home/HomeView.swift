//
//  HomeView.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 04/05/2026.
//

import SwiftUI

struct HomeView: View {
    @EnvironmentObject var navigation: NavigationManager
    @State private var recentAlbums: [Album] = []
    @State private var popularAlbums: [Album] = []
    @State private var artists: [Artist] = []
    @State private var savedPlaylists: [SavedPlaylist] = []
    @State private var smartPlaylists: [SmartPlaylist] = []
    @State private var loading = true
    @State private var errorText: String?

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 0) {
                Text("Home")
                    .font(.system(size: 28, weight: .bold))
                    .padding(.horizontal, 24)
                    .padding(.top, 24)
                    .padding(.bottom, 16)

                if !smartPlaylists.isEmpty || !savedPlaylists.isEmpty {
                    quickPicksGrid
                        .padding(.horizontal, 24)
                        .padding(.bottom, 24)
                }

                if !recentAlbums.isEmpty {
                    section("Recently played") {
                        albumRow(recentAlbums)
                    }
                }

                if !smartPlaylists.isEmpty {
                    section("Made for you") {
                        playlistRow(smartPlaylists.prefix(12).map { $0 }, isSmart: true)
                    }
                }

                if !artists.isEmpty {
                    section("Your top artists") {
                        artistRow(artists.prefix(12).map { $0 })
                    }
                }

                if !popularAlbums.isEmpty {
                    section("Popular albums") {
                        albumRow(popularAlbums)
                    }
                    .padding(.bottom, 30)
                }

                if !loading && recentAlbums.isEmpty && popularAlbums.isEmpty
                    && artists.isEmpty && smartPlaylists.isEmpty && savedPlaylists.isEmpty
                {
                    VStack {
                        Image(systemName: "music.note.list")
                            .font(.system(size: 48))
                            .foregroundStyle(.tertiary)
                        Text("Library is empty — wait for the daemon to finish scanning.")
                            .font(.system(size: 13))
                            .foregroundStyle(.secondary)
                            .padding(.top, 12)
                    }
                    .frame(maxWidth: .infinity)
                    .padding(.top, 60)
                }
            }
        }
        .task {
            do {
                async let tData = fetchTracks()
                async let arData = fetchArtists()
                async let spData = fetchSavedPlaylists()
                async let smData = fetchSmartPlaylists()
                let (trackData, artistData, savedData, smartData) =
                    try await (tData, arData, spData, smData)

                let (recents, populars) = aggregateAlbums(from: trackData)
                recentAlbums = recents
                popularAlbums = populars

                artists = artistData.map {
                    Artist(
                        cuid: $0.id, name: $0.name,
                        image: $0.hasImage ? $0.image : nil,
                        genre: $0.hasGenres ? $0.genres : "",
                        color: .gray.opacity(0.3)
                    )
                }
                savedPlaylists = savedData.map {
                    SavedPlaylist(
                        id: $0.id, name: $0.name,
                        description: $0.hasDescription_p ? $0.description_p : nil,
                        image: $0.hasImage ? $0.image : nil,
                        folderID: $0.hasFolderID ? $0.folderID : nil,
                        trackCount: $0.trackCount
                    )
                }
                smartPlaylists = smartData.map {
                    SmartPlaylist(
                        id: $0.id, name: $0.name,
                        description: $0.hasDescription_p ? $0.description_p : nil,
                        image: $0.hasImage ? $0.image : nil,
                        isSystem: $0.isSystem
                    )
                }
                loading = false
            } catch {
                loading = false
                errorText = String(describing: error)
            }
        }
        .alert("gRPC Error", isPresented: .constant(errorText != nil)) {
            Button("OK") { errorText = nil }
        } message: {
            Text(errorText ?? "")
        }
    }

    // Aggregates tracks into per-album rows, mirroring the GPUI client:
    // recently played → max track id desc (cuids are time-ordered);
    // popular        → track count desc, name asc.
    private func aggregateAlbums(
        from tracks: [Rockbox_V1alpha1_Track]
    ) -> (recent: [Album], popular: [Album]) {
        struct Agg {
            var albumName: String
            var artist: String
            var artistID: String
            var art: String
            var year: UInt32
            var trackCount: Int
            var maxId: String
        }
        var aggMap: [String: Agg] = [:]
        for t in tracks {
            if t.album.isEmpty { continue }
            let albumKey = t.hasAlbumID && !t.albumID.isEmpty ? t.albumID : t.album
            let displayArtist = t.albumArtist.isEmpty ? t.artist : t.albumArtist
            var entry = aggMap[albumKey] ?? Agg(
                albumName: t.album, artist: displayArtist, artistID: t.artistID,
                art: "", year: t.year, trackCount: 0, maxId: ""
            )
            entry.trackCount += 1
            if entry.art.isEmpty, t.hasAlbumArt, !t.albumArt.isEmpty {
                entry.art = t.albumArt
            }
            if t.id > entry.maxId { entry.maxId = t.id }
            aggMap[albumKey] = entry
        }

        let baseURL = ServerConfig.shared.coversBaseURL
        func makeAlbum(_ id: String, _ a: Agg) -> Album {
            Album(
                cuid: id, title: a.albumName, artist: a.artist,
                year: Int(a.year), color: .gray.opacity(0.3),
                cover: baseURL + a.art,
                releaseDate: nil, copyrightMessage: nil,
                artistID: a.artistID, tracks: []
            )
        }

        let recent = aggMap
            .sorted { $0.value.maxId > $1.value.maxId }
            .prefix(8)
            .map { makeAlbum($0.key, $0.value) }
        let popular = aggMap
            .sorted {
                $0.value.trackCount != $1.value.trackCount
                    ? $0.value.trackCount > $1.value.trackCount
                    : $0.value.albumName < $1.value.albumName
            }
            .prefix(12)
            .map { makeAlbum($0.key, $0.value) }
        return (recent, popular)
    }

    @ViewBuilder
    private func section<Content: View>(
        _ title: String, @ViewBuilder content: () -> Content
    ) -> some View {
        Text(title)
            .font(.system(size: 18, weight: .semibold))
            .padding(.horizontal, 24)
            .padding(.top, 12)
            .padding(.bottom, 8)
        content()
    }

    private var quickPicksGrid: some View {
        let picks = (smartPlaylists.map { (id: $0.id, name: $0.name, isSmart: true) }
            + savedPlaylists.map { (id: $0.id, name: $0.name, isSmart: false) })
            .prefix(6)
        return LazyVGrid(
            columns: [GridItem(.flexible()), GridItem(.flexible())],
            spacing: 8
        ) {
            ForEach(Array(picks), id: \.id) { item in
                Button(action: {
                    if item.isSmart, let p = smartPlaylists.first(where: { $0.id == item.id }) {
                        navigation.goToSmartPlaylist(p)
                    } else if let p = savedPlaylists.first(where: { $0.id == item.id }) {
                        navigation.goToPlaylist(p)
                    }
                }) {
                    HStack(spacing: 10) {
                        ZStack {
                            Color.secondary.opacity(0.18)
                            Image(systemName: "music.note")
                                .foregroundStyle(.secondary)
                        }
                        .frame(width: 48, height: 48)
                        .clipShape(RoundedRectangle(cornerRadius: 4))

                        Text(item.name)
                            .font(.system(size: 13, weight: .semibold))
                            .lineLimit(2)
                        Spacer()
                    }
                    .padding(8)
                    .background(Color.secondary.opacity(0.12))
                    .clipShape(RoundedRectangle(cornerRadius: 6))
                }
                .buttonStyle(.plain)
            }
        }
    }

    private func albumRow(_ items: [Album]) -> some View {
        ScrollView(.horizontal, showsIndicators: false) {
            HStack(alignment: .top, spacing: 16) {
                ForEach(items) { album in
                    HomeAlbumCell(album: album, onTap: { navigation.goToAlbum(album) })
                }
            }
            .padding(.horizontal, 24)
        }
    }

    private func playlistRow(_ items: [SmartPlaylist], isSmart: Bool) -> some View {
        ScrollView(.horizontal, showsIndicators: false) {
            HStack(alignment: .top, spacing: 16) {
                ForEach(items) { playlist in
                    Button(action: { navigation.goToSmartPlaylist(playlist) }) {
                        VStack(alignment: .leading, spacing: 6) {
                            ZStack {
                                LinearGradient(
                                    colors: [
                                        Genre.colorForSeed(playlist.id),
                                        Genre.colorForSeed(playlist.id).opacity(0.5),
                                    ], startPoint: .topLeading, endPoint: .bottomTrailing
                                )
                                Image(systemName: "bolt.fill")
                                    .font(.system(size: 28))
                                    .foregroundStyle(.white)
                            }
                            .frame(width: 160, height: 160)
                            .clipShape(RoundedRectangle(cornerRadius: 4))
                            Text(playlist.name)
                                .font(.system(size: 13, weight: .semibold))
                                .lineLimit(1)
                                .frame(maxWidth: 160, alignment: .leading)
                            if let description = playlist.description {
                                Text(description)
                                    .font(.system(size: 12))
                                    .foregroundStyle(.secondary)
                                    .lineLimit(2)
                                    .frame(maxWidth: 160, alignment: .leading)
                            }
                        }
                    }
                    .buttonStyle(.plain)
                }
            }
            .padding(.horizontal, 24)
        }
    }

    private func artistRow(_ items: [Artist]) -> some View {
        ScrollView(.horizontal, showsIndicators: false) {
            HStack(alignment: .top, spacing: 16) {
                ForEach(items) { artist in
                    HomeArtistCell(artist: artist, onTap: { navigation.goToArtist(artist) })
                }
            }
            .padding(.horizontal, 24)
        }
    }
}

private struct HomeAlbumCell: View {
    let album: Album
    let onTap: () -> Void

    var body: some View {
        Button(action: onTap) {
            VStack(alignment: .leading, spacing: 6) {
                CachedAsyncImage(url: URL(string: album.cover)) { phase in
                    switch phase {
                    case .success(let image):
                        image.resizable().scaledToFill()
                    default:
                        placeholder
                    }
                }
                .frame(width: 160, height: 160)
                .clipShape(RoundedRectangle(cornerRadius: 4))
                Text(album.title)
                    .font(.system(size: 13, weight: .semibold))
                    .lineLimit(1)
                    .frame(maxWidth: 160, alignment: .leading)
                Text(album.artist)
                    .font(.system(size: 12))
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
                    .frame(maxWidth: 160, alignment: .leading)
            }
        }
        .buttonStyle(.plain)
    }

    private var placeholder: some View {
        ZStack {
            Color.secondary.opacity(0.18)
            Image(systemName: "music.note")
                .font(.system(size: 32))
                .foregroundStyle(.secondary)
        }
    }
}

private struct HomeArtistCell: View {
    let artist: Artist
    let onTap: () -> Void

    var body: some View {
        Button(action: onTap) {
            VStack(spacing: 8) {
                avatar
                    .frame(width: 130, height: 130)
                    .clipShape(Circle())
                Text(artist.name)
                    .font(.system(size: 13, weight: .semibold))
                    .lineLimit(1)
                    .frame(width: 140)
                Text("Artist")
                    .font(.system(size: 12))
                    .foregroundStyle(.secondary)
            }
        }
        .buttonStyle(.plain)
    }

    @ViewBuilder
    private var avatar: some View {
        if let imageURL = artist.image, let url = URL(string: imageURL) {
            CachedAsyncImage(url: url) { phase in
                switch phase {
                case .success(let image):
                    image.resizable().scaledToFill()
                default:
                    placeholder
                }
            }
        } else {
            placeholder
        }
    }

    private var placeholder: some View {
        ZStack {
            Circle().fill(Color.gray.opacity(0.25))
            Image(systemName: "music.mic")
                .font(.system(size: 36))
                .foregroundStyle(.secondary)
        }
    }
}
