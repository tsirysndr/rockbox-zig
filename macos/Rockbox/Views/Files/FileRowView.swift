//
//  FileRowView.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 14/12/2025.
//

import SwiftUI

struct FileRowView: View {
    let file: FileItem
    let isEven: Bool
    let selectedIndex: Int
    let currentDirectory: String
    
    @State private var isHovering = false
    @State private var isHoveringMenu = false
    @State private var errorText: String? = nil
    @EnvironmentObject var player: PlayerState
    
    var body: some View {
        HStack(spacing: 12) {
            // Icon and name
            HStack(spacing: 10) {
                fileIconView
                
                Text(file.name)
                    .font(.system(size: 12, weight: .medium))
                    .lineLimit(1)
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            
            contextMenuButton
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 10)
        .background(rowBackground)
        .onHover { hovering in
            withAnimation(.easeInOut(duration: 0.15)) {
                isHovering = hovering
            }
        }
        .alert("Error", isPresented: .constant(errorText != nil)) {
            Button("OK") { errorText = nil }
        } message: {
            Text(errorText ?? "")
        }
    }
    
    // MARK: - Subviews
    
    private var fileIconView: some View {
        ZStack {
            Image(systemName: file.icon)
                .font(.system(size: 20))
                .foregroundStyle(file.iconColor)
                .opacity(isHovering ? 0 : 1)
            
            Button(action: playFile) {
                Image(systemName: "play.fill")
                    .font(.system(size: 15))
            }
            .buttonStyle(.plain)
            .opacity(isHovering ? 1 : 0)
        }
        .frame(width: 30, alignment: .center)
        .foregroundStyle(.secondary)
    }
    
    private var contextMenuButton: some View {
        Menu {
            playNextButton
            addToPlaylistButton
            playLastButton
            
            Divider()
            
            addShuffledButton
            
            if file.type == .directory {
                playLastShuffledButton
                playShuffledButton
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
        .frame(width: 40, alignment: .center)
        .opacity(isHovering ? 1 : 0)
        .onHover { isHoveringMenu = $0 }
    }
    
    private var rowBackground: Color {
        if isHovering {
            return Color.black.opacity(0.05)
        } else if isEven {
            return Color.black.opacity(0.02)
        }
        return Color.clear
    }
    
    // MARK: - Menu Buttons
    
    private var playNextButton: some View {
        Button(action: {
            Task {
                do {
                    if file.type == .audioFile {
                        try await insertTracks(tracks: [file.path], position: Int32(PlaylistPosition.insertFirst))
                    } else {
                        try await insertDirectory(directory: file.path, position: Int32(PlaylistPosition.insertFirst))
                    }
                    await player.fetchQueue()
                } catch {
                    errorText = String(describing: error)
                }
            }
        }) {
            Label("Play Next", systemImage: "text.insert")
        }
    }
    
    private var addToPlaylistButton: some View {
        Button(action: {
            // Add to Playlist
        }) {
            Label("Add to Playlist", systemImage: "text.append")
        }
    }
    
    private var playLastButton: some View {
        Button(action: {
            Task {
                do {
                    if file.type == .audioFile {
                        try await insertTracks(tracks: [file.path], position: Int32(PlaylistPosition.insertLast))
                    } else {
                        try await insertDirectory(directory: file.path, position: Int32(PlaylistPosition.insertLast))
                    }
                    await player.fetchQueue()
                } catch {
                    errorText = String(describing: error)
                }
            }
        }) {
            Label("Play Last", systemImage: "text.append")
        }
    }
    
    private var addShuffledButton: some View {
        Button(action: {
            Task {
                do {
                    if file.type == .audioFile {
                        try await insertTracks(tracks: [file.path], position: Int32(PlaylistPosition.insertShuffled))
                    } else {
                        try await insertDirectory(directory: file.path, position: Int32(PlaylistPosition.insertShuffled))
                    }
                    await player.fetchQueue()
                } catch {
                    errorText = String(describing: error)
                }
            }
        }) {
            Label("Add Shuffled", systemImage: "shuffle")
        }
    }
    
    private var playLastShuffledButton: some View {
        Button(action: {
            Task {
                do {
                    try await insertDirectory(directory: file.path, position: Int32(PlaylistPosition.insertLastShuffled))
                    await player.fetchQueue()
                } catch {
                    errorText = String(describing: error)
                }
            }
        }) {
            Label("Play Last Shuffled", systemImage: "shuffle")
        }
    }
    
    private var playShuffledButton: some View {
        Button(action: {
            Task {
                do {
                    try await playDirectory(path: file.path, shuffle: true)
                    await player.fetchQueue()
                } catch {
                    errorText = String(describing: error)
                }
            }
        }) {
            Label("Play Shuffled", systemImage: "shuffle")
        }
    }
    
    // MARK: - Actions
    
    private func playFile() {
        Task {
            do {
                if file.type == .directory {
                    try await playDirectory(path: file.path)
                } else {
                    try await playDirectory(path: currentDirectory, position: Int32(selectedIndex))
                }
                await player.fetchQueue()
            } catch {
                errorText = String(describing: error)
            }
        }
    }
}
