//
//  DeviceListView.swift
//  Rockbox
//

import SwiftUI

/// Returns the SF Symbol name that best represents a device's service/app type.
private func deviceSymbol(_ device: DeviceInfo) -> String {
    switch device.service {
    case "builtin":      return "macmini"
    case "fifo":         return "antenna.radiowaves.left.and.right"
    case "snapcast":     return "antenna.radiowaves.left.and.right"
    case "squeezelite":  return "hifispeaker"
    case "airplay":      return "airplayvideo"
    case "chromecast":   return "tv.and.mediabox"
    case "upnp":         return "network"
    default:             return "hifispeaker"
    }
}

/// Returns the accent colour for a device type.
private func deviceColor(_ device: DeviceInfo) -> Color {
    switch device.service {
    case "builtin":      return Color(hex: "28fce3")
    case "fifo":         return Color(hex: "9090ff")
    case "snapcast":     return Color(hex: "9090ff")
    case "squeezelite":  return Color(hex: "ffa028")
    case "airplay":      return Color(hex: "fe09a3")
    case "chromecast":   return Color(hex: "28cbfc")
    case "upnp":         return Color(hex: "fe09a3")
    default:             return .secondary
    }
}

struct DeviceListView: View {
    @EnvironmentObject var deviceState: DeviceState
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            Text("Output Device")
                .font(.system(size: 11, weight: .semibold))
                .foregroundStyle(.secondary)
                .padding(.horizontal, 16)
                .padding(.top, 14)
                .padding(.bottom, 8)

            Divider()
                .padding(.horizontal, 8)

            if deviceState.isLoading {
                HStack {
                    Spacer()
                    ProgressView()
                        .padding()
                    Spacer()
                }
            } else if deviceState.devices.isEmpty {
                Text("No devices found.")
                    .font(.system(size: 12))
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .center)
                    .padding()
            } else {
                ScrollView {
                    VStack(spacing: 2) {
                        ForEach(deviceState.devices) { device in
                            DeviceRow(device: device) {
                                Task {
                                    await deviceState.connect(device)
                                    dismiss()
                                }
                            }
                        }
                    }
                    .padding(.vertical, 6)
                    .padding(.horizontal, 8)
                }
                .frame(maxHeight: 280)
            }
        }
        .frame(width: 280)
        .task { await deviceState.refresh() }
    }
}

private struct DeviceRow: View {
    let device: DeviceInfo
    let onTap: () -> Void

    @State private var isHovering = false

    var body: some View {
        Button(action: onTap) {
            HStack(spacing: 10) {
                ZStack {
                    RoundedRectangle(cornerRadius: 6)
                        .fill(deviceColor(device).opacity(0.12))
                        .frame(width: 30, height: 30)
                    Image(systemName: deviceSymbol(device))
                        .font(.system(size: 14))
                        .foregroundStyle(deviceColor(device))
                }

                Text(device.name)
                    .font(.system(size: 13))
                    .lineLimit(1)
                    .frame(maxWidth: .infinity, alignment: .leading)

                if device.isCurrentDevice {
                    Image(systemName: "checkmark")
                        .font(.system(size: 11, weight: .semibold))
                        .foregroundStyle(Color(hex: "28fce3"))
                }
            }
            .padding(.horizontal, 8)
            .padding(.vertical, 6)
            .background(
                RoundedRectangle(cornerRadius: 6)
                    .fill(isHovering ? Color.secondary.opacity(0.1) : Color.clear)
            )
            .contentShape(RoundedRectangle(cornerRadius: 6))
        }
        .buttonStyle(.plain)
        .onHover { isHovering = $0 }
    }
}
