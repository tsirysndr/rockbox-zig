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

