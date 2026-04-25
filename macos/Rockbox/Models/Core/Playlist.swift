//
//  Playlist.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 21/12/2025.
//

import SwiftUI

struct Playlist: Identifiable {
    let id = UUID()
    let cuid: String
    let name: String
    let description: String?
    let tracks: [Song]
}

struct SavedPlaylist: Identifiable {
    let id: String
    let name: String
    let description: String?
    let image: String?
    let folderID: String?
    let trackCount: Int64
}

struct SmartPlaylist: Identifiable {
    let id: String
    let name: String
    let description: String?
    let image: String?
    let isSystem: Bool
}
