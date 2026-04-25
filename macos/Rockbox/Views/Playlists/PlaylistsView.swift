//
//  PlaylistsView.swift
//  Rockbox
//

import SwiftUI

struct PlaylistsView: View {
    @EnvironmentObject var navigation: NavigationManager
    @EnvironmentObject var player: PlayerState

    @State private var savedPlaylists: [SavedPlaylist] = []
    @State private var smartPlaylists: [SmartPlaylist] = []
    @State private var errorText: String?
    @State private var showCreateSheet = false
    @State private var newPlaylistName = ""
    @State private var isCreating = false

    private let columns = [GridItem(.adaptive(minimum: 170, maximum: 230), spacing: 20)]

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 24) {
                if !smartPlaylists.isEmpty {
                    VStack(alignment: .leading, spacing: 12) {
                        Text("Smart Playlists")
                            .font(.system(size: 13, weight: .semibold))
                            .foregroundStyle(.secondary)
                            .padding(.horizontal, 20)

                        LazyVGrid(columns: columns, spacing: 24) {
                            ForEach(smartPlaylists) { pl in
                                SmartPlaylistCardView(
                                    playlist: pl,
                                    onSelect: { navigation.goToSmartPlaylist(pl) },
                                    onPlay: {
                                        Task {
                                            do {
                                                try await playSmartPlaylist(id: pl.id)
                                                await player.fetchQueue()
                                            } catch {
                                                errorText = String(describing: error)
                                            }
                                        }
                                    }
                                )
                            }
                        }
                        .padding(.horizontal, 20)
                    }
                }

                VStack(alignment: .leading, spacing: 12) {
                    HStack {
                        Text("Saved Playlists")
                            .font(.system(size: 13, weight: .semibold))
                            .foregroundStyle(.secondary)
                        Spacer()
                        Button(action: { showCreateSheet = true }) {
                            Label("New Playlist", systemImage: "plus")
                                .font(.system(size: 12))
                        }
                        .buttonStyle(.bordered)
                    }
                    .padding(.horizontal, 20)

                    if savedPlaylists.isEmpty {
                        VStack(spacing: 8) {
                            Image(systemName: "music.note.list")
                                .font(.system(size: 36))
                                .foregroundStyle(.secondary)
                            Text("No playlists yet")
                                .font(.system(size: 13))
                                .foregroundStyle(.secondary)
                        }
                        .frame(maxWidth: .infinity)
                        .padding(.top, 40)
                    } else {
                        LazyVGrid(columns: columns, spacing: 24) {
                            ForEach(savedPlaylists) { pl in
                                PlaylistCardView(
                                    playlist: pl,
                                    onSelect: { navigation.goToPlaylist(pl) },
                                    onPlay: {
                                        Task {
                                            do {
                                                try await playSavedPlaylist(id: pl.id)
                                                await player.fetchQueue()
                                            } catch {
                                                errorText = String(describing: error)
                                            }
                                        }
                                    },
                                    onDelete: {
                                        Task {
                                            do {
                                                try await deleteSavedPlaylist(id: pl.id)
                                                savedPlaylists.removeAll { $0.id == pl.id }
                                            } catch {
                                                errorText = String(describing: error)
                                            }
                                        }
                                    }
                                )
                            }
                        }
                        .padding(.horizontal, 20)
                    }
                }
            }
            .padding(.vertical, 20)
        }
        .sheet(isPresented: $showCreateSheet) {
            CreatePlaylistSheet(isPresented: $showCreateSheet) { name in
                Task {
                    do {
                        isCreating = true
                        let pl = try await createSavedPlaylist(name: name)
                        let saved = SavedPlaylist(
                            id: pl.id, name: pl.name,
                            description: pl.hasDescription_p ? pl.description_p : nil,
                            image: pl.hasImage ? pl.image : nil,
                            folderID: pl.hasFolderID ? pl.folderID : nil,
                            trackCount: pl.trackCount
                        )
                        savedPlaylists.append(saved)
                        isCreating = false
                    } catch {
                        isCreating = false
                        errorText = String(describing: error)
                    }
                }
            }
        }
        .task {
            await loadPlaylists()
        }
        .alert("Error", isPresented: .constant(errorText != nil)) {
            Button("OK") { errorText = nil }
        } message: {
            Text(errorText ?? "")
        }
    }

    private func loadPlaylists() async {
        do {
            async let saved = fetchSavedPlaylists()
            async let smart = fetchSmartPlaylists()
            let (savedData, smartData) = try await (saved, smart)
            savedPlaylists = savedData.map {
                SavedPlaylist(
                    id: $0.id, name: $0.name,
                    description: $0.hasDescription_p ? $0.description_p : nil,
                    image: $0.hasImage ? $0.image : nil,
                    folderID: $0.hasFolderID ? $0.folderID : nil,
                    trackCount: $0.trackCount
                )
            }
            smartPlaylists = smartData.map {
                SmartPlaylist(
                    id: $0.id, name: $0.name,
                    description: $0.hasDescription_p ? $0.description_p : nil,
                    image: $0.hasImage ? $0.image : nil,
                    isSystem: $0.isSystem
                )
            }
        } catch {
            errorText = String(describing: error)
        }
    }
}

struct CreatePlaylistSheet: View {
    @Binding var isPresented: Bool
    var onCreate: (String) -> Void
    @State private var name = ""

    var body: some View {
        VStack(spacing: 20) {
            Text("New Playlist")
                .font(.headline)
            TextField("Name", text: $name)
                .textFieldStyle(.roundedBorder)
                .frame(width: 280)
            HStack(spacing: 12) {
                Button("Cancel") { isPresented = false }
                    .keyboardShortcut(.cancelAction)
                Button("Create") {
                    guard !name.trimmingCharacters(in: .whitespaces).isEmpty else { return }
                    onCreate(name.trimmingCharacters(in: .whitespaces))
                    isPresented = false
                }
                .keyboardShortcut(.defaultAction)
                .disabled(name.trimmingCharacters(in: .whitespaces).isEmpty)
            }
        }
        .padding(24)
        .frame(width: 340)
    }
}
