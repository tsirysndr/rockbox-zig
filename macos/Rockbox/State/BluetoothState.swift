//
//  BluetoothState.swift
//  Rockbox
//

import Foundation
import SwiftUI

struct BluetoothDeviceInfo: Identifiable, Hashable {
    var id: String { address }
    let address: String
    let name: String
    let paired: Bool
    let trusted: Bool
    var connected: Bool
}

@available(macOS 15.0, *)
@MainActor
class BluetoothState: ObservableObject {
    @Published var devices: [BluetoothDeviceInfo] = []
    @Published var isLoading = false
    @Published var available = false

    func checkAvailability() async {
        available = await checkBluetoothAvailable()
    }

    func refresh() async {
        guard available else { return }
        if devices.isEmpty { isLoading = true }
        do {
            let raw = try await fetchBluetoothDevices()
            devices = raw.map {
                BluetoothDeviceInfo(
                    address: $0.address,
                    name: $0.name.isEmpty ? $0.address : $0.name,
                    paired: $0.paired,
                    trusted: $0.trusted,
                    connected: $0.connected
                )
            }
        } catch {}
        isLoading = false
    }

    func connect(_ device: BluetoothDeviceInfo) async {
        do {
            try await connectBluetoothDevice(address: device.address)
            for i in devices.indices {
                devices[i].connected = devices[i].address == device.address
            }
        } catch {}
    }

    func disconnect(_ device: BluetoothDeviceInfo) async {
        do {
            try await disconnectBluetoothDevice(address: device.address)
            for i in devices.indices {
                if devices[i].address == device.address {
                    devices[i].connected = false
                }
            }
        } catch {}
    }
}
