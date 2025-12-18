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
    
    @State private var isHovering = false
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
                                try await playTrack(path: file.path)
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

