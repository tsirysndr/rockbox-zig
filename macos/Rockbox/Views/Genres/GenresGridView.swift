//
//  GenresGridView.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 04/05/2026.
//

import SwiftUI

struct GenresGridView: View {
    @State private var genres: [Genre] = []
    @State private var errorText: String?
    @Binding var selectedGenre: Genre?

    private let columns = [
        GridItem(.adaptive(minimum: 200, maximum: 280), spacing: 16)
    ]

    var body: some View {
        ScrollView {
            LazyVGrid(columns: columns, spacing: 16) {
                ForEach(genres) { genre in
                    GenreCardView(genre: genre) {
                        selectedGenre = genre
                    }
                }
            }
            .padding(20)
        }
        .task {
            do {
                let data = try await fetchGenres()
                genres = data.map {
                    Genre(
                        cuid: $0.id,
                        name: $0.name,
                        description: $0.hasDescription_p ? $0.description_p : nil,
                        image: $0.hasImage ? $0.image : nil,
                        trackCount: $0.trackCount,
                        color: Genre.colorForSeed($0.id.isEmpty ? $0.name : $0.id)
                    )
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

struct GenreCardView: View {
    let genre: Genre
    let onTap: () -> Void

    var body: some View {
        Button(action: onTap) {
            ZStack(alignment: .bottomLeading) {
                LinearGradient(
                    colors: [genre.color, genre.color.opacity(0.6)],
                    startPoint: .topLeading,
                    endPoint: .bottomTrailing
                )
                .frame(height: 120)
                .clipShape(RoundedRectangle(cornerRadius: 8))

                Text(genre.name)
                    .font(.system(size: 32, weight: .heavy))
                    .foregroundStyle(Color.white.opacity(0.18))
                    .rotationEffect(.degrees(-12))
                    .offset(x: 80, y: 12)
                    .frame(height: 120, alignment: .bottomTrailing)
                    .clipShape(RoundedRectangle(cornerRadius: 8))

                VStack(alignment: .leading, spacing: 6) {
                    Text(genre.name)
                        .font(.system(size: 18, weight: .semibold))
                        .foregroundStyle(.white)
                    Text("\(genre.trackCount) \(genre.trackCount == 1 ? "track" : "tracks")")
                        .font(.system(size: 11))
                        .foregroundStyle(Color.white.opacity(0.85))
                }
                .padding(14)
            }
        }
        .buttonStyle(.plain)
    }
}
