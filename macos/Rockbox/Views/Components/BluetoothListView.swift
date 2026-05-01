//
//  BluetoothListView.swift
//  Rockbox
//

import SwiftUI

@available(macOS 15.0, *)
struct BluetoothListView: View {
    @EnvironmentObject var bluetoothState: BluetoothState
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            Text("Bluetooth speakers")
                .font(.system(size: 11, weight: .semibold))
                .foregroundStyle(.secondary)
                .padding(.horizontal, 16)
                .padding(.top, 14)
                .padding(.bottom, 8)

            Divider()
                .padding(.horizontal, 8)

            if bluetoothState.isLoading {
                HStack {
                    Spacer()
                    ProgressView()
                        .padding()
                    Spacer()
                }
            } else if bluetoothState.devices.isEmpty {
                Text("No bluetooth devices found.")
                    .font(.system(size: 12))
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, alignment: .center)
                    .padding()
            } else {
                ScrollView {
                    VStack(spacing: 2) {
                        ForEach(bluetoothState.devices) { device in
                            BluetoothDeviceRow(device: device) {
                                Task {
                                    if device.connected {
                                        await bluetoothState.disconnect(device)
                                    } else {
                                        await bluetoothState.connect(device)
                                    }
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
        .task { await bluetoothState.refresh() }
    }
}

@available(macOS 15.0, *)
private struct BluetoothDeviceRow: View {
    let device: BluetoothDeviceInfo
    let onTap: () -> Void

    @State private var isHovering = false

    var body: some View {
        Button(action: onTap) {
            HStack(spacing: 10) {
                ZStack {
                    RoundedRectangle(cornerRadius: 6)
                        .fill(Color(hex: "1a91ff").opacity(0.12))
                        .frame(width: 30, height: 30)
                    Image("bluetooth")
                        .renderingMode(.template)
                        .resizable()
                        .scaledToFit()
                        .frame(width: 16, height: 16)
                        .foregroundStyle(Color(hex: "1a91ff"))
                }

                Text(device.name)
                    .font(.system(size: 13))
                    .lineLimit(1)
                    .frame(maxWidth: .infinity, alignment: .leading)

                if device.connected {
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
