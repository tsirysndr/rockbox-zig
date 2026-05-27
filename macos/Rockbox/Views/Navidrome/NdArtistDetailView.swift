import SwiftUI

struct NdArtistDetailView: View {
    let artist: NdArtist
    var onBack: () -> Void
    var onAlbumSelected: (NdAlbum) -> Void

    @State private var albums: [NdAlbum] = []
    @State private var isLoading = false
    @State private var errorText: String?
    @ObservedObject private var ndManager = NavidromeManager.shared

    private var server: NdServer? { ndManager.activeServer }
    private let columns = [GridItem(.adaptive(minimum: 170, maximum: 230), spacing: 20)]

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 0) {
                artistHeader

                if !albums.isEmpty {
                    HStack {
                        Text("Albums")
                            .font(.system(size: 18, weight: .bold))
                        Text("\(albums.count)")
                            .font(.system(size: 14))
                            .foregroundStyle(.secondary)
                        Spacer()
                    }
                    .padding(.horizontal, 24)
                    .padding(.vertical, 12)

                    LazyVGrid(columns: columns, spacing: 24) {
                        ForEach(albums) { album in
                            if let server {
                                NdAlbumCardView(album: album, server: server)
                                    .onTapGesture { onAlbumSelected(album) }
                            }
                        }
                    }
                    .padding(.horizontal, 24)
                    .padding(.bottom, 40)
                }
            }
        }
        .task(id: artist.id) {
            guard let server else { return }
            isLoading = true
            defer { isLoading = false }
            if let detail = try? await ndGetArtist(server: server, artistId: artist.id) {
                albums = detail.albums
            }
        }
        .alert("Error", isPresented: .constant(errorText != nil)) {
            Button("OK") { errorText = nil }
        } message: {
            Text(errorText ?? "")
        }
    }

    private var artistHeader: some View {
        HStack(alignment: .top, spacing: 24) {
            VStack(alignment: .leading, spacing: 12) {
                Button(action: onBack) {
                    HStack(spacing: 4) {
                        Image(systemName: "chevron.left").font(.system(size: 14, weight: .semibold))
                        Text("Back").font(.system(size: 13))
                    }
                    .foregroundStyle(.secondary)
                    .frame(minHeight: 34)
                    .contentShape(Rectangle())
                }
                .buttonStyle(.plain)

                Circle()
                    .fill(Color.gray.opacity(0.2).gradient)
                    .frame(width: 200, height: 200)
                    .overlay {
                        if let coverId = artist.coverArt, let url = server?.coverArtUrl(coverId: coverId, size: 400) {
                            CachedAsyncImage(url: url) { phase in
                                switch phase {
                                case .success(let img):
                                    img.resizable().aspectRatio(contentMode: .fill)
                                default:
                                    Image(systemName: "music.mic")
                                        .font(.system(size: 60))
                                        .foregroundStyle(.white.opacity(0.6))
                                }
                            }
                        } else {
                            Image(systemName: "music.mic")
                                .font(.system(size: 60))
                                .foregroundStyle(.white.opacity(0.6))
                        }
                    }
                    .clipShape(Circle())
            }

            VStack(alignment: .leading, spacing: 8) {
                Text(artist.name)
                    .font(.system(size: 28, weight: .bold))
                    .padding(.top, 30)
                Text("\(artist.albumCount) \(artist.albumCount == 1 ? "album" : "albums")")
                    .font(.system(size: 14))
                    .foregroundStyle(.secondary)
            }
            .frame(maxWidth: .infinity, alignment: .leading)
        }
        .padding(24)
    }
}
