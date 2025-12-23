//
//  SearchManager.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 22/12/2025.
//

import SwiftUI

@MainActor
class SearchManager: ObservableObject {
    @Published var searchText: String = ""
    @Published var isSearching: Bool = false
    @Published var searchResults: SearchResults = SearchResults()
    @Published var isLoading: Bool = false
    
    private var searchTask: Task<Void, Never>?
    
    struct SearchResults {
        var songs: [Song] = []
        var albums: [Album] = []
        var artists: [Artist] = []
        
        var isEmpty: Bool {
            songs.isEmpty && albums.isEmpty && artists.isEmpty
        }
        
        var totalCount: Int {
            songs.count + albums.count + artists.count
        }
    }
    
    func search() {
        searchTask?.cancel()
        
        guard !searchText.trimmingCharacters(in: .whitespaces).isEmpty else {
            searchResults = SearchResults()
            isSearching = false
            return
        }
        
        isSearching = true
        isLoading = true
        
        searchTask = Task {
            // Debounce
            try? await Task.sleep(for: .milliseconds(300))
            
            guard !Task.isCancelled else { return }
            
            do {
                let results = try await searchTrack(query: searchText)
                
                guard !Task.isCancelled else { return }
                
                searchResults = SearchResults(
                    songs: results.tracks.map { track in
                        Song(
                            cuid: track.id,
                            path: track.path,
                            title: track.title,
                            artist: track.artist,
                            album: track.album,
                            albumArt: URL(string: "http://localhost:6062/covers/" + track.albumArt),
                            duration: TimeInterval(track.length / 1000),
                            trackNumber: Int(track.trackNumber),
                            discNumber: Int(track.discNumber),
                            albumID: track.albumID,
                            artistID: track.artistID,
                            color: .gray.opacity(0.3)
                        )
                    },
                    albums: results.albums.map { album in
                        Album(
                            cuid: album.id,
                            title: album.title,
                            artist: album.artist,
                            year: Int(album.year),
                            color: .gray.opacity(0.3),
                            cover: "http://localhost:6062/covers/" + album.albumArt,
                            releaseDate: album.yearString,
                            copyrightMessage: album.copyrightMessage,
                            artistID: album.artistID,
                            tracks: []
                        )
                    },
                    artists: results.artists.map { artist in
                        Artist(
                            cuid: artist.id,
                            name: artist.name,
                            image: artist.image,
                            genre: artist.genres,
                            color: .gray.opacity(0.3)
                        )
                    }
                )
                
                isLoading = false
            } catch {
                if !Task.isCancelled {
                    isLoading = false
                    print("Search error: \(error)")
                }
            }
        }
    }
    
    func clear() {
        searchText = ""
        searchResults = SearchResults()
        isSearching = false
        isLoading = false
        searchTask?.cancel()
    }
}

