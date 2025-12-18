//
//  ArtistsGridView.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 14/12/2025.
//

import SwiftUI


struct ArtistsGridView: View {
    @State private var artists: [Artist] = []
    @State private var errorText: String?
    @Binding var selectedArtist: Artist?
    
    private let columns = [
        GridItem(.adaptive(minimum: 150, maximum: 180), spacing: 20)
    ]
    
    var body: some View {
        ScrollView {
            LazyVGrid(columns: columns, spacing: 24) {
                ForEach(artists) { artist in
                    ArtistCardView(artist: artist)
                        .onTapGesture {
                            selectedArtist = artist
                        }
                }
            }
            .padding(20)
        }
        .task {
            do {
                let data = try await fetchArtists()
                artists = []
                for artist in data {
                    artists.append(Artist(cuid: artist.id, name: artist.name, image: artist.image, genre: artist.genres, color: .gray.opacity(0.3)))
                }
                
            } catch {
                errorText = String(describing: error)
            }
        }
        .alert("gRPC Error", isPresented: .constant(errorText != nil)) {
          Button("OK") { errorText = nil }
         } message: {
           Text(errorText ?? "")
         }
    }
}
