//
//  SystemService.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 17/12/2025.
//

import Foundation
import GRPCCore
import GRPCNIOTransportHTTP2

func fetchGlobalStatus() async throws -> Rockbox_V1alpha1_GetGlobalStatusResponse {
  try await withRockboxGRPCClient { grpcClient in
    let system = Rockbox_V1alpha1_SystemService.Client(wrapping: grpcClient)
    let req = Rockbox_V1alpha1_GetGlobalStatusRequest()
    return try await system.getGlobalStatus(req)
  }
}
