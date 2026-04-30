//
//  BrowseService.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 17/12/2025.
//

import Foundation
import GRPCCore
import GRPCNIOTransportHTTP2

func fetchFiles(path: String?) async throws -> [Rockbox_V1alpha1_Entry] {
  try await withRockboxGRPCClient { grpcClient in
    let browse = Rockbox_V1alpha1_BrowseService.Client(wrapping: grpcClient)
    var req = Rockbox_V1alpha1_TreeGetEntriesRequest()
    if let path = path { req.path = path }
    let res = try await browse.treeGetEntries(req)
    return res.entries
  }
}
