//
//  ContentView.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 13/12/2025.
//

import SwiftUI

struct ContentView: View {
    @State private var selection: SidebarItem? = .albums
    @StateObject private var player = PlayerState()
    
    var body: some View {
        NavigationSplitView {
            Sidebar(selection: $selection)
        } detail: {
            DetailView(selection: selection, player: player)
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

struct Track {
    let title: String
    let artist: String
    let album: String
    let artworkColor: Color
}


struct Sidebar: View {
    @Binding var selection: SidebarItem?
    
    var body: some View {
        List(selection: $selection) {
            Section("Library") {
                ForEach(SidebarItem.allCases) { item in
                    Label(item.rawValue, systemImage: item.icon)
                        .tag(item)
                }
            }
        }
        .listStyle(.sidebar)
    }
}

struct DetailView: View {
    let selection: SidebarItem?
    @ObservedObject var player: PlayerState

    var title: String { selection?.rawValue ?? "Albums" }

    var body: some View {
        VStack(spacing: 0) {

            Group {
                if let selection {
                    Text(selection.rawValue)
                        .font(.largeTitle)
                        .frame(maxWidth: .infinity, maxHeight: .infinity)
                } else {
                    Text("Select an item")
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity, maxHeight: .infinity)
                }
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)

            Divider()
            PlayerControlsView(player: player)
        }
        .background(Color.white)
    }
}
struct PlayerControlsView: View {
    @ObservedObject var player: PlayerState
    @State private var isHoveringProgress = false
    
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
                    // Track metadata
                    VStack(alignment: .center, spacing: 1) {
                        Text(player.currentTrack.title)
                            .font(.system(size: 12, weight: .medium))
                            .lineLimit(1)
                        Text("\(player.currentTrack.artist) — \(player.currentTrack.album)")
                            .font(.system(size: 10))
                            .foregroundStyle(.secondary)
                            .lineLimit(1)
                    }
                    .frame(maxWidth: .infinity)
                    
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

