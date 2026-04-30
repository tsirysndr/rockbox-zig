//
//  SearchService.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 14/12/2025.
//

import Foundation
import GRPCCore
import GRPCNIOTransportHTTP2

func searchTrack(query: String) async throws -> Rockbox_V1alpha1_SearchResponse {
  try await withRockboxGRPCClient { grpcClient in
    let library = Rockbox_V1alpha1_LibraryService.Client(wrapping: grpcClient)
    var req = Rockbox_V1alpha1_SearchRequest()
    req.term = query
    return try await library.search(req)
  }
}
