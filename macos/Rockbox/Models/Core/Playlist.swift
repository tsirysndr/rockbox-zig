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
