//
//  FilesListView.swift
//  Rockbox
//

import Foundation
import SwiftUI

enum FilesMode {
    case root
    case local
    case upnpDevices
    case upnpBrowse
    case plexServers
    case plexBrowse
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
            }
        }
    }

    private func rootRow(name: String, systemImage: String, action: @escaping () -> Void) -> some View {
        Button(action: action) {
            HStack(spacing: 12) {
                Image(systemName: systemImage)
                    .frame(width: 20, height: 20)
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

    private func navigateTo(file: FileItem) {
        if file.path.hasPrefix("upnp://") {
            navigate(to: .upnpBrowse, path: file.path)
        } else if isPlexServerPath(file.path) {
            // Show token prompt before browsing the server.
            pendingPlexServer = file.path
        } else if file.path.hasPrefix("plex://") {
            navigate(to: .plexBrowse, path: file.path)
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
