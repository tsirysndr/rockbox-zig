//
//  BluetoothService.swift
//  Rockbox
//

import Foundation
import GRPCCore
import GRPCNIOTransportHTTP2

@available(macOS 15.0, *)
func fetchBluetoothDevices() async throws -> [Rockbox_V1alpha1_BluetoothDevice] {
    try await withRockboxGRPCClient { grpcClient in
        let bt = Rockbox_V1alpha1_BluetoothService.Client(wrapping: grpcClient)
        let resp = try await bt.getDevices(Rockbox_V1alpha1_GetBluetoothDevicesRequest())
        return resp.devices
    }
}

@available(macOS 15.0, *)
func checkBluetoothAvailable() async -> Bool {
    // Mirror GPUI: bluetooth is considered available whenever the server is
    // reachable. Calling getDevices here made the button disappear whenever
    // the service returned UNIMPLEMENTED or any transient error.
    do {
        _ = try await fetchGlobalStatus()
        return true
    } catch {
        return false
    }
}

@available(macOS 15.0, *)
func connectBluetoothDevice(address: String) async throws {
    try await withRockboxGRPCClient { grpcClient in
        let bt = Rockbox_V1alpha1_BluetoothService.Client(wrapping: grpcClient)
        var req = Rockbox_V1alpha1_ConnectBluetoothDeviceRequest()
        req.address = address
        let _ = try await bt.connectDevice(req)
    }
}

@available(macOS 15.0, *)
func disconnectBluetoothDevice(address: String) async throws {
    try await withRockboxGRPCClient { grpcClient in
        let bt = Rockbox_V1alpha1_BluetoothService.Client(wrapping: grpcClient)
        var req = Rockbox_V1alpha1_DisconnectBluetoothDeviceRequest()
        req.address = address
        let _ = try await bt.disconnect(req)
    }
}
