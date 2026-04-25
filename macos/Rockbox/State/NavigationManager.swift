//
//  NavigationManager.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 21/12/2025.
//

import SwiftUI

@MainActor
class NavigationManager: ObservableObject {
    @Published var selectedAlbum: Album? = nil
    @Published var selectedArtist: Artist? = nil
    @Published var selectedPlaylist: SavedPlaylist? = nil
    @Published var selectedSmartPlaylist: SmartPlaylist? = nil
    
    func goToAlbum(_ album: Album) {
        selectedArtist = nil
        selectedPlaylist = nil
        selectedSmartPlaylist = nil
        selectedAlbum = album
    }
    
    func goToAlbum(byId albumId: String) async {
        do {
            let albumData = try await fetchAlbum(id: albumId)
            let album = Album(
                cuid: albumData.id,
                title: albumData.title,
                artist: albumData.artist,
                year: Int(albumData.year),
                color: .gray.opacity(0.3),
                cover: "http://localhost:6062/covers/" + albumData.albumArt,
                releaseDate: albumData.yearString,
                copyrightMessage: albumData.copyrightMessage,
                artistID: albumData.artistID,
                tracks: []
            )
            selectedArtist = nil
            selectedAlbum = album
        } catch {
            print("Error fetching album: \(error)")
        }
    }
    
    func goToPlaylist(_ playlist: SavedPlaylist) {
        selectedAlbum = nil
        selectedArtist = nil
        selectedSmartPlaylist = nil
        selectedPlaylist = playlist
    }

    func goToSmartPlaylist(_ playlist: SmartPlaylist) {
        selectedAlbum = nil
        selectedArtist = nil
        selectedPlaylist = nil
        selectedSmartPlaylist = playlist
    }

    func goToArtist(_ artist: Artist) {
        selectedAlbum = nil
        selectedPlaylist = nil
        selectedSmartPlaylist = nil
        selectedArtist = artist
    }
    
    func goToArtist(byId artistID: String) async {
        do {
            let artistData = try await fetchArtist(id: artistID)
            let artist = Artist(
                cuid: artistData.id,
                name: artistData.name,
                image: artistData.image,
                genre: artistData.genres,
                color: .gray.opacity(0.3)
            )
            selectedAlbum = nil
            selectedArtist = artist
        } catch {
            print("Error fetching artist: \(error)")
        }
    }
    
    func goBack() {
        if selectedAlbum != nil {
            selectedAlbum = nil
        } else if selectedArtist != nil {
            selectedArtist = nil
        } else if selectedPlaylist != nil {
            selectedPlaylist = nil
        } else if selectedSmartPlaylist != nil {
            selectedSmartPlaylist = nil
        }
    }

    func reset() {
        selectedAlbum = nil
        selectedArtist = nil
        selectedPlaylist = nil
        selectedSmartPlaylist = nil
    }
}

