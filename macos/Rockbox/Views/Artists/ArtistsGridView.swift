//
//  ArtistsGridView.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 14/12/2025.
//

import SwiftUI


struct ArtistsGridView: View {
    @Binding var selectedArtist: Artist?
    
    private let columns = [
        GridItem(.adaptive(minimum: 150, maximum: 180), spacing: 20)
    ]
    
    var body: some View {
        ScrollView {
            LazyVGrid(columns: columns, spacing: 24) {
                ForEach(sampleArtists) { artist in
                    ArtistCardView(artist: artist)
                        .onTapGesture {
                            selectedArtist = artist
                        }
                }
            }
            .padding(20)
        }
    }
}
