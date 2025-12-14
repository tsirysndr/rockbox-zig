//
//  MusicLibrary.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 14/12/2025.
//

import Foundation

class MusicLibrary: ObservableObject {
    @Published var likedSongIds: Set<String> = []
    
    func isLiked(_ song: Song) -> Bool {
        likedSongIds.contains(song.cuid)
    }
    
    func toggleLike(_ song: Song) {
        if likedSongIds.contains(song.cuid) {
            likedSongIds.remove(song.cuid)
        } else {
            likedSongIds.insert(song.cuid)
        }
    }
    
    func likedSongs(from songs: [Song]) -> [Song] {
        songs.filter { likedSongIds.contains($0.cuid) }
    }
}
