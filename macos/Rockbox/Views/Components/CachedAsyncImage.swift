//
//  CachedImageLoader.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 20/12/2025.
//
import SwiftUI

enum CachedAsyncImagePhase {
    case empty
    case success(Image)
    case failure(Error)
}

struct CachedAsyncImage<Content: View>: View {
    let url: URL?
    let content: (CachedAsyncImagePhase) -> Content
    
    @State private var phase: CachedAsyncImagePhase = .empty
    
    init(url: URL?, @ViewBuilder content: @escaping (CachedAsyncImagePhase) -> Content) {
        self.url = url
        self.content = content
    }
    
    var body: some View {
        content(phase)
            .onAppear {
                loadImage()
            }
            .onChange(of: url) {
                loadImage()
            }
    }
    
    private func loadImage() {
        guard let url = url else {
            phase = .empty
            return
        }
        
        // Check cache first
        if let cached = ImageCache.shared.image(for: url) {
            phase = .success(Image(nsImage: cached))
            return
        }
        
        // Reset to loading state
        phase = .empty
        
        // Load from network
        Task {
            do {
                let (data, _) = try await URLSession.shared.data(from: url)
                if let nsImage = NSImage(data: data) {
                    ImageCache.shared.setImage(nsImage, for: url)
                    phase = .success(Image(nsImage: nsImage))
                } else {
                    phase = .failure(URLError(.cannotDecodeContentData))
                }
            } catch {
                phase = .failure(error)
            }
        }
    }
}

// Convenience initializer for simple usage
extension CachedAsyncImage where Content == AnyView {
    init(url: URL?) {
        self.url = url
        self.content = { phase in
            AnyView(
                Group {
                    switch phase {
                    case .empty:
                        ProgressView()
                            .scaleEffect(0.5)
                    case .success(let image):
                        image
                            .resizable()
                            .aspectRatio(contentMode: .fill)
                    case .failure:
                        Image(systemName: "music.note")
                            .foregroundStyle(.white.opacity(0.6))
                    }
                }
            )
        }
    }
}

@MainActor
class ImageCache {
    static let shared = ImageCache()
    private var cache = NSCache<NSURL, NSImage>()
    
    func image(for url: URL) -> NSImage? {
        cache.object(forKey: url as NSURL)
    }
    
    func setImage(_ image: NSImage, for url: URL) {
        cache.setObject(image, forKey: url as NSURL)
    }
}
