import CryptoKit
import Foundation

// MARK: - Models

struct NdServer: Codable, Identifiable, Equatable {
    let id: String
    var label: String
    var baseUrl: String
    var user: String
    var password: String
    var authToken: String
    var authSalt: String

    var authQueryString: String {
        "u=\(user)&t=\(authToken)&s=\(authSalt)&v=1.16.1&c=rockbox&f=json"
    }

    func coverArtUrl(coverId: String, size: Int = 300) -> URL? {
        URL(string: "\(baseUrl)/rest/getCoverArt.view?\(authQueryString)&id=\(coverId)&size=\(size)")
    }

    func streamUrl(songId: String) -> String {
        "\(baseUrl)/rest/stream.view?\(authQueryString)&id=\(songId)"
    }
}

struct NdAlbum: Identifiable, Equatable {
    let id: String
    let name: String
    let artist: String
    let artistId: String
    let coverArt: String?
    let year: Int?
    let songCount: Int

    init?(from dict: [String: Any]) {
        guard let id = dict["id"] as? String,
              let name = dict["name"] as? String
        else { return nil }
        self.id = id
        self.name = name
        self.artist = dict["artist"] as? String ?? ""
        self.artistId = dict["artistId"] as? String ?? ""
        self.coverArt = dict["coverArt"] as? String
        self.year = dict["year"] as? Int
        self.songCount = dict["songCount"] as? Int ?? 0
    }
}

struct NdArtist: Identifiable, Equatable {
    let id: String
    let name: String
    let coverArt: String?
    let albumCount: Int

    init?(from dict: [String: Any]) {
        guard let id = dict["id"] as? String,
              let name = dict["name"] as? String
        else { return nil }
        self.id = id
        self.name = name
        self.coverArt = dict["coverArt"] as? String
        self.albumCount = dict["albumCount"] as? Int ?? 0
    }
}

struct NdSong: Identifiable {
    let id: String
    let title: String
    let artist: String
    let artistId: String
    let album: String
    let albumId: String
    let coverArt: String?
    let duration: Int
    let track: Int?
    let streamUrl: String

    init?(from dict: [String: Any], server: NdServer) {
        guard let id = dict["id"] as? String,
              let title = dict["title"] as? String
        else { return nil }
        self.id = id
        self.title = title
        self.artist = dict["artist"] as? String ?? ""
        self.artistId = dict["artistId"] as? String ?? ""
        self.album = dict["album"] as? String ?? ""
        self.albumId = dict["albumId"] as? String ?? ""
        self.coverArt = dict["coverArt"] as? String
        self.duration = dict["duration"] as? Int ?? 0
        self.track = dict["track"] as? Int
        self.streamUrl = server.streamUrl(songId: id)
    }
}

struct NdPlaylist: Identifiable, Equatable {
    let id: String
    let name: String
    let comment: String?
    let coverArt: String?
    let songCount: Int
    let duration: Int

    init?(from dict: [String: Any]) {
        guard let id = dict["id"] as? String,
              let name = dict["name"] as? String
        else { return nil }
        self.id = id
        self.name = name
        self.comment = dict["comment"] as? String
        self.coverArt = dict["coverArt"] as? String
        self.songCount = dict["songCount"] as? Int ?? 0
        self.duration = dict["duration"] as? Int ?? 0
    }
}

// MARK: - Auth helpers

func ndGenerateSalt() -> String {
    let chars = "abcdefghijklmnopqrstuvwxyz0123456789"
    return String((0..<8).compactMap { _ in chars.randomElement() })
}

func ndMd5Hash(_ input: String) -> String {
    let digest = Insecure.MD5.hash(data: Data(input.utf8))
    return digest.map { String(format: "%02x", $0) }.joined()
}

// MARK: - Response cache

actor NdResponseCache {
    static let shared = NdResponseCache()
    private var store: [String: (data: [String: Any], date: Date)] = [:]
    private let ttl: TimeInterval = 1800       // 30 min — serve stale immediately
    private let staleTTL: TimeInterval = 86400 // 24 h — evict completely after this

    func get(key: String) -> [String: Any]? {
        guard let entry = store[key],
              Date().timeIntervalSince(entry.date) < staleTTL
        else { return nil }
        return entry.data
    }

    func isFresh(key: String) -> Bool {
        guard let entry = store[key] else { return false }
        return Date().timeIntervalSince(entry.date) < ttl
    }

    func set(key: String, value: [String: Any]) {
        store[key] = (value, Date())
    }

    func invalidate(prefix: String) {
        store = store.filter { !$0.key.hasPrefix(prefix) }
    }
}

private let mutatingEndpoints: Set<String> = ["star", "unstar", "scrobble"]

private func ndFetch(
    server: NdServer,
    endpoint: String,
    params: [String: String],
    cacheKey: String?
) async throws -> [String: Any] {
    var urlStr = "\(server.baseUrl)/rest/\(endpoint)?\(server.authQueryString)"
    for (k, v) in params {
        if let enc = v.addingPercentEncoding(withAllowedCharacters: .urlQueryAllowed) {
            urlStr += "&\(k)=\(enc)"
        }
    }
    guard let url = URL(string: urlStr) else { throw URLError(.badURL) }
    let (data, _) = try await URLSession.shared.data(from: url)
    guard
        let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
        let inner = json["subsonic-response"] as? [String: Any],
        inner["status"] as? String == "ok"
    else { throw URLError(.badServerResponse) }
    if let key = cacheKey {
        await NdResponseCache.shared.set(key: key, value: inner)
    }
    return inner
}

// MARK: - API

private func ndCall(
    server: NdServer,
    endpoint: String,
    params: [String: String] = [:]
) async throws -> [String: Any] {
    let cacheKey = "\(server.id):\(endpoint):\(params.sorted(by: { $0.key < $1.key }).map { "\($0.key)=\($0.value)" }.joined(separator: "&"))"
    let useCache = !mutatingEndpoints.contains(endpoint)

    if useCache {
        let cached = await NdResponseCache.shared.get(key: cacheKey)
        let fresh = await NdResponseCache.shared.isFresh(key: cacheKey)
        if let cached {
            if !fresh {
                // Stale-while-revalidate: return immediately, refresh in background
                Task.detached {
                    _ = try? await ndFetch(server: server, endpoint: endpoint, params: params, cacheKey: cacheKey)
                }
            }
            return cached
        }
    }

    return try await ndFetch(server: server, endpoint: endpoint, params: params, cacheKey: useCache ? cacheKey : nil)
}

func ndPing(baseUrl: String, user: String, password: String) async -> Bool {
    let salt = ndGenerateSalt()
    let token = ndMd5Hash(password + salt)
    let urlStr = "\(baseUrl)/rest/ping.view?u=\(user)&t=\(token)&s=\(salt)&v=1.16.1&c=rockbox&f=json"
    guard let url = URL(string: urlStr),
          let (data, _) = try? await URLSession.shared.data(from: url),
          let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
          let inner = json["subsonic-response"] as? [String: Any]
    else { return false }
    return inner["status"] as? String == "ok"
}

func ndGetAlbums(server: NdServer) async throws -> [NdAlbum] {
    let inner = try await ndCall(server: server, endpoint: "getAlbumList2", params: ["type": "alphabeticalByName", "size": "500"])
    guard let list = inner["albumList2"] as? [String: Any],
          let albums = list["album"] as? [[String: Any]]
    else { return [] }
    return albums.compactMap { NdAlbum(from: $0) }
}

func ndGetArtists(server: NdServer) async throws -> [NdArtist] {
    let inner = try await ndCall(server: server, endpoint: "getArtists")
    guard let wrapper = inner["artists"] as? [String: Any],
          let indices = wrapper["index"] as? [[String: Any]]
    else { return [] }
    return indices.flatMap { idx -> [NdArtist] in
        let artists = idx["artist"] as? [[String: Any]] ?? []
        return artists.compactMap { NdArtist(from: $0) }
    }
}

struct NdAlbumDetail {
    let album: NdAlbum
    let songs: [NdSong]
}

func ndGetAlbum(server: NdServer, albumId: String) async throws -> NdAlbumDetail {
    let inner = try await ndCall(server: server, endpoint: "getAlbum", params: ["id": albumId])
    guard let albumDict = inner["album"] as? [String: Any],
          let album = NdAlbum(from: albumDict)
    else { throw URLError(.badServerResponse) }
    let songs = (albumDict["song"] as? [[String: Any]] ?? []).compactMap { NdSong(from: $0, server: server) }
    return NdAlbumDetail(album: album, songs: songs)
}

struct NdArtistDetail {
    let artist: NdArtist
    let albums: [NdAlbum]
}

func ndGetArtist(server: NdServer, artistId: String) async throws -> NdArtistDetail {
    let inner = try await ndCall(server: server, endpoint: "getArtist", params: ["id": artistId])
    guard let artistDict = inner["artist"] as? [String: Any],
          let artist = NdArtist(from: artistDict)
    else { throw URLError(.badServerResponse) }
    let albums = (artistDict["album"] as? [[String: Any]] ?? []).compactMap { NdAlbum(from: $0) }
    return NdArtistDetail(artist: artist, albums: albums)
}

func ndGetSongs(server: NdServer, count: Int = 100, offset: Int = 0) async throws -> [NdSong] {
    let inner = try await ndCall(server: server, endpoint: "search3", params: [
        "query": "",
        "songCount": String(count),
        "songOffset": String(offset),
        "albumCount": "0",
        "artistCount": "0",
    ])
    guard let results = inner["searchResult3"] as? [String: Any],
          let songs = results["song"] as? [[String: Any]]
    else { return [] }
    return songs.compactMap { NdSong(from: $0, server: server) }
        .sorted { $0.title.localizedCaseInsensitiveCompare($1.title) == .orderedAscending }
}

func ndGetStarred(server: NdServer) async throws -> [NdSong] {
    let inner = try await ndCall(server: server, endpoint: "getStarred2")
    guard let starred = inner["starred2"] as? [String: Any],
          let songs = starred["song"] as? [[String: Any]]
    else { return [] }
    return songs.compactMap { NdSong(from: $0, server: server) }
}

struct NdPlaylistDetail {
    let playlist: NdPlaylist
    let songs: [NdSong]
}

func ndGetPlaylists(server: NdServer) async throws -> [NdPlaylist] {
    let inner = try await ndCall(server: server, endpoint: "getPlaylists")
    guard let wrapper = inner["playlists"] as? [String: Any],
          let playlists = wrapper["playlist"] as? [[String: Any]]
    else { return [] }
    return playlists.compactMap { NdPlaylist(from: $0) }
}

func ndGetPlaylist(server: NdServer, playlistId: String) async throws -> NdPlaylistDetail {
    let inner = try await ndCall(server: server, endpoint: "getPlaylist", params: ["id": playlistId])
    guard let plDict = inner["playlist"] as? [String: Any],
          let playlist = NdPlaylist(from: plDict)
    else { throw URLError(.badServerResponse) }
    let songs = (plDict["entry"] as? [[String: Any]] ?? []).compactMap { NdSong(from: $0, server: server) }
    return NdPlaylistDetail(playlist: playlist, songs: songs)
}

struct NdSearchResults {
    var songs: [NdSong] = []
    var albums: [NdAlbum] = []
    var artists: [NdArtist] = []
    var isEmpty: Bool { songs.isEmpty && albums.isEmpty && artists.isEmpty }
}

func ndSearch(server: NdServer, query: String) async throws -> NdSearchResults {
    let inner = try await ndCall(server: server, endpoint: "search3", params: [
        "query": query,
        "songCount": "10",
        "albumCount": "10",
        "artistCount": "5",
    ])
    guard let results = inner["searchResult3"] as? [String: Any] else { return NdSearchResults() }
    let songs = (results["song"] as? [[String: Any]] ?? []).compactMap { NdSong(from: $0, server: server) }
    let albums = (results["album"] as? [[String: Any]] ?? []).compactMap { NdAlbum(from: $0) }
    let artists = (results["artist"] as? [[String: Any]] ?? []).compactMap { NdArtist(from: $0) }
    return NdSearchResults(songs: songs, albums: albums, artists: artists)
}

func ndStar(server: NdServer, songId: String) async throws {
    _ = try await ndCall(server: server, endpoint: "star", params: ["id": songId])
}

func ndUnstar(server: NdServer, songId: String) async throws {
    _ = try await ndCall(server: server, endpoint: "unstar", params: ["id": songId])
}

func ndScrobble(server: NdServer, songId: String) async {
    _ = try? await ndCall(server: server, endpoint: "scrobble", params: ["id": songId, "submission": "true"])
}
