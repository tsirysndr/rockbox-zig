import Foundation

@MainActor
class NavidromeManager: ObservableObject {
    static let shared = NavidromeManager()

    @Published var servers: [NdServer] = []
    @Published var activeId: String? = nil
    @Published var starredIds: Set<String> = []

    var activeServer: NdServer? {
        servers.first { $0.id == activeId }
    }

    private var saveURL: URL? {
        guard let support = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first
        else { return nil }
        let dir = support.appendingPathComponent("Rockbox", isDirectory: true)
        try? FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        return dir.appendingPathComponent("navidrome_servers.json")
    }

    private init() {
        load()
    }

    func addServer(label: String, baseUrl: String, user: String, password: String) {
        let salt = ndGenerateSalt()
        let token = ndMd5Hash(password + salt)
        let server = NdServer(
            id: UUID().uuidString,
            label: label,
            baseUrl: baseUrl,
            user: user,
            password: password,
            authToken: token,
            authSalt: salt
        )
        servers.append(server)
        if activeId == nil { activeId = server.id }
        save()
        Task { await loadStarred() }
    }

    func removeServer(id: String) {
        servers.removeAll { $0.id == id }
        if activeId == id { activeId = servers.first?.id }
        save()
        if activeServer == nil { starredIds = [] }
        Task { await NdResponseCache.shared.invalidate(prefix: id) }
    }

    func setActive(id: String) {
        activeId = id
        save()
        Task { await loadStarred() }
    }

    func toggleStar(songId: String) {
        guard let server = activeServer else { return }
        if starredIds.contains(songId) {
            starredIds.remove(songId)
            Task { try? await ndUnstar(server: server, songId: songId) }
        } else {
            starredIds.insert(songId)
            Task { try? await ndStar(server: server, songId: songId) }
        }
    }

    func coverArtUrl(forStreamUrl streamUrl: String, size: Int = 300) -> URL? {
        guard let server = servers.first(where: { streamUrl.hasPrefix($0.baseUrl) }),
              let comps = URLComponents(string: streamUrl),
              let songId = comps.queryItems?.first(where: { $0.name == "id" })?.value
        else { return nil }
        return server.coverArtUrl(coverId: songId, size: size)
    }

    func loadStarred() async {
        guard let server = activeServer else { return }
        let songs = (try? await ndGetStarred(server: server)) ?? []
        starredIds = Set(songs.map { $0.id })
    }

    private func save() {
        guard let url = saveURL else { return }
        struct Payload: Codable {
            var servers: [NdServer]
            var activeId: String?
        }
        if let data = try? JSONEncoder().encode(Payload(servers: servers, activeId: activeId)) {
            try? data.write(to: url)
        }
    }

    private func load() {
        guard let url = saveURL,
              let data = try? Data(contentsOf: url)
        else { return }
        struct Payload: Codable {
            var servers: [NdServer]
            var activeId: String?
        }
        if let payload = try? JSONDecoder().decode(Payload.self, from: data) {
            servers = payload.servers
            activeId = payload.activeId
            Task { await loadStarred() }
        }
    }
}
