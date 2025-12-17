//
//  Song.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 14/12/2025.
//

import SwiftUI

struct Song: Identifiable {
    let id = UUID()
    let cuid: String
    let title: String
    let artist: String
    let album: String
    let albumArt: URL?
    let duration: TimeInterval
    let trackNumber: Int
    let discNumber: Int
    let color: Color
}
