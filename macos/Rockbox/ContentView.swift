//
//  ContentView.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 13/12/2025.
//

import SwiftUI
import AppKit

extension Color {
    init(hex: String) {
        let hex = hex.trimmingCharacters(in: CharacterSet.alphanumerics.inverted)
        var int: UInt64 = 0
        Scanner(string: hex).scanHexInt64(&int)
        let a, r, g, b: UInt64
        switch hex.count {
        case 6: // RGB
            (a, r, g, b) = (255, int >> 16, int >> 8 & 0xFF, int & 0xFF)
        case 8: // ARGB
            (a, r, g, b) = (int >> 24, int >> 16 & 0xFF, int >> 8 & 0xFF, int & 0xFF)
        default:
            (a, r, g, b) = (255, 0, 0, 0)
        }
        self.init(
            .sRGB,
            red: Double(r) / 255,
            green: Double(g) / 255,
            blue: Double(b) / 255,
            opacity: Double(a) / 255
        )
    }
}

struct VisualEffectView: NSViewRepresentable {
    var material: NSVisualEffectView.Material = .sidebar
    var blendingMode: NSVisualEffectView.BlendingMode = .behindWindow
    var state: NSVisualEffectView.State = .active

    func makeNSView(context: Context) -> NSVisualEffectView {
        let v = NSVisualEffectView()
        v.material = material
        v.blendingMode = blendingMode
        v.state = state
        return v
    }

    func updateNSView(_ nsView: NSVisualEffectView, context: Context) {
        nsView.material = material
        nsView.blendingMode = blendingMode
        nsView.state = state
    }
}

struct ContentView: View {
    @State private var selection: SidebarItem? = .albums
    @StateObject private var player = PlayerState()
    @StateObject private var library = MusicLibrary()

    
    var body: some View {
        NavigationSplitView {
            Sidebar(selection: $selection)
        } detail: {
            DetailView(selection: selection, player: player, library: library)
        }
    }
}

enum SidebarItem: String, CaseIterable, Identifiable {
    case albums = "Albums"
    case artists = "Artists"
    case songs = "Songs"
    case likes = "Likes"
    case files = "Files"
    
    var id: String { rawValue }
    
    var icon: String {
        switch self {
        case .albums: return "square.stack"
        case .artists: return "music.mic"
        case .songs: return "music.note"
        case .likes: return "heart"
        case .files: return "folder"
        }
    }
}

struct Artist: Identifiable {
    let id = UUID()
    let name: String
    let genre: String
    let color: Color
}

struct Song: Identifiable {
    let id = UUID()
    let title: String
    let artist: String
    let album: String
    let duration: TimeInterval
    let color: Color
}

struct Track {
    let title: String
    let artist: String
    let album: String
    let artworkColor: Color
}

struct Album: Identifiable {
    let id = UUID()
    let title: String
    let artist: String
    let year: Int
    let color: Color
}

class MusicLibrary: ObservableObject {
    @Published var likedSongIds: Set<UUID> = []
    
    func isLiked(_ song: Song) -> Bool {
        likedSongIds.contains(song.id)
    }
    
    func toggleLike(_ song: Song) {
        if likedSongIds.contains(song.id) {
            likedSongIds.remove(song.id)
        } else {
            likedSongIds.insert(song.id)
        }
    }
    
    func likedSongs(from songs: [Song]) -> [Song] {
        songs.filter { likedSongIds.contains($0.id) }
    }
}

enum FileItemType {
    case directory
    case audioFile
}

struct FileItem: Identifiable {
    let id = UUID()
    let name: String
    let type: FileItemType
    let size: String?
    let itemCount: Int?
    
    var icon: String {
        switch type {
        case .directory:
            return "folder.fill"
        case .audioFile:
            return "music.note"
        }
    }
    
    var iconColor: Color {
        switch type {
        case .directory:
            return .blue
        case .audioFile:
            return .pink
        }
    }
}

let sampleFiles: [FileItem] = [
    FileItem(name: "Music", type: .directory, size: nil, itemCount: 245),
    FileItem(name: "Downloads", type: .directory, size: nil, itemCount: 18),
    FileItem(name: "Playlists", type: .directory, size: nil, itemCount: 12),
    FileItem(name: "Podcasts", type: .directory, size: nil, itemCount: 34),
    FileItem(name: "Recordings", type: .directory, size: nil, itemCount: 8),
    FileItem(name: "Bohemian Rhapsody.mp3", type: .audioFile, size: "8.2 MB", itemCount: nil),
    FileItem(name: "Hotel California.flac", type: .audioFile, size: "42.1 MB", itemCount: nil),
    FileItem(name: "Stairway to Heaven.mp3", type: .audioFile, size: "12.4 MB", itemCount: nil),
    FileItem(name: "Billie Jean.m4a", type: .audioFile, size: "6.8 MB", itemCount: nil),
    FileItem(name: "Purple Rain.mp3", type: .audioFile, size: "9.1 MB", itemCount: nil),
    FileItem(name: "Smells Like Teen Spirit.mp3", type: .audioFile, size: "7.3 MB", itemCount: nil),
    FileItem(name: "The Chain.flac", type: .audioFile, size: "38.5 MB", itemCount: nil),
    FileItem(name: "Come Together.mp3", type: .audioFile, size: "6.2 MB", itemCount: nil),
]

let sampleAlbums: [Album] = [
    Album(title: "A Night at the Opera", artist: "Queen", year: 1975, color: .purple),
    Album(title: "Rumours", artist: "Fleetwood Mac", year: 1977, color: .blue),
    Album(title: "Back in Black", artist: "AC/DC", year: 1980, color: .black),
    Album(title: "Thriller", artist: "Michael Jackson", year: 1982, color: .orange),
    Album(title: "The Dark Side of the Moon", artist: "Pink Floyd", year: 1973, color: .indigo),
    Album(title: "Abbey Road", artist: "The Beatles", year: 1969, color: .cyan),
    Album(title: "Led Zeppelin IV", artist: "Led Zeppelin", year: 1971, color: .brown),
    Album(title: "Hotel California", artist: "Eagles", year: 1976, color: .yellow),
    Album(title: "Born to Run", artist: "Bruce Springsteen", year: 1975, color: .red),
    Album(title: "Purple Rain", artist: "Prince", year: 1984, color: .purple),
    Album(title: "The Joshua Tree", artist: "U2", year: 1987, color: .gray),
    Album(title: "Nevermind", artist: "Nirvana", year: 1991, color: .teal),
]

let sampleArtists: [Artist] = [
    Artist(name: "Queen", genre: "Rock", color: .purple),
    Artist(name: "Fleetwood Mac", genre: "Rock", color: .blue),
    Artist(name: "AC/DC", genre: "Hard Rock", color: .black),
    Artist(name: "Michael Jackson", genre: "Pop", color: .orange),
    Artist(name: "Pink Floyd", genre: "Progressive Rock", color: .indigo),
    Artist(name: "The Beatles", genre: "Rock", color: .cyan),
    Artist(name: "Led Zeppelin", genre: "Hard Rock", color: .brown),
    Artist(name: "Eagles", genre: "Rock", color: .yellow),
    Artist(name: "Bruce Springsteen", genre: "Rock", color: .red),
    Artist(name: "Prince", genre: "Pop", color: .purple),
    Artist(name: "U2", genre: "Rock", color: .gray),
    Artist(name: "Nirvana", genre: "Grunge", color: .teal),
]


let sampleSongs: [Song] = [
    Song(title: "Bohemian Rhapsody", artist: "Queen", album: "A Night at the Opera", duration: 354, color: .purple),
    Song(title: "You're My Best Friend", artist: "Queen", album: "A Night at the Opera", duration: 170, color: .purple),
    Song(title: "Love of My Life", artist: "Queen", album: "A Night at the Opera", duration: 219, color: .purple),
    Song(title: "The Chain", artist: "Fleetwood Mac", album: "Rumours", duration: 271, color: .blue),
    Song(title: "Dreams", artist: "Fleetwood Mac", album: "Rumours", duration: 257, color: .blue),
    Song(title: "Go Your Own Way", artist: "Fleetwood Mac", album: "Rumours", duration: 222, color: .blue),
    Song(title: "Back in Black", artist: "AC/DC", album: "Back in Black", duration: 255, color: .black),
    Song(title: "Hells Bells", artist: "AC/DC", album: "Back in Black", duration: 312, color: .black),
    Song(title: "Thriller", artist: "Michael Jackson", album: "Thriller", duration: 357, color: .orange),
    Song(title: "Billie Jean", artist: "Michael Jackson", album: "Thriller", duration: 294, color: .orange),
    Song(title: "Beat It", artist: "Michael Jackson", album: "Thriller", duration: 258, color: .orange),
    Song(title: "Time", artist: "Pink Floyd", album: "The Dark Side of the Moon", duration: 413, color: .indigo),
    Song(title: "Money", artist: "Pink Floyd", album: "The Dark Side of the Moon", duration: 382, color: .indigo),
    Song(title: "Come Together", artist: "The Beatles", album: "Abbey Road", duration: 259, color: .cyan),
    Song(title: "Here Comes the Sun", artist: "The Beatles", album: "Abbey Road", duration: 185, color: .cyan),
    Song(title: "Stairway to Heaven", artist: "Led Zeppelin", album: "Led Zeppelin IV", duration: 482, color: .brown),
    Song(title: "Rock and Roll", artist: "Led Zeppelin", album: "Led Zeppelin IV", duration: 220, color: .brown),
    Song(title: "Hotel California", artist: "Eagles", album: "Hotel California", duration: 391, color: .yellow),
    Song(title: "Born to Run", artist: "Bruce Springsteen", album: "Born to Run", duration: 270, color: .red),
    Song(title: "Purple Rain", artist: "Prince", album: "Purple Rain", duration: 520, color: .purple),
    Song(title: "With or Without You", artist: "U2", album: "The Joshua Tree", duration: 296, color: .gray),
    Song(title: "Smells Like Teen Spirit", artist: "Nirvana", album: "Nevermind", duration: 301, color: .teal),
]

class PlayerState: ObservableObject {
    @Published var isPlaying = false
    @Published var currentTime: TimeInterval = 73
    @Published var duration: TimeInterval = 237
    @Published var currentTrack = Track(
        title: "Bohemian Rhapsody",
        artist: "Queen",
        album: "A Night at the Opera",
        artworkColor: .purple
    )
    
    var progress: Double {
        get { duration > 0 ? currentTime / duration : 0 }
        set { currentTime = newValue * duration }
    }
}

struct Sidebar: View {
    @Binding var selection: SidebarItem?
    @State private var searchText = ""
    
    var body: some View {
        ZStack {
            VisualEffectView(material: .sidebar)
                .ignoresSafeArea()

            VStack(spacing: 0) {
                // Search input
                HStack(spacing: 8) {
                    Image(systemName: "magnifyingglass")
                        .font(.system(size: 12))
                        .foregroundStyle(.secondary)
                    
                    TextField("Search", text: $searchText)
                        .textFieldStyle(.plain)
                        .font(.system(size: 13))
                    
                    if !searchText.isEmpty {
                        Button(action: { searchText = "" }) {
                            Image(systemName: "xmark.circle.fill")
                                .font(.system(size: 12))
                                .foregroundStyle(.secondary)
                        }
                        .buttonStyle(.plain)
                    }
                }
                .padding(.horizontal, 10)
                .padding(.vertical, 6)
                .background(Color.black.opacity(0.05))
                .cornerRadius(8)
                .padding(.horizontal, 12)
                .padding(.top, 12)
                .padding(.bottom, 8)
                
                List(selection: $selection) {
                    Section("Library") {
                        ForEach(SidebarItem.allCases) { item in
                            Label(item.rawValue, systemImage: item.icon)
                                .tag(item)
                        }
                    }
                }
                .listStyle(.sidebar)
                .scrollContentBackground(.hidden)
            }
        }
    }
}

struct DetailView: View {
    let selection: SidebarItem?
    @ObservedObject var player: PlayerState
    @ObservedObject var library: MusicLibrary
    
    var body: some View {
        VStack(spacing: 0) {
            // Main content area
            Group {
                if let selection {
                    switch selection {
                    case .albums:
                        AlbumsGridView()
                    case .artists:
                        ArtistsGridView()
                    case .songs:
                        SongsListView(library: library)
                    case .likes:
                        LikesListView(library: library)
                    case .files:
                        FilesListView()
                    }
                } else {
                    Text("Select an item")
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity, maxHeight: .infinity)
                }
            }
            .background(.white)
            
            Divider()
            
            // Player controls
            PlayerControlsView(player: player)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(.white)
    }
}

struct ArtistsGridView: View {
    private let columns = [
        GridItem(.adaptive(minimum: 150, maximum: 180), spacing: 20)
    ]
    
    var body: some View {
        ScrollView {
            LazyVGrid(columns: columns, spacing: 24) {
                ForEach(sampleArtists) { artist in
                    ArtistCardView(artist: artist)
                }
            }
            .padding(20)
        }
    }
}

struct ArtistCardView: View {
    let artist: Artist
    @State private var isHovering = false
    
    var body: some View {
        VStack(spacing: 8) {
            ZStack {
                Circle()
                    .fill(artist.color.gradient)
                    .aspectRatio(1, contentMode: .fit)
                    .shadow(color: .black.opacity(0.2), radius: isHovering ? 10 : 4, y: isHovering ? 6 : 2)
                
                Image(systemName: "music.mic")
                    .font(.system(size: 40))
                    .foregroundStyle(.white.opacity(0.6))
                
                // Play button on hover
                if isHovering {
                    ZStack {
                        Circle()
                            .fill(.black.opacity(0.5))
                            .frame(width: 44, height: 44)
                        
                        Image(systemName: "play.fill")
                            .font(.system(size: 18))
                            .foregroundStyle(.white)
                    }
                }
            }
            .onHover { hovering in
                withAnimation(.easeInOut(duration: 0.2)) {
                    isHovering = hovering
                }
            }
            
            VStack(spacing: 2) {
                Text(artist.name)
                    .font(.system(size: 12, weight: .medium))
                    .lineLimit(1)
                
                Text(artist.genre)
                    .font(.system(size: 11))
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
            }
        }
    }
}

struct AlbumsGridView: View {
    private let columns = [
        GridItem(.adaptive(minimum: 150, maximum: 180), spacing: 20)
    ]
    
    var body: some View {
        ScrollView {
            LazyVGrid(columns: columns, spacing: 24) {
                ForEach(sampleAlbums) { album in
                    AlbumCardView(album: album)
                }
            }
            .padding(20)
        }
    }
}

struct AlbumCardView: View {
    let album: Album
    @State private var isHovering = false
    
    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            // Album artwork
            ZStack {
                RoundedRectangle(cornerRadius: 8)
                    .fill(album.color.gradient)
                    .aspectRatio(1, contentMode: .fit)
                    .shadow(color: .black.opacity(0.2), radius: isHovering ? 10 : 4, y: isHovering ? 6 : 2)
                
                Image(systemName: "music.note")
                    .font(.system(size: 40))
                    .foregroundStyle(.white.opacity(0.6))
                
                // Play button on hover
                if isHovering {
                    ZStack {
                        Circle()
                            .fill(.black.opacity(0.5))
                            .frame(width: 44, height: 44)
                        
                        Image(systemName: "play.fill")
                            .font(.system(size: 18))
                            .foregroundStyle(.white)
                    }
                }
            }
            .onHover { hovering in
                withAnimation(.easeInOut(duration: 0.2)) {
                    isHovering = hovering
                }
            }
            
            // Album info
            VStack(alignment: .leading, spacing: 2) {
                Text(album.title)
                    .font(.system(size: 12, weight: .medium))
                    .lineLimit(1)
                
                Text("\(album.artist) · \(String(album.year))")
                    .font(.system(size: 11))
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
            }
        }
    }
}

struct LikesListView: View {
    @ObservedObject var library: MusicLibrary
    
    var likedSongs: [Song] {
        library.likedSongs(from: sampleSongs)
    }
    
    var body: some View {
        if likedSongs.isEmpty {
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
            ScrollView {
                LazyVStack(spacing: 0) {
                    // Header row
                    SongHeaderRow(showLike: true)
                    
                    Divider()
                    
                    // Liked song rows
                    ForEach(Array(likedSongs.enumerated()), id: \.element.id) { index, song in
                        SongRowView(song: song, index: index + 1, isEven: index % 2 == 0, showLike: true, library: library)
                    }
                }
            }
        }
    }
}


struct SongsListView: View {
    @ObservedObject var library: MusicLibrary
    
    var body: some View {
        ScrollView {
            LazyVStack(spacing: 0) {
                // Header row
                SongHeaderRow(showLike: true)
                
                Divider()
                
                // Song rows
                ForEach(Array(sampleSongs.enumerated()), id: \.element.id) { index, song in
                    SongRowView(song: song, index: index + 1, isEven: index % 2 == 0, showLike: true, library: library)
                }
            }
        }
    }
}

struct SongHeaderRow: View {
    var showLike: Bool = false
    
    var body: some View {
        HStack(spacing: 12) {
            Text("#")
                .frame(width: 30, alignment: .center)
            
            Text("Title")
                .frame(maxWidth: .infinity, alignment: .leading)
            
            Text("Artist")
                .frame(width: 150, alignment: .leading)
            
            Text("Album")
                .frame(width: 180, alignment: .leading)
            
            Image(systemName: "clock")
                .frame(width: 50, alignment: .center)
            
            if showLike {
                // Placeholder for heart column
                Color.clear
                    .frame(width: 40)
            }
        }
        .font(.system(size: 11, weight: .medium))
        .foregroundStyle(.secondary)
        .padding(.horizontal, 16)
        .padding(.vertical, 8)
    }
}

struct SongRowView: View {
    let song: Song
    let index: Int
    let isEven: Bool
    var showLike: Bool = false
    @ObservedObject var library: MusicLibrary
    
    @State private var isHovering = false
    
    var body: some View {
        HStack(spacing: 12) {
            // Track number or play button
            ZStack {
                Text("\(index)")
                    .opacity(isHovering ? 0 : 1)
                
                Image(systemName: "play.fill")
                    .font(.system(size: 10))
                    .opacity(isHovering ? 1 : 0)
            }
            .frame(width: 30, alignment: .center)
            
            // Title with artwork
            HStack(spacing: 10) {
                RoundedRectangle(cornerRadius: 4)
                    .fill(song.color.gradient)
                    .frame(width: 36, height: 36)
                    .overlay {
                        Image(systemName: "music.note")
                            .font(.system(size: 14))
                            .foregroundStyle(.white.opacity(0.8))
                    }
                
                Text(song.title)
                    .lineLimit(1)
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
            Text(formatDuration(song.duration))
                .foregroundStyle(.secondary)
                .frame(width: 50, alignment: .center)
            
            // Like button
            if showLike {
                Button(action: {
                    withAnimation(.easeInOut(duration: 0.2)) {
                        library.toggleLike(song)
                    }
                }) {
                    Image(systemName: library.isLiked(song) ? "heart.fill" : "heart")
                        .font(.system(size: 14))
                        .foregroundStyle(library.isLiked(song) ? Color(hex:"#fe09a3") : .secondary)
                }
                .buttonStyle(.plain)
                .frame(width: 40, alignment: .center)
            }
        }
        .font(.system(size: 12))
        .padding(.horizontal, 16)
        .padding(.vertical, 8)
        .background(isHovering ? Color.black.opacity(0.05) : (isEven ? Color.black.opacity(0.02) : Color.clear))
        .onHover { hovering in
            withAnimation(.easeInOut(duration: 0.15)) {
                isHovering = hovering
            }
        }
    }
    
    private func formatDuration(_ duration: TimeInterval) -> String {
        let minutes = Int(duration) / 60
        let seconds = Int(duration) % 60
        return String(format: "%d:%02d", minutes, seconds)
    }
}


struct FilesListView: View {
    var body: some View {
        ScrollView {
            LazyVStack(spacing: 0) {
                // Header row
                FileHeaderRow()
                
                Divider()
                
                // File rows
                ForEach(Array(sampleFiles.enumerated()), id: \.element.id) { index, file in
                    FileRowView(file: file, isEven: index % 2 == 0)
                }
            }
        }
    }
}

struct FileHeaderRow: View {
    var body: some View {
        HStack(spacing: 12) {
            Text("Name")
                .frame(maxWidth: .infinity, alignment: .leading)
            
            Text("Size")
                .frame(width: 100, alignment: .trailing)
        }
        .font(.system(size: 11, weight: .medium))
        .foregroundStyle(.secondary)
        .padding(.horizontal, 16)
        .padding(.vertical, 8)
    }
}

struct FileRowView: View {
    let file: FileItem
    let isEven: Bool
    
    @State private var isHovering = false
    
    var body: some View {
        HStack(spacing: 12) {
            // Icon and name
            HStack(spacing: 10) {
                Image(systemName: file.icon)
                    .font(.system(size: 20))
                    .foregroundStyle(file.iconColor)
                    .frame(width: 28, alignment: .center)
                
                VStack(alignment: .leading, spacing: 2) {
                    Text(file.name)
                        .font(.system(size: 12, weight: .medium))
                        .lineLimit(1)
                    
                    if file.type == .directory, let count = file.itemCount {
                        Text("\(count) items")
                            .font(.system(size: 10))
                            .foregroundStyle(.secondary)
                    }
                }
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            
            // Size or item count
            if let size = file.size {
                Text(size)
                    .font(.system(size: 11))
                    .foregroundStyle(.secondary)
                    .frame(width: 100, alignment: .trailing)
            } else {
                Color.clear
                    .frame(width: 100)
            }
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 10)
        .background(isHovering ? Color.black.opacity(0.05) : (isEven ? Color.black.opacity(0.02) : Color.clear))
        .onHover { hovering in
            withAnimation(.easeInOut(duration: 0.15)) {
                isHovering = hovering
            }
        }
    }
}

struct PlayerControlsView: View {
    @ObservedObject var player: PlayerState
    @State private var isHoveringProgress = false
    @State private var isHoveringTrackInfo = false
    @State private var isCurrentTrackLiked = false
    
    var body: some View {
        HStack(spacing: 0) {
            // Playback controls (left, but centered in its space)
            HStack( alignment: .center, spacing: 16) {
                Button(action: { /* previous */ }) {
                    Image(systemName: "backward.fill")
                        .font(.system(size: 13))
                }
                .buttonStyle(.plain)
                
                Button(action: { player.isPlaying.toggle() }) {
                    Image(systemName: player.isPlaying ? "pause.fill" : "play.fill")
                        .font(.system(size: 16))
                }
                .buttonStyle(.plain)
                
                Button(action: { /* next */ }) {
                    Image(systemName: "forward.fill")
                        .font(.system(size: 13))
                }
                .buttonStyle(.plain)
            }
            .foregroundStyle(.primary)
            .frame(maxWidth: 280)
            
            // Track info with artwork and progress (center)
            HStack(spacing: 10) {
                // Album artwork
                RoundedRectangle(cornerRadius: 4)
                    .fill(player.currentTrack.artworkColor.gradient)
                    .frame(width: 44, height: 44)
                    .overlay {
                        Image(systemName: "music.note")
                            .foregroundStyle(.white.opacity(0.8))
                    }
                
                VStack(alignment: .leading, spacing: 0) {
                    // Track metadata with heart button
                    HStack(spacing: 8) {
                        Spacer()
                        
                        VStack(alignment: .center, spacing: 1) {
                            Text(player.currentTrack.title)
                                .font(.system(size: 12, weight: .medium))
                                .lineLimit(1)
                            Text("\(player.currentTrack.artist) — \(player.currentTrack.album)")
                                .font(.system(size: 10))
                                .foregroundStyle(.secondary)
                                .lineLimit(1)
                        }
                        
                        // Heart button (shows on hover or when liked)
                        Button(action: {
                            withAnimation(.easeInOut(duration: 0.2)) {
                                isCurrentTrackLiked.toggle()
                            }
                        }) {
                            Image(systemName: isCurrentTrackLiked ? "heart.fill" : "heart")
                                .font(.system(size: 12))
                                .foregroundStyle(isCurrentTrackLiked ? Color(hex: "fe09a3") : .secondary)
                        }
                        .buttonStyle(.plain)
                        .opacity(isHoveringTrackInfo || isCurrentTrackLiked ? 1 : 0)
                        
                        Spacer()
                    }
                    
                    Spacer()
                        .frame(height: 6)
                    
                    // Progress bar with times
                    HStack(spacing: 3) {
                        Text(formatTime(player.currentTime))
                            .font(.system(size: 9, weight: .medium).monospacedDigit())
                            .foregroundStyle(.tertiary)
                            .frame(width: 32, alignment: .trailing)
                        
                        GeometryReader { geometry in
                            ZStack(alignment: .leading) {
                                // Track background
                                Capsule()
                                    .fill(.quaternary)
                                    .frame(height: isHoveringProgress ? 6 : 3)
                                
                                // Progress fill
                                Capsule()
                                    .fill(.primary.opacity(0.8))
                                    .frame(width: geometry.size.width * player.progress, height: isHoveringProgress ? 6 : 3)
                            }
                            .frame(maxHeight: .infinity)
                            .contentShape(Rectangle())
                            .gesture(
                                DragGesture(minimumDistance: 0)
                                    .onChanged { value in
                                        let progress = max(0, min(1, value.location.x / geometry.size.width))
                                        player.progress = progress
                                    }
                            )
                            .onHover { hovering in
                                withAnimation(.easeInOut(duration: 0.15)) {
                                    isHoveringProgress = hovering
                                }
                            }
                        }
                        .frame(height: 10)
                        
                        Text(formatTime(player.duration))
                            .font(.system(size: 9, weight: .medium).monospacedDigit())
                            .foregroundStyle(.tertiary)
                            .frame(width: 32, alignment: .leading)
                    }
                }
                .frame(maxWidth: 800)
            }
            .frame(maxWidth: .infinity)
            .onHover { hovering in
                withAnimation(.easeInOut(duration: 0.15)) {
                    isHoveringTrackInfo = hovering
                }
            }
            
            // Volume (right)
            HStack(spacing: 8) {
                Image(systemName: "speaker.fill")
                    .font(.system(size: 10))
                    .foregroundStyle(.secondary)
                
                Slider(value: .constant(0.7))
                    .frame(width: 80)
                
                Image(systemName: "speaker.wave.3.fill")
                    .font(.system(size: 10))
                    .foregroundStyle(.secondary)
            }
            .frame(width: 120)
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 10)
        .background(.bar)
    }
    
    private func formatTime(_ time: TimeInterval) -> String {
        let minutes = Int(time) / 60
        let seconds = Int(time) % 60
        return String(format: "%d:%02d", minutes, seconds)
    }
}

#Preview {
    ContentView()
}

