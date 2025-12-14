//
//  SongHeaderRow.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 14/12/2025.
//

import SwiftUI

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

