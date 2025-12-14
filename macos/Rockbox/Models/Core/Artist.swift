//
//  Artist.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 14/12/2025.
//
import SwiftUI

struct Artist: Identifiable {
    let id = UUID()
    let cuid: String
    let name: String
    let genre: String
    let color: Color
    
    func albums(from allAlbums: [Album]) -> [Album] {
        allAlbums.filter { $0.artist == name }
    }
    
    func tracks(from allAlbums: [Album]) -> [Song] {
        albums(from: allAlbums).flatMap { $0.tracks }
    }
}
