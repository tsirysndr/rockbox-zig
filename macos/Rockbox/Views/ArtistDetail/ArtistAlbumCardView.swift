//
//  ArtistAlbumCardView.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 14/12/2025.
//

import SwiftUI

struct ArtistAlbumCardView: View {
    let album: Album
    var playlists: [Playlist] = []
    var onSelect: () -> Void

    @State private var isHovering = false
    @State private var isHoveringPlayButton = false
    @State private var isHoveringMenu = false
    @State private var showMenu = false
    @State private var errorText: String?
    
    @EnvironmentObject var player: PlayerState

    
    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            // Album artwork
            RoundedRectangle(cornerRadius: 5)
                .fill(album.color.gradient)
                .aspectRatio(1, contentMode: .fit)
                .overlay {
                    CachedAsyncImage(url: URL(string: album.cover)) { phase in
                        switch phase {
                        case .success(let image):
                            image
                                .resizable()
                                .aspectRatio(contentMode: .fill)
                        default:
                            Image(systemName: "music.note")
                                .font(.system(size: 40))
                                .foregroundStyle(.white.opacity(0.6))
                        }
                    }
                }
                .overlay(alignment: .bottom) {
                    if isHovering || showMenu {
                        HStack(spacing: 0) {
                            // Play button - left half
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
                                    .fill(isHoveringPlayButton ? Color(hex: "fe09a3") : .white.opacity(0.3))
                                    .frame(width: 32, height: 32)
                                    .overlay {
                                        Image(systemName: "play.fill")
                                            .font(.system(size: 12))
                                            .foregroundStyle(.white)
                                    }
                            }
                            .buttonStyle(.borderless)
                            .onHover { isHoveringPlayButton = $0 }
                            .frame(maxWidth: .infinity, alignment: .center)
                            
                            // Context menu - right half
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
                                                    do {
                                                        try await playAlbum(albumID: album.cuid)
                                                        await player.fetchQueue()
                                                    } catch {
                                                        errorText = String(describing: error)
                                                    }
                                                }
                                            }
                                            
                                            MenuItemButton(title: "Play Shuffled", icon: "shuffle") {
                                                showMenu = false
                                                Task {
                                                    do {
                                                        try await playAlbum(albumID: album.cuid, shuffle: true)
                                                        await player.fetchQueue()
                                                    } catch {
                                                        errorText = String(describing: error)
                                                    }
                                                }
                                            }
                                            
                                            Divider().padding(.vertical, 4)
                                            
                                            MenuItemButton(title: "Play Next", icon: "text.insert") {
                                                showMenu = false
                                                Task {
                                                    do {
                                                        try await insertAlbum(albumID: album.cuid, position: Int32(PlaylistPosition.insertFirst))
                                                        await player.fetchQueue()
                                                    } catch {
                                                        errorText = String(describing: error)
                                                    }
                                                }
                                            }
                                            
                                            MenuItemButton(title: "Play Last", icon: "text.append") {
                                                showMenu = false
                                                Task {
                                                    do {
                                                        try await insertAlbum(albumID: album.cuid, position: Int32(PlaylistPosition.insertLast))
                                                        await player.fetchQueue()
                                                    } catch {
                                                        errorText = String(describing: error)
                                                    }
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
                                                    // Add to selected playlist
                                                },
                                                onCreateNew: {
                                                    showMenu = false
                                                    // Create new playlist
                                                },
                                                action: {}
                                            )
                                            
                                            Divider().padding(.vertical, 4)
                                            
                                            MenuItemButton(title: "Add Shuffled", icon: "shuffle") {
                                                showMenu = false
                                                Task {
                                                    do {
                                                        try await insertAlbum(albumID: album.cuid, position: Int32(PlaylistPosition.insertShuffled))
                                                        await player.fetchQueue()
                                                    } catch {
                                                        errorText = String(describing: error)
                                                    }
                                                }
                                            }
                                            
                                            MenuItemButton(title: "Play Last Shuffled", icon: "shuffle") {
                                                showMenu = false
                                                Task {
                                                    do {
                                                        try await insertAlbum(albumID: album.cuid, position: Int32(PlaylistPosition.insertLastShuffled))
                                                        await player.fetchQueue()
                                                    } catch {
                                                        errorText = String(describing: error)
                                                    }
                                                }
                                            }
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
                .clipShape(RoundedRectangle(cornerRadius: 5))
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
            
            // Album info
            VStack(alignment: .leading, spacing: 2) {
                Text(album.title)
                    .font(.system(size: 11, weight: .medium))
                    .lineLimit(1)
                
                Text(String(album.year))
                    .font(.system(size: 10))
                    .foregroundStyle(.secondary)
            }
            .onTapGesture {
                onSelect()
            }
        }
        .contentShape(Rectangle())
        .alert("gRPC Error", isPresented: .constant(errorText != nil)) {
            Button("OK") { errorText = nil }
        } message: {
            Text(errorText ?? "")
        }
    }
}

