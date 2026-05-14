//
//  FilesListView.swift
//  Rockbox
//

import CryptoKit
import Foundation
import SwiftUI

enum FilesMode {
    case root
    case local
    case upnpDevices
    case upnpBrowse
    case plexServers
    case plexBrowse
    case jellyfinServers
    case jellyfinBrowse
    case navidromeServers
    case navidromeBrowse
    case kodiServers
    case kodiBrowse
}

enum JellyfinAuthMode {
    case credentials
    case apiKey
}

struct FilesListView: View {
    @State private var files: [FileItem] = []
    @State private var errorText: String?
    @State private var isLoading = false
    @State private var currentPath: String? = nil
    @State private var mode: FilesMode = .root
    @State private var history: [(FilesMode, String?)] = []
    @AppStorage("plex_token") private var plexToken: String = ""
    @State private var pendingPlexServer: String? = nil
    @State private var pendingJellyfinServer: String? = nil
    @State private var jellyfinUsername = ""
    @State private var jellyfinPassword = ""
    @State private var jellyfinError: String? = nil
    @State private var jellyfinSigningIn = false
    @State private var jellyfinAuthMode: JellyfinAuthMode = .credentials
    @State private var jellyfinApiKey = ""
    @State private var showJellyfinManualEntry = false
    @State private var jellyfinManualUrl = ""
    @State private var showNavidromeEntry = false
    @State private var navidromeUrl = ""
    @State private var navidromeUsername = ""
    @State private var navidromePassword = ""
    @State private var navidromeError: String? = nil
    @State private var navidromeConnecting = false
    @State private var showKodiEntry = false
    @State private var kodiUrl = ""
    @State private var kodiUsername = ""
    @State private var kodiPassword = ""
    @State private var kodiError: String? = nil
    @State private var kodiConnecting = false

    var body: some View {
        VStack(spacing: 0) {
            HStack {
                Button(action: goBack) {
                    Image(systemName: "chevron.left")
                        .font(.system(size: 14, weight: .medium))
                        .frame(minHeight: 30)
                        .frame(minWidth: 30)
                        .contentShape(Rectangle())
                }
                .buttonStyle(.plain)
                .disabled(history.isEmpty)
                .opacity(history.isEmpty ? 0.3 : 1)

                Text(pathDisplay)
                    .font(.system(size: 13, weight: .medium))
                    .lineLimit(1)

                Spacer()
            }
            .padding(.horizontal, 16)
            .padding(.vertical, 10)

            Divider()

            if mode == .root {
                rootView
            } else if isLoading {
                ProgressView()
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                fileListView
            }
        }
        .task(id: "\(mode)-\(currentPath ?? "")") {
            guard mode != .root else { return }
            await loadFiles()
        }
        .alert("gRPC Error", isPresented: .constant(errorText != nil)) {
            Button("OK") { errorText = nil }
        } message: {
            Text(errorText ?? "")
        }
        .sheet(isPresented: Binding(
            get: { pendingPlexServer != nil },
            set: { if !$0 { pendingPlexServer = nil } }
        )) {
            VStack(alignment: .leading, spacing: 12) {
                Text("Plex Token")
                    .font(.headline)
                Text("Required for private servers. Leave blank for public ones.")
                    .font(.caption)
                    .foregroundColor(.secondary)
                SecureField("paste your X-Plex-Token here", text: $plexToken)
                    .textFieldStyle(.roundedBorder)
                HStack {
                    Spacer()
                    Button("Cancel") { pendingPlexServer = nil }
                        .keyboardShortcut(.cancelAction)
                    Button("Connect") { connectToPlexServer() }
                        .keyboardShortcut(.defaultAction)
                        .buttonStyle(.borderedProminent)
                }
            }
            .padding(24)
            .frame(width: 340)
        }
        .sheet(isPresented: Binding(
            get: { pendingJellyfinServer != nil },
            set: { if !$0 { pendingJellyfinServer = nil } }
        )) {
            VStack(alignment: .leading, spacing: 12) {
                Text("Sign in to Jellyfin")
                    .font(.headline)
                // Auth mode picker
                Picker("", selection: $jellyfinAuthMode) {
                    Text("Credentials").tag(JellyfinAuthMode.credentials)
                    Text("API Key").tag(JellyfinAuthMode.apiKey)
                }
                .pickerStyle(.segmented)
                if jellyfinAuthMode == .credentials {
                    TextField("Username", text: $jellyfinUsername)
                        .textFieldStyle(.roundedBorder)
                        .disabled(jellyfinSigningIn)
                    SecureField("Password", text: $jellyfinPassword)
                        .textFieldStyle(.roundedBorder)
                        .disabled(jellyfinSigningIn)
                } else {
                    SecureField("API Key", text: $jellyfinApiKey)
                        .textFieldStyle(.roundedBorder)
                        .disabled(jellyfinSigningIn)
                }
                if let err = jellyfinError {
                    Text(err)
                        .font(.caption)
                        .foregroundColor(.red)
                }
                HStack {
                    Spacer()
                    Button("Cancel") {
                        pendingJellyfinServer = nil
                        jellyfinError = nil
                    }
                    .keyboardShortcut(.cancelAction)
                    .disabled(jellyfinSigningIn)
                    Button(jellyfinSigningIn ? "Signing in…" : "Sign in") {
                        Task {
                            if jellyfinAuthMode == .credentials {
                                await connectToJellyfinServer()
                            } else {
                                await connectToJellyfinWithApiKey()
                            }
                        }
                    }
                    .keyboardShortcut(.defaultAction)
                    .buttonStyle(.borderedProminent)
                    .disabled(jellyfinSigningIn)
                }
            }
            .padding(24)
            .frame(width: 340)
        }
        .sheet(isPresented: $showJellyfinManualEntry) {
            VStack(alignment: .leading, spacing: 12) {
                Text("Add Jellyfin Server")
                    .font(.headline)
                Text("Enter the server URL (e.g. http://192.168.1.10:8096)")
                    .font(.caption)
                    .foregroundColor(.secondary)
                TextField("http://192.168.1.x:8096", text: $jellyfinManualUrl)
                    .textFieldStyle(.roundedBorder)
                HStack {
                    Spacer()
                    Button("Cancel") { showJellyfinManualEntry = false }
                        .keyboardShortcut(.cancelAction)
                    Button("Connect") {
                        let url = jellyfinManualUrl.trimmingCharacters(in: .whitespaces)
                        guard !url.isEmpty,
                              let encoded = url.addingPercentEncoding(withAllowedCharacters: .urlQueryAllowed)
                        else { return }
                        showJellyfinManualEntry = false
                        jellyfinManualUrl = ""
                        jellyfinUsername = ""
                        jellyfinPassword = ""
                        jellyfinError = nil
                        jellyfinAuthMode = .credentials
                        pendingJellyfinServer = "jellyfin://\(encoded)"
                    }
                    .keyboardShortcut(.defaultAction)
                    .buttonStyle(.borderedProminent)
                }
            }
            .padding(24)
            .frame(width: 360)
        }
        .sheet(isPresented: $showNavidromeEntry) {
            VStack(alignment: .leading, spacing: 12) {
                Text("Connect to Navidrome")
                    .font(.headline)
                Text("Enter your server URL, username, and password.")
                    .font(.caption)
                    .foregroundColor(.secondary)
                TextField("http://192.168.1.x:4533", text: $navidromeUrl)
                    .textFieldStyle(.roundedBorder)
                    .disabled(navidromeConnecting)
                TextField("Username", text: $navidromeUsername)
                    .textFieldStyle(.roundedBorder)
                    .disabled(navidromeConnecting)
                SecureField("Password", text: $navidromePassword)
                    .textFieldStyle(.roundedBorder)
                    .disabled(navidromeConnecting)
                if let err = navidromeError {
                    Text(err)
                        .font(.caption)
                        .foregroundColor(.red)
                }
                HStack {
                    Spacer()
                    Button("Cancel") {
                        showNavidromeEntry = false
                        navidromeError = nil
                    }
                    .keyboardShortcut(.cancelAction)
                    .disabled(navidromeConnecting)
                    Button(navidromeConnecting ? "Connecting…" : "Connect") {
                        Task { await connectToNavidrome() }
                    }
                    .keyboardShortcut(.defaultAction)
                    .buttonStyle(.borderedProminent)
                    .disabled(navidromeConnecting)
                }
            }
            .padding(24)
            .frame(width: 360)
        }
        .sheet(isPresented: $showKodiEntry) {
            VStack(alignment: .leading, spacing: 12) {
                Text("Connect to Kodi")
                    .font(.headline)
                Text("Username and password are optional for local servers.")
                    .font(.caption)
                    .foregroundColor(.secondary)
                TextField("http://192.168.1.x:8080", text: $kodiUrl)
                    .textFieldStyle(.roundedBorder)
                    .disabled(kodiConnecting)
                TextField("Username (optional)", text: $kodiUsername)
                    .textFieldStyle(.roundedBorder)
                    .disabled(kodiConnecting)
                SecureField("Password (optional)", text: $kodiPassword)
                    .textFieldStyle(.roundedBorder)
                    .disabled(kodiConnecting)
                if let err = kodiError {
                    Text(err)
                        .font(.caption)
                        .foregroundColor(.red)
                }
                HStack {
                    Spacer()
                    Button("Cancel") {
                        showKodiEntry = false
                        kodiError = nil
                    }
                    .keyboardShortcut(.cancelAction)
                    .disabled(kodiConnecting)
                    Button(kodiConnecting ? "Connecting…" : "Connect") {
                        Task { await connectToKodi() }
                    }
                    .keyboardShortcut(.defaultAction)
                    .buttonStyle(.borderedProminent)
                    .disabled(kodiConnecting)
                }
            }
            .padding(24)
            .frame(width: 360)
        }
    }

    // MARK: - Root landing

    private var rootView: some View {
        ScrollView {
            LazyVStack(spacing: 0) {
                rootRow(name: "Music", systemImage: "folder") {
                    navigate(to: .local, path: nil)
                }
                rootRow(name: "UPnP Devices", systemImage: "network") {
                    navigate(to: .upnpDevices, path: "upnp://")
                }
                rootRow(name: "Plex", systemImage: "play.rectangle") {
                    navigate(to: .plexServers, path: "plex://")
                }
                rootRow(name: "Jellyfin", systemImage: "", imageName: "jellyfin") {
                    navigate(to: .jellyfinServers, path: "jellyfin://")
                }
                rootRow(name: "Navidrome", systemImage: "", imageName: "navidrome") {
                    navigate(to: .navidromeServers, path: "navidrome://")
                    showNavidromeEntry = true
                }
                rootRow(name: "Kodi", systemImage: "", imageName: "kodi") {
                    navigate(to: .kodiServers, path: "kodi://")
                }
            }
        }
    }

    private func rootRow(name: String, systemImage: String, imageName: String? = nil, action: @escaping () -> Void) -> some View {
        Button(action: action) {
            HStack(spacing: 12) {
                if let imageName = imageName {
                    Image(imageName)
                        .resizable()
                        .scaledToFit()
                        .frame(width: 20, height: 20)
                } else {
                    Image(systemName: systemImage)
                        .frame(width: 20, height: 20)
                }
                Text(name)
                    .font(.system(size: 13))
                Spacer()
                Image(systemName: "chevron.right")
                    .font(.system(size: 11))
                    .foregroundColor(.secondary)
            }
            .padding(.horizontal, 16)
            .padding(.vertical, 10)
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
    }

    // MARK: - File list

    private var fileListView: some View {
        ScrollView {
            LazyVStack(spacing: 0) {
                FileHeaderRow()
                Divider()
                ForEach(Array(files.enumerated()), id: \.element.id) { index, file in
                    FileRowView(
                        file: file,
                        isEven: index % 2 == 0,
                        selectedIndex: index,
                        currentDirectory: currentPath ?? ""
                    )
                    .contentShape(Rectangle())
                    .onTapGesture {
                        if file.type == .directory {
                            navigateTo(file: file)
                        }
                    }
                }
                if mode == .jellyfinServers {
                    Divider()
                    Button(action: { showJellyfinManualEntry = true }) {
                        HStack(spacing: 12) {
                            Image(systemName: "plus.circle")
                                .frame(width: 20, height: 20)
                            Text("Add Server Manually…")
                                .font(.system(size: 13))
                            Spacer()
                        }
                        .padding(.horizontal, 16)
                        .padding(.vertical, 10)
                        .contentShape(Rectangle())
                    }
                    .buttonStyle(.plain)
                    .foregroundColor(.secondary)
                }
                if mode == .navidromeServers {
                    Button(action: { showNavidromeEntry = true }) {
                        HStack(spacing: 12) {
                            Image(systemName: "plus.circle")
                                .frame(width: 20, height: 20)
                            Text("Connect to Navidrome…")
                                .font(.system(size: 13))
                            Spacer()
                        }
                        .padding(.horizontal, 16)
                        .padding(.vertical, 10)
                        .contentShape(Rectangle())
                    }
                    .buttonStyle(.plain)
                    .foregroundColor(.secondary)
                }
                if mode == .kodiServers {
                    Button(action: { showKodiEntry = true }) {
                        HStack(spacing: 12) {
                            Image(systemName: "plus.circle")
                                .frame(width: 20, height: 20)
                            Text("Add Manually…")
                                .font(.system(size: 13))
                            Spacer()
                        }
                        .padding(.horizontal, 16)
                        .padding(.vertical, 10)
                        .contentShape(Rectangle())
                    }
                    .buttonStyle(.plain)
                    .foregroundColor(.secondary)
                }
            }
        }
    }

    // MARK: - Navigation

    private var pathDisplay: String {
        switch mode {
        case .root: return "Files"
        case .local:
            guard let path = currentPath else { return "Music" }
            return URL(fileURLWithPath: path).lastPathComponent
        case .upnpDevices: return "UPnP Devices"
        case .upnpBrowse:
            return currentPath?.split(separator: "/").last.map(String.init) ?? "UPnP"
        case .plexServers: return "Plex Servers"
        case .plexBrowse:
            return currentPath?.split(separator: "/").last.map(String.init) ?? "Plex"
        case .jellyfinServers: return "Jellyfin Servers"
        case .jellyfinBrowse:
            return currentPath?.split(separator: "/").last.map(String.init) ?? "Jellyfin"
        case .navidromeServers: return "Navidrome"
        case .navidromeBrowse:
            return currentPath?.split(separator: "/").last.map(String.init) ?? "Navidrome"
        case .kodiServers: return "Kodi Servers"
        case .kodiBrowse:
            return currentPath?.split(separator: "/").last.map(String.init) ?? "Kodi"
        }
    }

    private func navigate(to newMode: FilesMode, path: String?) {
        history.append((mode, currentPath))
        mode = newMode
        currentPath = path
    }

    private func isPlexServerPath(_ path: String) -> Bool {
        guard path.hasPrefix("plex://") else { return false }
        let rest = path.dropFirst("plex://".count)
        return !rest.isEmpty && !rest.contains("/")
    }

    private func isJellyfinServerPath(_ path: String) -> Bool {
        guard path.hasPrefix("jellyfin://") else { return false }
        let rest = path.dropFirst("jellyfin://".count)
        return !rest.isEmpty && !rest.contains("/")
    }

    private func isKodiServerPath(_ path: String) -> Bool {
        guard path.hasPrefix("kodi://") else { return false }
        let rest = path.dropFirst("kodi://".count)
        return !rest.isEmpty && !rest.contains("/")
    }

    private func navigateTo(file: FileItem) {
        if file.path.hasPrefix("upnp://") {
            navigate(to: .upnpBrowse, path: file.path)
        } else if isPlexServerPath(file.path) {
            // Show token prompt before browsing the server.
            pendingPlexServer = file.path
        } else if file.path.hasPrefix("plex://") {
            navigate(to: .plexBrowse, path: file.path)
        } else if isJellyfinServerPath(file.path) {
            // Show auth prompt before browsing the server.
            jellyfinUsername = ""
            jellyfinPassword = ""
            jellyfinError = nil
            pendingJellyfinServer = file.path
        } else if file.path.hasPrefix("jellyfin://") {
            navigate(to: .jellyfinBrowse, path: file.path)
        } else if file.path.hasPrefix("navidrome://") {
            navigate(to: .navidromeBrowse, path: file.path)
        } else if isKodiServerPath(file.path) {
            // Show connect sheet with URL pre-filled from the decoded base URL.
            let encoded = file.path.dropFirst("kodi://".count)
            if let decoded = encoded.removingPercentEncoding {
                // Strip any existing kodi_user/kodi_pass query params
                let rawBase = decoded.components(separatedBy: "?").first ?? decoded
                kodiUrl = rawBase
            }
            kodiError = nil
            showKodiEntry = true
        } else if file.path.hasPrefix("kodi://") {
            navigate(to: .kodiBrowse, path: file.path)
        } else {
            navigate(to: .local, path: file.path)
        }
    }

    private func connectToPlexServer() {
        guard let server = pendingPlexServer else { return }
        let navPath = plexToken.isEmpty
            ? server
            : "\(server)%3FX-Plex-Token%3D\(plexToken)"
        pendingPlexServer = nil
        navigate(to: .plexBrowse, path: navPath)
    }

    private func connectToJellyfinServer() async {
        guard let server = pendingJellyfinServer else { return }
        // Decode the base URL from the jellyfin:// path
        let encoded = server.dropFirst("jellyfin://".count)
        guard let baseUrl = encoded.removingPercentEncoding else {
            jellyfinError = "Invalid server URL."
            return
        }
        let urlStr = "\(baseUrl.hasSuffix("/") ? String(baseUrl.dropLast()) : baseUrl)/Users/AuthenticateByName"
        guard let url = URL(string: urlStr) else {
            jellyfinError = "Could not build request URL."
            return
        }

        jellyfinSigningIn = true
        jellyfinError = nil
        defer { jellyfinSigningIn = false }

        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.setValue(
            #"MediaBrowser Client="Rockbox", Device="macOS", DeviceId="rockbox-macos", Version="1.0""#,
            forHTTPHeaderField: "X-Emby-Authorization"
        )
        let body: [String: String] = ["Username": jellyfinUsername, "Pw": jellyfinPassword]
        request.httpBody = try? JSONSerialization.data(withJSONObject: body)

        do {
            let (data, response) = try await URLSession.shared.data(for: request)
            guard let httpResponse = response as? HTTPURLResponse, httpResponse.statusCode == 200 else {
                jellyfinError = "Authentication failed. Check username/password."
                return
            }
            guard
                let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
                let token = json["AccessToken"] as? String,
                let user = json["User"] as? [String: Any],
                let userId = user["Id"] as? String
            else {
                jellyfinError = "Unexpected response from server."
                return
            }
            // Build the navigation path with token embedded
            let tokenEncoded = token.addingPercentEncoding(withAllowedCharacters: .urlQueryAllowed) ?? token
            let userIdEncoded = userId.addingPercentEncoding(withAllowedCharacters: .urlQueryAllowed) ?? userId
            let navPath = "\(server)%3FX-Jellyfin-Token%3D\(tokenEncoded)%26userId%3D\(userIdEncoded)"
            pendingJellyfinServer = nil
            navigate(to: .jellyfinBrowse, path: navPath)
        } catch {
            jellyfinError = "Network error: \(error.localizedDescription)"
        }
    }

    private func connectToJellyfinWithApiKey() async {
        guard let server = pendingJellyfinServer else { return }
        let encoded = server.dropFirst("jellyfin://".count)
        guard let baseUrl = encoded.removingPercentEncoding else {
            jellyfinError = "Invalid server URL."
            return
        }
        let urlStr = "\(baseUrl.hasSuffix("/") ? String(baseUrl.dropLast()) : baseUrl)/Users"
        guard let url = URL(string: urlStr) else {
            jellyfinError = "Could not build request URL."
            return
        }

        jellyfinSigningIn = true
        jellyfinError = nil
        defer { jellyfinSigningIn = false }

        var request = URLRequest(url: url)
        request.httpMethod = "GET"
        request.setValue(jellyfinApiKey, forHTTPHeaderField: "X-Emby-Token")

        do {
            let (data, response) = try await URLSession.shared.data(for: request)
            guard let httpResponse = response as? HTTPURLResponse, httpResponse.statusCode == 200 else {
                jellyfinError = "Invalid API key or insufficient permissions."
                return
            }
            guard
                let users = try? JSONSerialization.jsonObject(with: data) as? [[String: Any]],
                let firstUser = users.first,
                let userId = firstUser["Id"] as? String
            else {
                jellyfinError = "No users found for this API key."
                return
            }
            let tokenEncoded = jellyfinApiKey.addingPercentEncoding(withAllowedCharacters: .urlQueryAllowed) ?? jellyfinApiKey
            let userIdEncoded = userId.addingPercentEncoding(withAllowedCharacters: .urlQueryAllowed) ?? userId
            let navPath = "\(server)%3FX-Jellyfin-Token%3D\(tokenEncoded)%26userId%3D\(userIdEncoded)"
            pendingJellyfinServer = nil
            navigate(to: .jellyfinBrowse, path: navPath)
        } catch {
            jellyfinError = "Network error: \(error.localizedDescription)"
        }
    }

    private func connectToNavidrome() async {
        let baseUrl = navidromeUrl.trimmingCharacters(in: .whitespaces)
        let username = navidromeUsername.trimmingCharacters(in: .whitespaces)
        guard !baseUrl.isEmpty, !username.isEmpty else {
            navidromeError = "Server URL and username are required."
            return
        }
        let trimmedBase = baseUrl.hasSuffix("/") ? String(baseUrl.dropLast()) : baseUrl
        let salt = generateSalt()
        let token = md5Hash(navidromePassword + salt)
        let pingUrlStr = "\(trimmedBase)/rest/ping.view?u=\(username)&t=\(token)&s=\(salt)&v=1.16.1&c=rockbox&f=json"
        guard let pingUrl = URL(string: pingUrlStr) else {
            navidromeError = "Invalid server URL."
            return
        }

        navidromeConnecting = true
        navidromeError = nil
        defer { navidromeConnecting = false }

        do {
            let (data, response) = try await URLSession.shared.data(from: pingUrl)
            guard let httpResponse = response as? HTTPURLResponse, httpResponse.statusCode == 200 else {
                navidromeError = "Server returned an error. Check the URL."
                return
            }
            guard
                let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
                let inner = json["subsonic-response"] as? [String: Any],
                inner["status"] as? String == "ok"
            else {
                navidromeError = "Authentication failed. Check username and password."
                return
            }
            let authSuffix = "?nd_user=\(username)&nd_token=\(token)&nd_salt=\(salt)"
            guard let encoded = "\(trimmedBase)\(authSuffix)"
                .addingPercentEncoding(withAllowedCharacters: .urlQueryAllowed)
            else {
                navidromeError = "Failed to encode server URL."
                return
            }
            let navPath = "navidrome://\(encoded)"
            showNavidromeEntry = false
            navigate(to: .navidromeBrowse, path: navPath)
        } catch {
            navidromeError = "Network error: \(error.localizedDescription)"
        }
    }

    private func connectToKodi() async {
        let baseUrl = kodiUrl.trimmingCharacters(in: .whitespacesAndNewlines)
            .replacingOccurrences(of: "/$", with: "", options: .regularExpression)
        guard !baseUrl.isEmpty else {
            kodiError = "Server URL is required."
            return
        }
        guard let url = URL(string: "\(baseUrl)/jsonrpc") else {
            kodiError = "Invalid server URL."
            return
        }

        kodiConnecting = true
        kodiError = nil
        defer { kodiConnecting = false }

        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        if !kodiUsername.trimmingCharacters(in: .whitespaces).isEmpty {
            let credentials = "\(kodiUsername):\(kodiPassword)"
            if let data = credentials.data(using: .utf8) {
                let encoded = data.base64EncodedString()
                request.setValue("Basic \(encoded)", forHTTPHeaderField: "Authorization")
            }
        }
        request.httpBody = try? JSONSerialization.data(
            withJSONObject: ["jsonrpc": "2.0", "method": "JSONRPC.Ping", "id": 1]
        )

        do {
            let (data, response) = try await URLSession.shared.data(for: request)
            guard let httpResponse = response as? HTTPURLResponse, httpResponse.statusCode == 200 else {
                kodiError = "HTTP error. Check the URL and credentials."
                return
            }
            guard
                let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
                json["result"] as? String == "pong"
            else {
                kodiError = "Kodi did not respond correctly. Check the URL."
                return
            }
            let trimmedUser = kodiUsername.trimmingCharacters(in: .whitespaces)
            let credSuffix = trimmedUser.isEmpty
                ? ""
                : "?kodi_user=\(trimmedUser)&kodi_pass=\(kodiPassword)"
            let credUrl = "\(baseUrl)\(credSuffix)"
            guard let encoded = credUrl.addingPercentEncoding(withAllowedCharacters: .urlQueryAllowed) else {
                kodiError = "Failed to encode server URL."
                return
            }
            let navPath = "kodi://\(encoded)"
            showKodiEntry = false
            navigate(to: .kodiBrowse, path: navPath)
        } catch {
            kodiError = "Network error: \(error.localizedDescription)"
        }
    }

    private func generateSalt() -> String {
        let chars = "abcdefghijklmnopqrstuvwxyz0123456789"
        return String((0..<8).compactMap { _ in chars.randomElement() })
    }

    private func md5Hash(_ input: String) -> String {
        let digest = Insecure.MD5.hash(data: Data(input.utf8))
        return digest.map { String(format: "%02x", $0) }.joined()
    }

    private func goBack() {
        guard let (prevMode, prevPath) = history.popLast() else { return }
        mode = prevMode
        currentPath = prevPath
        if mode == .root { files = [] }
    }

    // MARK: - Data loading

    private func loadFiles() async {
        isLoading = true
        defer { isLoading = false }
        do {
            let entries = try await fetchFiles(path: currentPath)
            files = entries.compactMap { entry -> FileItem? in
                let isDir = entry.attr == 16
                let displayName: String
                if entry.hasDisplayName {
                    displayName = entry.displayName
                } else if entry.name.hasPrefix("upnp://") {
                    displayName = entry.name
                } else {
                    displayName = URL(fileURLWithPath: entry.name).lastPathComponent
                }
                return FileItem(
                    name: displayName,
                    path: entry.name,
                    type: isDir ? .directory : .audioFile,
                    size: nil,
                    itemCount: nil
                )
            }
            .sorted { a, b in
                if a.type == b.type {
                    return a.name.localizedCaseInsensitiveCompare(b.name) == .orderedAscending
                }
                return a.type == .directory
            }
        } catch {
            errorText = String(describing: error)
        }
    }
}
