//
//  PlayerState.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 14/12/2025.
//

import Foundation

class PlayerState: ObservableObject {
    @Published var isPlaying = false
    @Published var currentTime: TimeInterval = 73
    @Published var duration: TimeInterval = 237
    @Published var currentTrack = Track(
        title: "Bohemian Rhapsody",
        artist: "Queen",
        album: "A Night at the Opera",
        artworkColor: .purple
    )
    
    var progress: Double {
        get { duration > 0 ? currentTime / duration : 0 }
        set { currentTime = newValue * duration }
    }
}
