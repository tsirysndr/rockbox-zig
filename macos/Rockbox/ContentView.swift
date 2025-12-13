//
//  ContentView.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 13/12/2025.
//

import SwiftUI

struct ContentView: View {
    @State private var selection: SidebarItem? = .library
    
    var body: some View {
        NavigationSplitView {
            Sidebar(selection: $selection)
        } detail: {
            DetailView(selection: selection)
        }
    }
}

enum SidebarItem: String, CaseIterable, Identifiable {
    case library = "Library"
    case recentlyAdded = "Recently Added"
    case artists = "Artists"
    case albums = "Albums"
    case songs = "Songs"
    case playlists = "Playlists"
    
    var id: String { rawValue }
    
    var icon: String {
        switch self {
        case .library: return "music.note.house"
        case .recentlyAdded: return "clock"
        case .artists: return "music.mic"
        case .albums: return "square.stack"
        case .songs: return "music.note"
        case .playlists: return "music.note.list"
        }
    }
}

struct Sidebar: View {
    @Binding var selection: SidebarItem?
    
    var body: some View {
        List(selection: $selection) {
            Section("Library") {
                ForEach(SidebarItem.allCases) { item in
                    Label(item.rawValue, systemImage: item.icon)
                        .tag(item)
                }
            }
        }
        .navigationTitle("Rockbox")
        .listStyle(.sidebar)
    }
}

struct DetailView: View {
    let selection: SidebarItem?
    
    var body: some View {
        if let selection {
            Text(selection.rawValue)
                .font(.largeTitle)
                .frame(maxWidth: .infinity, maxHeight: .infinity)
        } else {
            Text("Select an item")
                .foregroundStyle(.secondary)
        }
    }
}

#Preview {
    ContentView()
}

