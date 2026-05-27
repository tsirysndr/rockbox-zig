//
//  SidebarItem.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 14/12/2025.
//

import SwiftUI

enum SidebarItem: String, CaseIterable, Identifiable {
    case home = "Home"
    case albums = "Albums"
    case artists = "Artists"
    case genres = "Genres"
    case songs = "Songs"
    case likes = "Likes"
    case playlists = "Playlists"
    case files = "Files"
    case navidrome = "Navidrome"

    var id: String { rawValue }

    var icon: String {
        switch self {
        case .home: return "house"
        case .albums: return "square.stack"
        case .artists: return "music.mic"
        case .genres: return "guitars"
        case .songs: return "music.note"
        case .likes: return "heart"
        case .playlists: return "music.note.list"
        case .files: return "folder"
        case .navidrome: return "music.note.house"
        }
    }
}
