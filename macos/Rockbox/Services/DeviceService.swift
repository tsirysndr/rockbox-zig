//
//  DeviceService.swift
//  Rockbox
//

import Foundation

private let httpBase = "http://127.0.0.1:6062"

struct DeviceInfo: Codable, Identifiable, Hashable {
    var id: String
    var name: String
    var ip: String
    var port: UInt16
    var service: String
    var app: String
    var isConnected: Bool
    var isCurrentDevice: Bool

    enum CodingKeys: String, CodingKey {
        case id, name, ip, port, service, app
        case isConnected      = "is_connected"
        case isCurrentDevice  = "is_current_device"
    }
}

func fetchDevices() async throws -> [DeviceInfo] {
    let url = URL(string: "\(httpBase)/devices")!
    let (data, _) = try await URLSession.shared.data(from: url)
    return try JSONDecoder().decode([DeviceInfo].self, from: data)
}

func connectDevice(id: String) async throws {
    var request = URLRequest(url: URL(string: "\(httpBase)/devices/\(id)/connect")!)
    request.httpMethod = "PUT"
    let _ = try await URLSession.shared.data(for: request)
}

func disconnectDevice(id: String) async throws {
    var request = URLRequest(url: URL(string: "\(httpBase)/devices/\(id)/disconnect")!)
    request.httpMethod = "PUT"
    let _ = try await URLSession.shared.data(for: request)
}
