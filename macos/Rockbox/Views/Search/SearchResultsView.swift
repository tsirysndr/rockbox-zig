//
//  SearchResultsView.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 22/12/2025.
//

import SwiftUI

struct SearchResultsView: View {
    @EnvironmentObject var searchManager: SearchManager
    @EnvironmentObject var navigation: NavigationManager
    @EnvironmentObject var player: PlayerState
    @ObservedObject var library: MusicLibrary
    var playlists: [Playlist] = []
    
    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 24) {
                if searchManager.isLoading {
                    HStack {
                        Spacer()
                        ProgressView()
                        Spacer()
                    }
                    .padding(.top, 40)
                } else if searchManager.searchResults.isEmpty {
                    emptyResultsView
                } else {
                    // Artists section
                    if !searchManager.searchResults.artists.isEmpty {
                        searchSection(title: "Artists") {
                            ScrollView(.horizontal, showsIndicators: false) {
                                HStack(spacing: 16) {
                                    ForEach(searchManager.searchResults.artists) { artist in
                                        SearchArtistCard(artist: artist) {
                                            navigation.goToArtist(artist)
                                            searchManager.clear()
                                        }
                                    }
                                }
                                .padding(.horizontal, 20)
                            }
                        }
                    }
                    
                    // Albums section
                    if !searchManager.searchResults.albums.isEmpty {
                        searchSection(title: "Albums") {
                            ScrollView(.horizontal, showsIndicators: false) {
                                HStack(spacing: 16) {
                                    ForEach(searchManager.searchResults.albums) { album in
                                        SearchAlbumCard(album: album, playlists: playlists) {
                                            navigation.goToAlbum(album)
                                            searchManager.clear()
                                        }
                                    }
                                }
                                .padding(.horizontal, 20)
                            }
                        }
                    }
                    
                    // Songs section
                    if !searchManager.searchResults.songs.isEmpty {
                        searchSection(title: "Songs") {
                            LazyVStack(spacing: 0) {
                                ForEach(Array(searchManager.searchResults.songs.prefix(10).enumerated()), id: \.element.id) { index, song in
                                    SearchSongRow(
                                        song: song,
                                        index: index,
                                        library: library,
                                        playlists: playlists
                                    )
                                }
                            }
                            .padding(.horizontal, 20)
                        }
                    }
                }
            }
            .padding(.vertical, 20)
        }
    }
    
    private var emptyResultsView: some View {
        VStack(spacing: 12) {
            Image(systemName: "magnifyingglass")
                .font(.system(size: 48))
                .foregroundStyle(.tertiary)
            
            Text("No results found")
                .font(.title3)
                .foregroundStyle(.secondary)
            
            Text("Try searching for something else")
                .font(.subheadline)
                .foregroundStyle(.tertiary)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .padding(.top, 60)
    }
    
    private func searchSection<Content: View>(title: String, @ViewBuilder content: () -> Content) -> some View {
        VStack(alignment: .leading, spacing: 12) {
            Text(title)
                .font(.title2.bold())
                .padding(.horizontal, 20)
            
            content()
        }
    }
}

// MARK: - Search Artist Card

struct SearchArtistCard: View {
    let artist: Artist
    let onSelect: () -> Void
    
    @State private var isHovering = false
    @State private var isHoveringPlay = false
    @State private var errorText: String? = nil
    @EnvironmentObject var player: PlayerState
    
    var body: some View {
        VStack(spacing: 8) {
            Circle()
                .fill(artist.color.gradient)
                .frame(width: 120, height: 120)
                .overlay {
                    if let imageUrl = artist.image {
                        CachedAsyncImage(url: URL(string: imageUrl)) { phase in
                            switch phase {
                            case .success(let image):
                                image
                                    .resizable()
                                    .aspectRatio(contentMode: .fill)
                            default:
                                Image(systemName: "music.mic")
                                    .font(.system(size: 32))
                                    .foregroundStyle(.white.opacity(0.6))
                            }
                        }
                    } else {
                        Image(systemName: "music.mic")
                            .font(.system(size: 32))
                            .foregroundStyle(.white.opacity(0.6))
                    }
                }
                .overlay {
                    // Floating play button
                    if isHovering {
                        Button(action: {
                            Task {
                                do {
                                    try await playArtistTracks(artistID: artist.cuid)
                                    await player.fetchQueue()
                                } catch {
                                    errorText = String(describing: error)
                                }
                            }
                        }) {
                            Circle()
                                .fill(isHoveringPlay ? Color(hex: "fe09a3") : .white.opacity(0.3))
                                .frame(width: 40, height: 40)
                                .overlay {
                                    Image(systemName: "play.fill")
                                        .font(.system(size: 16))
                                        .foregroundStyle(.white)
                                }
                        }
                        .buttonStyle(.borderless)
                        .onHover { isHoveringPlay = $0 }
                        .transition(.scale.combined(with: .opacity))
                    }
                }
                .clipShape(Circle())
                .scaleEffect(isHovering ? 1.05 : 1.0)
                .onHover { hovering in
                    withAnimation(.easeInOut(duration: 0.2)) {
                        isHovering = hovering
                    }
                }
            
            Text(artist.name)
                .font(.system(size: 12, weight: .medium))
                .lineLimit(1)
            
            Text("Artist")
                .font(.system(size: 11))
                .foregroundStyle(.secondary)
        }
        .frame(width: 120)
        .onTapGesture {
            onSelect()
        }
        .alert("Error", isPresented: .constant(errorText != nil)) {
            Button("OK") { errorText = nil }
        } message: {
            Text(errorText ?? "")
        }
    }
}

// MARK: - Search Album Card

struct SearchAlbumCard: View {
    let album: Album
    var playlists: [Playlist] = []
    let onSelect: () -> Void
    
    @State private var isHovering = false
    @State private var isHoveringPlay = false
    @State private var isHoveringMenu = false
    @State private var showMenu = false
    @State private var errorText: String? = nil
    @EnvironmentObject var player: PlayerState
    
    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            RoundedRectangle(cornerRadius: 8)
                .fill(album.color.gradient)
                .frame(width: 140, height: 140)
                .overlay {
                    CachedAsyncImage(url: URL(string: album.cover)) { phase in
                        switch phase {
                        case .success(let image):
                            image
                                .resizable()
                                .aspectRatio(contentMode: .fill)
                        default:
                            Image(systemName: "music.note")
                                .font(.system(size: 32))
                                .foregroundStyle(.white.opacity(0.6))
                        }
                    }
                }
                .overlay(alignment: .bottom) {
                    if isHovering || showMenu {
                        HStack(spacing: 0) {
                            // Play button
                            Button(action: {
                                Task {
                                    do {
                                        try await playAlbum(albumID: album.cuid)
                                        await player.fetchQueue()
                                    } catch {
                                        errorText = String(describing: error)
                                    }
                                }
                            }) {
                                Circle()
                                    .fill(isHoveringPlay ? Color(hex: "fe09a3") : .white.opacity(0.3))
                                    .frame(width: 32, height: 32)
                                    .overlay {
                                        Image(systemName: "play.fill")
                                            .font(.system(size: 12))
                                            .foregroundStyle(.white)
                                    }
                            }
                            .buttonStyle(.borderless)
                            .onHover { isHoveringPlay = $0 }
                            .frame(maxWidth: .infinity, alignment: .center)
                            
                            // Context menu
                            ZStack {
                                Circle()
                                    .fill(isHoveringMenu || showMenu ? Color(hex: "fe09a3") : .white.opacity(0.3))
                                    .frame(width: 32, height: 32)
                                
                                Image(systemName: "ellipsis")
                                    .font(.system(size: 12, weight: .medium))
                                    .foregroundStyle(.white)
                                    .allowsHitTesting(false)
                                
                                Button(action: { showMenu.toggle() }) {
                                    Circle()
                                        .fill(Color.clear)
                                        .frame(width: 32, height: 32)
                                }
                                .buttonStyle(.borderless)
                                .onHover { isHoveringMenu = $0 }
                                .popover(isPresented: $showMenu, arrowEdge: .bottom) {
                                    ZStack {
                                        Color.white.ignoresSafeArea()
                                        
                                        VStack(alignment: .leading, spacing: 0) {
                                            MenuItemButton(title: "Play", icon: "play.fill") {
                                                showMenu = false
                                                Task {
                                                    try? await playAlbum(albumID: album.cuid)
                                                    await player.fetchQueue()
                                                }
                                            }
                                            
                                            MenuItemButton(title: "Play Shuffled", icon: "shuffle") {
                                                showMenu = false
                                                Task {
                                                    try? await playAlbum(albumID: album.cuid, shuffle: true)
                                                    await player.fetchQueue()
                                                }
                                            }
                                            
                                            Divider().padding(.vertical, 4)
                                            
                                            MenuItemButton(title: "Play Next", icon: "text.insert") {
                                                showMenu = false
                                                Task {
                                                    try? await insertAlbum(albumID: album.cuid, position: Int32(PlaylistPosition.insertFirst))
                                                    await player.fetchQueue()
                                                }
                                            }
                                            
                                            MenuItemButton(title: "Play Last", icon: "text.append") {
                                                showMenu = false
                                                Task {
                                                    try? await insertAlbum(albumID: album.cuid, position: Int32(PlaylistPosition.insertLast))
                                                    await player.fetchQueue()
                                                }
                                            }
                                            
                                            Divider().padding(.vertical, 4)
                                            
                                            MenuItemButton(
                                                title: "Add to Playlist",
                                                icon: "music.note.list",
                                                hasSubmenu: true,
                                                submenuItems: playlists,
                                                onSubmenuSelect: { playlist in
                                                    showMenu = false
                                                },
                                                onCreateNew: {
                                                    showMenu = false
                                                },
                                                action: {}
                                            )
                                        }
                                        .padding(8)
                                        .frame(width: 200)
                                    }
                                }
                            }
                            .frame(maxWidth: .infinity, alignment: .center)
                        }
                        .padding(.vertical, 10)
                        .transition(.opacity.combined(with: .move(edge: .bottom)))
                    }
                }
                .clipShape(RoundedRectangle(cornerRadius: 8))
                .onTapGesture {
                    onSelect()
                }
                .onHover { hovering in
                    withAnimation(.easeInOut(duration: 0.2)) {
                        if !showMenu {
                            isHovering = hovering
                        }
                    }
                }
                .onChange(of: showMenu) { oldValue, newValue in
                    if !newValue {
                        withAnimation(.easeInOut(duration: 0.2)) {
                            isHovering = false
                        }
                    }
                }
            
            VStack(alignment: .leading, spacing: 2) {
                Text(album.title)
                    .font(.system(size: 12, weight: .medium))
                    .lineLimit(1)
                
                Text(album.artist)
                    .font(.system(size: 11))
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
            }
        }
        .frame(width: 140)
        .alert("Error", isPresented: .constant(errorText != nil)) {
            Button("OK") { errorText = nil }
        } message: {
            Text(errorText ?? "")
        }
    }
}

// MARK: - Search Song Row

struct SearchSongRow: View {
    let song: Song
    let index: Int
    @ObservedObject var library: MusicLibrary
    var playlists: [Playlist] = []
    
    @State private var isHovering = false
    @State private var isHoveringPlay = false
    @State private var isHoveringMenu = false
    @State private var errorText: String? = nil
    @EnvironmentObject var player: PlayerState
    @EnvironmentObject var navigation: NavigationManager
    @EnvironmentObject var searchManager: SearchManager
    
    var body: some View {
        HStack(spacing: 12) {
            // Play button / index
            ZStack {
                Text("\(index + 1)")
                    .font(.system(size: 12))
                    .foregroundStyle(.secondary)
                    .opacity(isHovering ? 0 : 1)
                
                Button(action: {
                    Task {
                        do {
                            try await playTrack(path: song.path)
                            await player.fetchQueue()
                        } catch {
                            errorText = String(describing: error)
                        }
                    }
                }) {
                    Image(systemName: "play.fill")
                        .font(.system(size: 12))
                        .foregroundStyle(isHoveringPlay ? .primary : .secondary)
                }
                .buttonStyle(.plain)
                .opacity(isHovering ? 1 : 0)
                .onHover { isHoveringPlay = $0 }
            }
            .frame(width: 24)
            
            // Album art
            RoundedRectangle(cornerRadius: 4)
                .fill(song.color.gradient)
                .frame(width: 40, height: 40)
                .overlay {
                    CachedAsyncImage(url: song.albumArt) { phase in
                        switch phase {
                        case .success(let image):
                            image
                                .resizable()
                                .aspectRatio(contentMode: .fill)
                        default:
                            Image(systemName: "music.note")
                                .font(.system(size: 12))
                                .foregroundStyle(.white.opacity(0.6))
                        }
                    }
                }
                .clipShape(RoundedRectangle(cornerRadius: 4))
            
            VStack(alignment: .leading, spacing: 2) {
                Text(song.title)
                    .font(.system(size: 13))
                    .lineLimit(1)
                
                Text("\(song.artist) · \(song.album)")
                    .font(.system(size: 11))
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
            }
            
            Spacer()
            
            // Duration
            Text(formatDuration(song.duration))
                .font(.system(size: 12))
                .foregroundStyle(.secondary)
            
            // Like button
            Button(action: {
                library.toggleLike(song)
            }) {
                Image(systemName: library.isLiked(song) ? "heart.fill" : "heart")
                    .font(.system(size: 12))
                    .foregroundStyle(library.isLiked(song) ? Color(hex: "fe09a3") : .secondary)
            }
            .buttonStyle(.plain)
            .opacity(isHovering || library.isLiked(song) ? 1 : 0)
            
            // Context menu
            Menu {
                Button(action: {
                    Task {
                        do {
                            try await insertTracks(tracks: [song.path], position: Int32(PlaylistPosition.insertFirst))
                            await player.fetchQueue()
                        } catch {
                            errorText = String(describing: error)
                        }
                    }
                }) {
                    Label("Play Next", systemImage: "text.insert")
                }
                
                Button(action: {
                    Task {
                        do {
                            try await insertTracks(tracks: [song.path], position: Int32(PlaylistPosition.insertLast))
                            await player.fetchQueue()
                        } catch {
                            errorText = String(describing: error)
                        }
                    }
                }) {
                    Label("Play Last", systemImage: "text.append")
                }
                
                Divider()
                
                MenuItemButton(
                    title: "Add to Playlist",
                    icon: "music.note.list",
                    hasSubmenu: true,
                    submenuItems: playlists,
                    onSubmenuSelect: { playlist in
                        // Add to playlist
                    },
                    onCreateNew: {
                        // Create new playlist
                    },
                    action: {}
                )
                
                Divider()
                
                Button(action: {
                    library.toggleLike(song)
                }) {
                    Label(library.isLiked(song) ? "Remove from Liked" : "Add to Liked",
                          systemImage: library.isLiked(song) ? "heart.slash" : "heart")
                }
                
                Divider()
                
                Button(action: {
                    Task {
                        await navigation.goToAlbum(byId: song.albumID)
                        searchManager.clear()
                    }
                }) {
                    Label("Go to Album", systemImage: "square.stack")
                }
                
                Button(action: {
                    Task {
                        await navigation.goToArtist(byId: song.artistID)
                        searchManager.clear()
                    }
                }) {
                    Label("Go to Artist", systemImage: "music.mic")
                }
            } label: {
                Image(systemName: "ellipsis")
                    .font(.system(size: 14))
                    .foregroundStyle(isHoveringMenu ? .primary : .secondary)
                    .frame(width: 32, height: 32)
                    .contentShape(Rectangle())
            }
            .menuStyle(.borderlessButton)
            .menuIndicator(.hidden)
            .frame(width: 32)
            .opacity(isHovering ? 1 : 0)
            .onHover { isHoveringMenu = $0 }
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 8)
        .background(isHovering ? Color.secondary.opacity(0.1) : Color.clear)
        .cornerRadius(8)
        .onHover { isHovering = $0 }
        .alert("Error", isPresented: .constant(errorText != nil)) {
            Button("OK") { errorText = nil }
        } message: {
            Text(errorText ?? "")
        }
    }
    
    private func formatDuration(_ duration: TimeInterval) -> String {
        let minutes = Int(duration) / 60
        let seconds = Int(duration) % 60
        return String(format: "%d:%02d", minutes, seconds)
    }
}
