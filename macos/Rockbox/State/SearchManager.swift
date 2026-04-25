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
    @Published var allSavedPlaylists: [SavedPlaylist] = []
    @Published var allSmartPlaylists: [SmartPlaylist] = []

    private var searchTask: Task<Void, Never>?
    private var playlistsLoaded = false

    struct SearchResults {
        var songs: [Song] = []
        var albums: [Album] = []
        var artists: [Artist] = []

        var isEmpty: Bool {
            songs.isEmpty && albums.isEmpty && artists.isEmpty
        }
    }

    var filteredSavedPlaylists: [SavedPlaylist] {
        let q = searchText.trimmingCharacters(in: .whitespaces).lowercased()
        guard !q.isEmpty else { return [] }
        return allSavedPlaylists.filter { $0.name.lowercased().contains(q) }
    }

    var filteredSmartPlaylists: [SmartPlaylist] {
        let q = searchText.trimmingCharacters(in: .whitespaces).lowercased()
        guard !q.isEmpty else { return [] }
        return allSmartPlaylists.filter { $0.name.lowercased().contains(q) }
    }

    var hasPlaylistResults: Bool {
        !filteredSavedPlaylists.isEmpty || !filteredSmartPlaylists.isEmpty
    }

    func loadPlaylists() async {
        guard !playlistsLoaded else { return }
        playlistsLoaded = true
        async let savedData = fetchSavedPlaylists()
        async let smartData = fetchSmartPlaylists()
        if let saved = try? await savedData {
            allSavedPlaylists = saved.map {
                SavedPlaylist(
                    id: $0.id, name: $0.name,
                    description: $0.hasDescription_p ? $0.description_p : nil,
                    image: $0.hasImage ? $0.image : nil,
                    folderID: $0.hasFolderID ? $0.folderID : nil,
                    trackCount: $0.trackCount
                )
            }
        }
        if let smart = try? await smartData {
            allSmartPlaylists = smart.map {
                SmartPlaylist(
                    id: $0.id, name: $0.name,
                    description: $0.hasDescription_p ? $0.description_p : nil,
                    image: $0.hasImage ? $0.image : nil,
                    isSystem: $0.isSystem
                )
            }
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

        searchTask = Task {
            // Load playlists once in the background (non-blocking for search results)
            if !playlistsLoaded {
                await loadPlaylists()
            }

            // Debounce gRPC search
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
            } catch {
                if !Task.isCancelled {
                    print("Search error: \(error)")
                }
            }
        }
    }

    func clear() {
        searchText = ""
        searchResults = SearchResults()
        isSearching = false
        searchTask?.cancel()
    }
}
