//
//  Album.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 14/12/2025.
//
import SwiftUI

struct Album: Identifiable {
    let id = UUID()
    let cuid: String
    let title: String
    let artist: String
    let year: Int
    let color: Color
    let cover: String
    let releaseDate: String?
    let copyrightMessage: String?
    let artistID: String
    let tracks: [Song]
}
