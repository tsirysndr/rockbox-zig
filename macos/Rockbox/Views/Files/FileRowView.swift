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
    
    var body: some View {
        HStack(spacing: 12) {
            // Icon and name
            HStack(spacing: 10) {
                ZStack {
                    Image(systemName: file.icon)
                        .font(.system(size: 20))
                        .foregroundStyle(file.iconColor)
                        .opacity(isHovering ? 0 : 1)
                    
                    Button(action: {
                        Task {
                            do {
                                if file.type == .directory {
                                    try await playDirectory(path: file.path)
                                    return
                                }
                                try await playDirectory(path: currentDirectory, position: Int32(selectedIndex))
                            } catch {
                                errorText = String(describing: error)
                            }
                        }
                    }) {
                        Image(systemName: "play.fill")
                            .font(.system(size: 15))
                            .opacity(isHovering ? 1 : 0)
                    }.buttonStyle(.plain)
                }
                .frame(width: 30, alignment: .center)
                .foregroundStyle(.secondary)

                
                VStack(alignment: .leading, spacing: 2) {
                    Text(file.name)
                        .font(.system(size: 12, weight: .medium))
                        .lineLimit(1)

                }
                
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            
            /*
             Play Next
             Add to Playlist
             Play Last
             Add Shuffled
             */
            // Context menu button
            Menu {
                Button(action: {
                    Task {
                        do {
                            // try await addToQueue(songId: song.cuid)
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
                            // try await addToQueueLast(songId: song.cuid)
                        } catch {
                            errorText = String(describing: error)
                        }
                    }
                }) {
                    Label("Add to Playlist", systemImage: "text.append")
                }
                
                Button(action: {
                    Task {
                        do {
                            // try await addToQueueLast(songId: song.cuid)
                        } catch {
                            errorText = String(describing: error)
                        }
                    }
                }) {
                    Label("Play Last", systemImage: "text.append")
                }
                
                Divider()
                
                Button(action: {
                    // Add Shuffled
                }) {
                    Label("Add Shuffled", systemImage: "square.stack")
                }
                
                if file.type == .directory {
                    Button(action: {
                        // Play Last Shuffled
                    }) {
                        Label("Play Last Shuffled", systemImage: "music.mic")
                        
                    }
                    
                    Button(action: {
                        // Play Last Shuffled
                    }) {
                        Label("Play Shuffled", systemImage: "music.mic")
                        
                    }
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
                .onHover { hovering in
                    isHoveringMenu = hovering
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

