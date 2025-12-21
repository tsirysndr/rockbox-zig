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
    @State private var showQueue = false
    
    var body: some View {
        NavigationSplitView {
            Sidebar(selection: $selection)
        } detail: {
            DetailView(selection: selection, player: player, library: library, showQueue: $showQueue)
        }
        .inspector(isPresented: $showQueue) {
            QueueView(library: library)
               .inspectorColumnWidth(min: 280, ideal: 300, max: 350)
       }
    }
}

#Preview {
    ContentView()
}

