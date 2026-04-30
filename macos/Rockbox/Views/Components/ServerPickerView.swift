//
//  ServerPickerView.swift
//  Rockbox
//

import SwiftUI

struct ServerPickerView: View {
  @ObservedObject var serverManager: ServerManager

  var body: some View {
    VStack(spacing: 0) {
      Divider()

      if serverManager.isScanning || !serverManager.discoveredServers.isEmpty {
        serverList
      }

      // Toggle / status row
      HStack(spacing: 6) {
        Image(systemName: "antenna.radiowaves.left.and.right")
          .font(.system(size: 11))
          .foregroundStyle(.secondary)

        Text(serverManager.currentServer.displayName)
          .font(.system(size: 12))
          .foregroundStyle(.secondary)
          .lineLimit(1)
          .truncationMode(.tail)

        Spacer()

        Button {
          Task { await serverManager.scan() }
        } label: {
          if serverManager.isScanning {
            ProgressView()
              .scaleEffect(0.6)
              .frame(width: 14, height: 14)
          } else {
            Image(systemName: "arrow.clockwise")
              .font(.system(size: 10))
              .foregroundStyle(.secondary)
          }
        }
        .buttonStyle(.plain)
        .disabled(serverManager.isScanning)
      }
      .padding(.horizontal, 12)
      .padding(.vertical, 8)
    }
  }

  @ViewBuilder
  private var serverList: some View {
    VStack(spacing: 0) {
      // Header
      HStack {
        Text("Servers")
          .font(.system(size: 10, weight: .semibold))
          .foregroundStyle(.secondary)
          .textCase(.uppercase)
        Spacer()
      }
      .padding(.horizontal, 12)
      .padding(.top, 8)
      .padding(.bottom, 4)

      // Localhost row
      serverRow(RockboxServerInfo.localhost)

      // Discovered rows
      ForEach(serverManager.discoveredServers) { server in
        serverRow(server)
      }

      if serverManager.isScanning {
        HStack {
          Text("Scanning…")
            .font(.system(size: 11))
            .foregroundStyle(.tertiary)
          Spacer()
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 4)
      }
    }
  }

  @ViewBuilder
  private func serverRow(_ server: RockboxServerInfo) -> some View {
    let isActive = server.host == serverManager.currentServer.host
    Button {
      serverManager.selectServer(server)
    } label: {
      HStack(spacing: 8) {
        Circle()
          .fill(isActive ? Color(red: 0.22, green: 1.0, blue: 0.08) : Color.secondary.opacity(0.4))
          .frame(width: 6, height: 6)
        Text(server.displayName)
          .font(.system(size: 12))
          .foregroundStyle(isActive ? .primary : .secondary)
          .lineLimit(1)
          .truncationMode(.tail)
        Spacer()
      }
      .padding(.horizontal, 12)
      .padding(.vertical, 5)
      .background(isActive ? Color.accentColor.opacity(0.08) : Color.clear)
    }
    .buttonStyle(.plain)
  }
}
