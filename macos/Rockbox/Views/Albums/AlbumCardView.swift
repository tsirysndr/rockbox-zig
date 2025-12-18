//
//  AlbumCardView.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 14/12/2025.
//

import SwiftUI

struct AlbumCardView: View {
    let album: Album
    @State private var isHovering = false
    @State private var isHoveringPlayButton = false  // Track play button hover
    @State private var errorText: String?
    var onSelect: () -> Void
    
    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            // Album artwork
            ZStack {
                RoundedRectangle(cornerRadius: 5)
                    .fill(album.color.gradient)
                    .aspectRatio(1, contentMode: .fit)
                    .overlay {
                        AsyncImage(url: URL(string: album.cover)) { phase in
                            switch phase {
                            case .empty:
                                Image(systemName: "music.note")
                                    .font(.system(size: 40))
                                    .foregroundStyle(.white.opacity(0.6))
                            case .success(let image):
                                image
                                    .resizable()
                                    .aspectRatio(contentMode: .fill)
                            case .failure:
                                Image(systemName: "music.note")
                                    .font(.system(size: 40))
                                    .foregroundStyle(.white.opacity(0.6))
                            @unknown default:
                                EmptyView()
                            }
                        }
                    }
                    .clipShape(RoundedRectangle(cornerRadius: 5))
                    .onTapGesture {
                        onSelect()  // Tap on artwork selects album
                    }
                
                if isHovering {
                    Button(action: {
                        Task {
                            do {
                                try await playAlbum(albumID: album.cuid)
                            } catch {
                                errorText = String(describing: error)
                            }
                        }
                    }) {
                        ZStack {
                            Circle()
                                .fill(.black.opacity(0.5))
                                .frame(width: 44, height: 44)
                            
                            Image(systemName: "play.fill")
                                .font(.system(size: 18))
                                .foregroundStyle(.white)
                        }
                    }
                    .buttonStyle(.borderless)
                }
            }
            .onHover { hovering in
                withAnimation(.easeInOut(duration: 0.2)) {
                    isHovering = hovering
                }
            }
            
            // Album info - tapping here also selects
            VStack(alignment: .leading, spacing: 2) {
                Text(album.title)
                    .font(.system(size: 12, weight: .medium))
                    .lineLimit(1)
                
                Text("\(album.artist) · \(String(album.year))")
                    .font(.system(size: 11))
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
            }
            .onTapGesture {
                onSelect()
            }
        }
        .alert("gRPC Error", isPresented: .constant(errorText != nil)) {
            Button("OK") { errorText = nil }
        } message: {
            Text(errorText ?? "")
        }
    }
}
