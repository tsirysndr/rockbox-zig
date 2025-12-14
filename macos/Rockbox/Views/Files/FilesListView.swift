//
//  FilesListView.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 14/12/2025.
//

import SwiftUI

struct FilesListView: View {
    var body: some View {
        ScrollView {
            LazyVStack(spacing: 0) {
                // Header row
                FileHeaderRow()
                
                Divider()
                
                // File rows
                ForEach(Array(sampleFiles.enumerated()), id: \.element.id) { index, file in
                    FileRowView(file: file, isEven: index % 2 == 0)
                }
            }
        }
    }
}

