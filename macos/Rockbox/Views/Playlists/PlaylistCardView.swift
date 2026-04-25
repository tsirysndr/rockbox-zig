//
//  PlaylistCardView.swift
//  Rockbox
//

import SwiftUI

struct PlaylistCardView: View {
    let playlist: SavedPlaylist
    var onSelect: () -> Void
    var onPlay: () -> Void
    var onDelete: () -> Void

    @State private var isHovering = false
    @State private var isHoveringPlayButton = false
    @State private var showMenu = false
    @State private var isHoveringMenu = false

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            RoundedRectangle(cornerRadius: 5)
                .fill(Color.gray.opacity(0.3).gradient)
                .aspectRatio(1, contentMode: .fit)
                .overlay {
                    if let imageURL = playlist.image, !imageURL.isEmpty {
                        CachedAsyncImage(url: URL(string: "http://localhost:6062/covers/" + imageURL)) { phase in
                            switch phase {
                            case .success(let image):
                                image.resizable().aspectRatio(contentMode: .fill)
                            default:
                                Image(systemName: "music.note.list")
                                    .font(.system(size: 40))
                                    .foregroundStyle(.white.opacity(0.6))
                            }
                        }
                    } else {
                        Image(systemName: "music.note.list")
                            .font(.system(size: 40))
                            .foregroundStyle(.white.opacity(0.6))
                    }
                }
                .overlay(alignment: .bottom) {
                    if isHovering || showMenu {
                        HStack(spacing: 0) {
                            Button(action: onPlay) {
                                Circle()
                                    .fill(isHoveringPlayButton ? Color(hex: "fe09a3") : .white.opacity(0.3))
                                    .frame(width: 36, height: 36)
                                    .overlay {
                                        Image(systemName: "play.fill")
                                            .font(.system(size: 14))
                                            .foregroundStyle(.white)
                                    }
                            }
                            .buttonStyle(.borderless)
                            .onHover { isHoveringPlayButton = $0 }
                            .frame(maxWidth: .infinity, alignment: .center)

                            ZStack {
                                Circle()
                                    .fill(isHoveringMenu || showMenu ? Color(hex: "fe09a3") : .white.opacity(0.3))
                                    .frame(width: 36, height: 36)
                                Image(systemName: "ellipsis")
                                    .font(.system(size: 14, weight: .medium))
                                    .foregroundStyle(.white)
                                    .allowsHitTesting(false)
                                Button(action: { showMenu.toggle() }) {
                                    Circle().fill(Color.clear).frame(width: 36, height: 36)
                                }
                                .buttonStyle(.borderless)
                                .onHover { isHoveringMenu = $0 }
                                .popover(isPresented: $showMenu, arrowEdge: .bottom) {
                                    ZStack {
                                        Color.white.ignoresSafeArea()
                                        VStack(alignment: .leading, spacing: 0) {
                                            MenuItemButton(title: "Play", icon: "play.fill") {
                                                showMenu = false; onPlay()
                                            }
                                            Divider().padding(.vertical, 4)
                                            MenuItemButton(title: "Delete", icon: "trash") {
                                                showMenu = false; onDelete()
                                            }
                                        }
                                        .padding(8)
                                        .frame(width: 180)
                                    }
                                }
                            }
                            .frame(maxWidth: .infinity, alignment: .center)
                        }
                        .padding(.vertical, 12)
                        .transition(.opacity.combined(with: .move(edge: .bottom)))
                    }
                }
                .clipShape(RoundedRectangle(cornerRadius: 5))
                .onTapGesture { onSelect() }
                .onHover { hovering in
                    withAnimation(.easeInOut(duration: 0.2)) {
                        if !showMenu { isHovering = hovering }
                    }
                }
                .onChange(of: showMenu) { _, newValue in
                    if !newValue {
                        withAnimation(.easeInOut(duration: 0.2)) { isHovering = false }
                    }
                }

            VStack(alignment: .leading, spacing: 2) {
                Text(playlist.name)
                    .font(.system(size: 12, weight: .medium))
                    .lineLimit(1)
                Text("\(playlist.trackCount) tracks")
                    .font(.system(size: 11))
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
            }
            .onTapGesture { onSelect() }
        }
    }
}

struct SmartPlaylistCardView: View {
    let playlist: SmartPlaylist
    var onSelect: () -> Void
    var onPlay: () -> Void

    @State private var isHovering = false
    @State private var isHoveringPlayButton = false

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            RoundedRectangle(cornerRadius: 5)
                .fill(Color.purple.opacity(0.25).gradient)
                .aspectRatio(1, contentMode: .fit)
                .overlay {
                    Image(systemName: playlist.isSystem ? "sparkles" : "wand.and.stars")
                        .font(.system(size: 40))
                        .foregroundStyle(.white.opacity(0.7))
                }
                .overlay(alignment: .bottom) {
                    if isHovering {
                        HStack(spacing: 0) {
                            Button(action: onPlay) {
                                Circle()
                                    .fill(isHoveringPlayButton ? Color(hex: "fe09a3") : .white.opacity(0.3))
                                    .frame(width: 36, height: 36)
                                    .overlay {
                                        Image(systemName: "play.fill")
                                            .font(.system(size: 14))
                                            .foregroundStyle(.white)
                                    }
                            }
                            .buttonStyle(.borderless)
                            .onHover { isHoveringPlayButton = $0 }
                            .frame(maxWidth: .infinity, alignment: .center)
                        }
                        .padding(.vertical, 12)
                        .transition(.opacity.combined(with: .move(edge: .bottom)))
                    }
                }
                .clipShape(RoundedRectangle(cornerRadius: 5))
                .onTapGesture { onSelect() }
                .onHover { hovering in
                    withAnimation(.easeInOut(duration: 0.2)) { isHovering = hovering }
                }

            VStack(alignment: .leading, spacing: 2) {
                Text(playlist.name)
                    .font(.system(size: 12, weight: .medium))
                    .lineLimit(1)
                Text(playlist.isSystem ? "Smart · System" : "Smart")
                    .font(.system(size: 11))
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
            }
            .onTapGesture { onSelect() }
        }
    }
}
