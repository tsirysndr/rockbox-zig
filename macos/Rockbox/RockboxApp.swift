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
    @State private var startupFailed = false
    @State private var startupError: Error?
    
    var body: some Scene {
        WindowGroup {
            ContentView()
                .environmentObject(player)
                .environmentObject(navigation)
                .environmentObject(searchManager)
                .alert("Connection Failed", isPresented: $startupFailed) {
                    Button("Retry") {
                        retry()
                    }
                    .keyboardShortcut(.defaultAction)
                    
                    Button("Quit") {
                        NSApplication.shared.terminate(nil)
                    }
                    .keyboardShortcut(.cancelAction)
                } message: {
                    Text(startupError?.localizedDescription ?? "Failed to connect to the server. Please make sure the Rockbox server is running.")
                }
                .task {
                    await performStartup()
                }
        }
        .windowStyle(.hiddenTitleBar)
    }
    
    private func performStartup() async {
        do {
            // Check if server is available first
            _ = try await fetchGlobalStatus()
            
            // If successful, start normal operations
            player.startStreaming()
            player.fetchSettings()
            
        } catch {
            // Show error and allow retry or quit
            startupError = error
            startupFailed = true
        }
    }
    
    private func retry() {
        Task {
            await performStartup()
        }
    }
}
