//
//  SidebarItem.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 14/12/2025.
//

import SwiftUI

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

