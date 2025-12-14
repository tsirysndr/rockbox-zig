//
//  AlbumsGridView.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 14/12/2025.
//

import SwiftUI

struct AlbumsGridView: View {
    @Binding var selectedAlbum: Album?
    
    private let columns = [
        GridItem(.adaptive(minimum: 150, maximum: 180), spacing: 20)
    ]
    
    var body: some View {
        ScrollView {
            LazyVGrid(columns: columns, spacing: 24) {
                ForEach(sampleAlbums) { album in
                    AlbumCardView(album: album)
                        .onTapGesture {
                            selectedAlbum = album
                        }
                }
            }
            .padding(20)
        }
    }
}
