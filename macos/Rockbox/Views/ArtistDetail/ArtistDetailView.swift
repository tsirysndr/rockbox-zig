//
//  ArtistDetailsView.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 14/12/2025.
//

import SwiftUI

// MARK: - Artist Detail View

struct ArtistDetailView: View {
    let artist: Artist
    @ObservedObject var library: MusicLibrary
    var onBack: () -> Void
    var onAlbumSelected: (Album) -> Void
    
    var artistAlbums: [Album] {
        artist.albums(from: sampleAlbums)
    }
    
    var artistTracks: [Song] {
        artist.tracks(from: sampleAlbums)
    }
    
    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 0) {
                // Artist header
                ArtistHeaderView(
                    artist: artist,
                    trackCount: artistTracks.count,
                    albumCount: artistAlbums.count,
                    onBack: onBack
                )
                
                Divider()
                    .padding(.top, 20)
                
                // Tracks section
                if !artistTracks.isEmpty {
                    SectionHeaderView(title: "Songs", count: artistTracks.count)
                    
                    LazyVStack(spacing: 0) {
                        ForEach(Array(artistTracks.enumerated()), id: \.element.id) { index, track in
                            ArtistTrackRowView(
                                track: track,
                                index: index + 1,
                                isEven: index % 2 == 0,
                                library: library
                            )
                        }
                    }
                }
                
                // Albums section
                if !artistAlbums.isEmpty {
                    SectionHeaderView(title: "Albums", count: artistAlbums.count)
                        .padding(.top, 24)
                    
                    ArtistAlbumsGridView(albums: artistAlbums, onAlbumSelected: onAlbumSelected)
                }
            }
        }
    }
}

struct SectionHeaderView: View {
    let title: String
    let count: Int
    
    var body: some View {
        HStack {
            Text(title)
                .font(.system(size: 18, weight: .bold))
            
            Text("\(count)")
                .font(.system(size: 14))
                .foregroundStyle(.secondary)
            
            Spacer()
        }
        .padding(.horizontal, 24)
        .padding(.vertical, 12)
    }
}

