import SwiftUI

@main
struct RockboxApp: App {
    @StateObject private var player = PlayerState()
    @StateObject private var navigation = NavigationManager()
    @StateObject private var searchManager = SearchManager()
    @StateObject private var deviceState = DeviceState()
    @StateObject private var bluetoothState = BluetoothState()
    @State private var isDaemonStarting = true
    @State private var startupFailed = false
    @State private var startupError: Error?

    var body: some Scene {
        WindowGroup {
            Group {
                if isDaemonStarting {
                    startingView
                } else {
                    ContentView()
                        .environmentObject(player)
                        .environmentObject(navigation)
                        .environmentObject(searchManager)
                        .environmentObject(deviceState)
                        .environmentObject(bluetoothState)
                }
            }
            .alert("Startup Failed", isPresented: $startupFailed) {
                Button("Retry") { retry() }
                    .keyboardShortcut(.defaultAction)
                Button("Quit") { NSApplication.shared.terminate(nil) }
                    .keyboardShortcut(.cancelAction)
            } message: {
                Text(startupError?.localizedDescription ?? "Failed to start the Rockbox daemon.")
            }
            .task { await performStartup() }
            .onReceive(NotificationCenter.default.publisher(for: .rockboxServerDidChange)) { _ in
                Task { await onServerChanged() }
            }
        }
        .windowStyle(.hiddenTitleBar)
    }

    private var startingView: some View {
        VStack(spacing: 16) {
            ProgressView()
                .controlSize(.large)
            Text("Starting Rockbox…")
                .foregroundStyle(.secondary)
        }
        .frame(minWidth: 400, minHeight: 300)
    }

    private func performStartup() async {
        do {
            // rb_daemon_start blocks for up to 30 s — run off the main actor.
            let port = try await Task.detached(priority: .userInitiated) {
                try DaemonManager.shared.start()
            }.value

            let info = RockboxServerInfo(
                name: "localhost", host: "127.0.0.1",
                grpcPort: port, graphqlPort: 6062, httpPort: 6063
            )
            ServerManager.shared.selectServer(info)

            isDaemonStarting = false

            player.startStreaming()
            player.fetchSettings()
            await deviceState.refresh()
            await bluetoothState.checkAvailability()

            // Start a background mDNS scan so the server picker stays populated.
            Task { await ServerManager.shared.scan() }
        } catch {
            isDaemonStarting = false
            startupError = error
            startupFailed = true
        }
    }

    private func onServerChanged() async {
        player.startStreaming()
        player.fetchSettings()
        await deviceState.refresh()
        await bluetoothState.checkAvailability()
    }

    private func retry() {
        Task { await performStartup() }
    }
}
