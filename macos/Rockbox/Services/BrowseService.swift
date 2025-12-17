//
//  BrowseService.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 17/12/2025.
//

import Foundation
import GRPCCore
import GRPCNIOTransportHTTP2


func fetchFiles(path: String?, host: String = "127.0.0.1", port: Int = 6061) async throws -> [Rockbox_V1alpha1_Entry] {
  try await withGRPCClient(
    transport: .http2NIOPosix(
      target: .dns(host: host, port: port),
      transportSecurity: .plaintext
    )
  ) { grpcClient in
    let browse = Rockbox_V1alpha1_BrowseService.Client(wrapping: grpcClient)

    var req = Rockbox_V1alpha1_TreeGetEntriesRequest()
    if path != nil {
      req.path = path ?? String()
    }
    let res = try await browse.treeGetEntries(req)
      return res.entries
  }
}
