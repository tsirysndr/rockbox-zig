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
    @StateObject private var deviceState = DeviceState()
    @State private var startupFailed = false
    @State private var startupError: Error?

    var body: some Scene {
        WindowGroup {
            ContentView()
                .environmentObject(player)
                .environmentObject(navigation)
                .environmentObject(searchManager)
                .environmentObject(deviceState)
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
            // Priority 1: localhost. Priority 2: first mDNS-discovered server.
            // ServerManager already kicked off a background scan; if localhost is
            // unreachable we wait for it to finish and try the first result.
            if (try? await fetchGlobalStatus()) == nil {
                // localhost not available — wait for mDNS scan results
                let serverManager = ServerManager.shared
                if !serverManager.isScanning {
                    await serverManager.scan()
                } else {
                    // Already scanning (started at init) — poll until done
                    while serverManager.isScanning {
                        try? await Task.sleep(nanoseconds: 100_000_000)
                    }
                }
                if let first = serverManager.discoveredServers.first {
                    serverManager.selectServer(first)
                }
                _ = try await fetchGlobalStatus()
            }

            player.startStreaming()
            player.fetchSettings()
            await deviceState.refresh()

        } catch {
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
