//
//  SearchService.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 14/12/2025.
//

import Foundation
import GRPCCore
import GRPCNIOTransportHTTP2

func searchTrack(query: String, host: String = "127.0.0.1", port: Int = 6061) async throws
  -> Rockbox_V1alpha1_SearchResponse
{
  try await withGRPCClient(
    transport: .http2NIOPosix(
      target: .dns(host: host, port: port),
      transportSecurity: .plaintext
    )
  ) { grpcClient in
    let library = Rockbox_V1alpha1_LibraryService.Client(wrapping: grpcClient)

    var req = Rockbox_V1alpha1_SearchRequest()
    req.term = query

    let res = try await library.search(req)

    return res
  }
}
