//
//  Album.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 14/12/2025.
//
import SwiftUI

struct Album: Identifiable {
    let id = UUID()
    let title: String
    let artist: String
    let year: Int
    let color: Color
    let tracks: [Song]
}
