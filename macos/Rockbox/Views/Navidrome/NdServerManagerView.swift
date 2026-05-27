import SwiftUI

struct NdServerManagerView: View {
    var onDismiss: () -> Void

    @ObservedObject private var ndManager = NavidromeManager.shared
    @State private var urlText = ""
    @State private var username = ""
    @State private var password = ""
    @State private var connecting = false
    @State private var errorText: String?
    @State private var serverToRemove: NdServer? = nil

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            // Title bar
            HStack {
                Text("Navidrome / Subsonic Servers")
                    .font(.headline)
                Spacer()
                Button(action: onDismiss) {
                    Image(systemName: "xmark.circle.fill")
                        .font(.system(size: 16))
                        .foregroundStyle(.secondary)
                }
                .buttonStyle(.plain)
            }
            .padding(.horizontal, 20)
            .padding(.top, 20)
            .padding(.bottom, 16)

            Divider()

            ScrollView {
                VStack(alignment: .leading, spacing: 20) {
                    // Connected servers
                    if !ndManager.servers.isEmpty {
                        VStack(alignment: .leading, spacing: 8) {
                            Text("CONNECTED SERVERS")
                                .font(.system(size: 10, weight: .bold))
                                .foregroundStyle(.secondary)
                                .padding(.horizontal, 20)

                            VStack(spacing: 0) {
                                ForEach(ndManager.servers) { server in
                                    serverRow(server)
                                    if server.id != ndManager.servers.last?.id {
                                        Divider().padding(.leading, 52)
                                    }
                                }
                            }
                            .background(Color.black.opacity(0.03))
                            .cornerRadius(8)
                            .padding(.horizontal, 20)
                        }
                        .padding(.top, 16)
                    }

                    // Add server form
                    VStack(alignment: .leading, spacing: 8) {
                        Text("ADD SERVER")
                            .font(.system(size: 10, weight: .bold))
                            .foregroundStyle(.secondary)
                            .padding(.horizontal, 20)

                        VStack(spacing: 0) {
                            TextField("Server URL (e.g. http://192.168.1.10:4533)", text: $urlText)
                                .textFieldStyle(.plain)
                                .font(.system(size: 13, design: .monospaced))
                                .padding(.horizontal, 12)
                                .padding(.vertical, 9)
                                .disabled(connecting)
                            Divider().padding(.leading, 12)
                            TextField("Username", text: $username)
                                .textFieldStyle(.plain)
                                .font(.system(size: 13))
                                .padding(.horizontal, 12)
                                .padding(.vertical, 9)
                                .disabled(connecting)
                            Divider().padding(.leading, 12)
                            SecureField("Password", text: $password)
                                .textFieldStyle(.plain)
                                .font(.system(size: 13))
                                .padding(.horizontal, 12)
                                .padding(.vertical, 9)
                                .disabled(connecting)
                        }
                        .background(Color.black.opacity(0.03))
                        .cornerRadius(8)
                        .padding(.horizontal, 20)

                        if let err = errorText {
                            Text(err)
                                .font(.system(size: 12))
                                .foregroundColor(.red)
                                .padding(.horizontal, 20)
                        }

                        HStack {
                            Spacer()
                            Button(connecting ? "Connecting…" : "Connect") {
                                Task { await connect() }
                            }
                            .buttonStyle(.borderedProminent)
                            .disabled(connecting || urlText.trimmingCharacters(in: .whitespaces).isEmpty || username.trimmingCharacters(in: .whitespaces).isEmpty)
                        }
                        .padding(.horizontal, 20)
                    }
                    .padding(.top, ndManager.servers.isEmpty ? 16 : 0)

                    Text("Your password is stored locally and used only to generate auth tokens for each request.")
                        .font(.system(size: 12))
                        .foregroundStyle(.secondary)
                        .padding(.horizontal, 20)
                        .padding(.bottom, 20)
                }
            }
        }
        .frame(width: 420)
        .confirmationDialog(
            "Remove \"\(serverToRemove?.label ?? "")\"?",
            isPresented: .constant(serverToRemove != nil),
            titleVisibility: .visible
        ) {
            Button("Remove", role: .destructive) {
                if let s = serverToRemove { ndManager.removeServer(id: s.id) }
                serverToRemove = nil
            }
            Button("Cancel", role: .cancel) { serverToRemove = nil }
        }
    }

    @ViewBuilder
    private func serverRow(_ server: NdServer) -> some View {
        let isActive = server.id == ndManager.activeId
        HStack(spacing: 12) {
            // Active indicator / switch button
            Button(action: { ndManager.setActive(id: server.id) }) {
                ZStack {
                    Circle()
                        .fill(isActive ? Color.accentColor : Color.black.opacity(0.1))
                        .frame(width: 28, height: 28)
                    Image(systemName: isActive ? "checkmark" : "music.note")
                        .font(.system(size: 12, weight: .semibold))
                        .foregroundStyle(isActive ? .white : .secondary)
                }
            }
            .buttonStyle(.plain)

            VStack(alignment: .leading, spacing: 2) {
                Text(server.label)
                    .font(.system(size: 13, weight: .semibold))
                    .lineLimit(1)
                Text("\(server.user)@\(server.baseUrl.replacingOccurrences(of: "https://", with: "").replacingOccurrences(of: "http://", with: ""))")
                    .font(.system(size: 11, design: .monospaced))
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
            }

            Spacer()

            if isActive {
                Text("Active")
                    .font(.system(size: 11, weight: .medium))
                    .foregroundStyle(Color.accentColor)
            }

            // Delete button
            Button(action: { serverToRemove = server }) {
                Image(systemName: "trash")
                    .font(.system(size: 13))
                    .foregroundStyle(.red.opacity(0.8))
            }
            .buttonStyle(.plain)
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 10)
    }

    private func connect() async {
        let baseUrl = urlText.trimmingCharacters(in: .whitespaces)
            .replacingOccurrences(of: "/$", with: "", options: .regularExpression)
        let user = username.trimmingCharacters(in: .whitespaces)
        guard !baseUrl.isEmpty, !user.isEmpty else {
            errorText = "Server URL and username are required."
            return
        }
        connecting = true
        errorText = nil
        defer { connecting = false }

        let ok = await ndPing(baseUrl: baseUrl, user: user, password: password)
        guard ok else {
            errorText = "Could not connect. Check the URL and credentials."
            return
        }
        let label = baseUrl
            .replacingOccurrences(of: "https://", with: "")
            .replacingOccurrences(of: "http://", with: "")
            .components(separatedBy: "/").first ?? baseUrl
        await MainActor.run {
            ndManager.addServer(label: label, baseUrl: baseUrl, user: user, password: password)
        }
        urlText = ""
        username = ""
        password = ""
    }
}
