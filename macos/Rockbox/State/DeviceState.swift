//
//  DeviceState.swift
//  Rockbox
//

import Foundation
import SwiftUI

@MainActor
class DeviceState: ObservableObject {
    @Published var devices: [DeviceInfo] = []
    @Published var isLoading = false

    var currentDevice: DeviceInfo? {
        devices.first { $0.isCurrentDevice }
    }

    func refresh() async {
        if devices.isEmpty { isLoading = true }
        do {
            devices = try await fetchDevices()
        } catch {
            // silently ignore — UI shows stale state
        }
        isLoading = false
    }

    func connect(_ device: DeviceInfo) async {
        do {
            try await connectDevice(id: device.id)
            // Optimistically update local state.
            for i in devices.indices {
                devices[i].isCurrentDevice = devices[i].id == device.id
            }
        } catch {}
    }

    func disconnect() async {
        guard let current = currentDevice else { return }
        do {
            try await disconnectDevice(id: current.id)
            for i in devices.indices {
                devices[i].isCurrentDevice = devices[i].id == "builtin"
            }
        } catch {}
    }
}
