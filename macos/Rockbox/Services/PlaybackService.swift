//
//  PlaybackService.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 14/12/2025.
//

import Foundation
import GRPCCore
import GRPCNIOTransportHTTP2

func resume(host: String = "127.0.0.1", port: Int = 6061) async throws -> Void {
  try await withGRPCClient(
    transport: .http2NIOPosix(
      target: .dns(host: host, port: port),
      transportSecurity: .plaintext
    )
  ) { grpcClient in
      let playback = Rockbox_V1alpha1_PlaybackService.Client(wrapping: grpcClient)
      let req = Rockbox_V1alpha1_ResumeRequest()
      let _ = try await playback.resume(req)
  }
}

func pause(host: String = "127.0.0.1", port: Int = 6061) async throws -> Void{
  try await withGRPCClient(
    transport: .http2NIOPosix(
      target: .dns(host: host, port: port),
      transportSecurity: .plaintext
    )
  ) { grpcClient in
      let playback = Rockbox_V1alpha1_PlaybackService.Client(wrapping: grpcClient)
      let req = Rockbox_V1alpha1_PauseRequest()
      let _ = try await playback.pause(req)
  }
}

func previous(host: String = "127.0.0.1", port: Int = 6061) async throws -> Void {
  try await withGRPCClient(
    transport: .http2NIOPosix(
      target: .dns(host: host, port: port),
      transportSecurity: .plaintext
    )
  ) { grpcClient in
      let playback = Rockbox_V1alpha1_PlaybackService.Client(wrapping: grpcClient)
      let req = Rockbox_V1alpha1_PreviousRequest()
      let _ = try await playback.previous(req)
  }
}

func next(host: String = "127.0.0.1", port: Int = 6061) async throws -> Void {
  try await withGRPCClient(
    transport: .http2NIOPosix(
      target: .dns(host: host, port: port),
      transportSecurity: .plaintext
    )
  ) { grpcClient in
      let playback = Rockbox_V1alpha1_PlaybackService.Client(wrapping: grpcClient)
      let req = Rockbox_V1alpha1_NextRequest()
      let _ = try await playback.next(req)
  }
}
