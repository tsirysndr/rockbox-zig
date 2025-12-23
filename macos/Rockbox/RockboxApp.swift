//
//  RockboxApp.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 13/12/2025.
//

import SwiftUI

@main
struct RockboxApp: App {
    @StateObject private var player = PlayerState()
    @StateObject private var navigation = NavigationManager()
    @StateObject private var searchManager = SearchManager()
    
    var body: some Scene {
        WindowGroup {
            ContentView()
                .environmentObject(player)
                .environmentObject(navigation)
                .environmentObject(searchManager)
                .task {
                    player.startStreaming()
                }
                .task {
                    player.fetchSettings()
                }
        }
        .windowStyle(.hiddenTitleBar)
    }
}
