//
//  File.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 14/12/2025.
//

import SwiftUI

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
