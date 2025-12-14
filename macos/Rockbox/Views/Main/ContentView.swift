//
//  ContentView.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 14/12/2025.
//

import SwiftUI

struct ContentView: View {
    @State private var selection: SidebarItem? = .albums
    @StateObject private var player = PlayerState()
    @StateObject private var library = MusicLibrary()

    
    var body: some View {
        NavigationSplitView {
            Sidebar(selection: $selection)
        } detail: {
            DetailView(selection: selection, player: player, library: library)
        }
    }
}

#Preview {
    ContentView()
}

