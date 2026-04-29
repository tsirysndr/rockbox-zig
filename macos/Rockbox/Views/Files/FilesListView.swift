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
}

struct FilesListView: View {
    @State private var files: [FileItem] = []
    @State private var errorText: String?
    @State private var currentPath: String? = nil
    @State private var mode: FilesMode = .root
    @State private var history: [(FilesMode, String?)] = []

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
            } else {
                fileListView
            }
        }
        .task(id: currentPath) {
            guard mode != .root else { return }
            await loadFiles()
        }
        .alert("gRPC Error", isPresented: .constant(errorText != nil)) {
            Button("OK") { errorText = nil }
        } message: {
            Text(errorText ?? "")
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
        }
    }

    private func navigate(to newMode: FilesMode, path: String?) {
        history.append((mode, currentPath))
        mode = newMode
        currentPath = path
    }

    private func navigateTo(file: FileItem) {
        let newMode: FilesMode = file.path.hasPrefix("upnp://") ? .upnpBrowse : .local
        navigate(to: newMode, path: file.path)
    }

    private func goBack() {
        guard let (prevMode, prevPath) = history.popLast() else { return }
        mode = prevMode
        currentPath = prevPath
        if mode == .root { files = [] }
    }

    // MARK: - Data loading

    private func loadFiles() async {
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
