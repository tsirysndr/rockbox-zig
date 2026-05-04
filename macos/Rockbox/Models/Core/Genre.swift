//
//  Genre.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 04/05/2026.
//

import SwiftUI

struct Genre: Identifiable {
    let id = UUID()
    let cuid: String
    let name: String
    let description: String?
    let image: String?
    let trackCount: Int64
    let color: Color

    static func colorForSeed(_ seed: String) -> Color {
        var hash: UInt64 = 0
        for codeUnit in seed.unicodeScalars {
            hash = hash &* 31 &+ UInt64(codeUnit.value)
        }
        let hue = Double(hash % 360) / 360.0
        return Color(hue: hue, saturation: 0.65, brightness: 0.45)
    }
}
