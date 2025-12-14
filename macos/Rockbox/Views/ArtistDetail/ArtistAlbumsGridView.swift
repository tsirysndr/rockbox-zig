//
//  ArtistAlbumsGridView.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 14/12/2025.
//

import SwiftUI

struct ArtistAlbumsGridView: View {
    let albums: [Album]
    var onAlbumSelected: (Album) -> Void
    
    private let columns = [
        GridItem(.adaptive(minimum: 140, maximum: 160), spacing: 16)
    ]
    
    var body: some View {
        LazyVGrid(columns: columns, spacing: 20) {
            ForEach(albums) { album in
                ArtistAlbumCardView(album: album)
                    .onTapGesture {
                        onAlbumSelected(album)
                    }
            }
        }
        .padding(.horizontal, 24)
        .padding(.bottom, 24)
    }
}

