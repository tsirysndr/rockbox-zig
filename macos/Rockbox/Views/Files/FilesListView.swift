//
//  FilesListView.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 14/12/2025.
//

import Foundation
import SwiftUI


struct FilesListView: View {
    @State private var files: [FileItem] = []
    @State private var errorText: String?
    @State private var currentPath: String? = nil
    @State private var pathHistory: [String?] = []  // For back navigation
    
    var body: some View {
        VStack(spacing: 0) {
            HStack {
                Button(action: goBack) {
                    Image(systemName: "chevron.left")
                        .font(.system(size: 14, weight: .medium))
                }
                .buttonStyle(.plain)
                .disabled(pathHistory.isEmpty)
                .opacity(pathHistory.isEmpty ? 0.3 : 1)
                
                Text(currentPathDisplay)
                    .font(.system(size: 13, weight: .medium))
                    .lineLimit(1)
                
                Spacer()
            }
            .padding(.horizontal, 16)
            .padding(.vertical, 10)
            
            Divider()
            
            ScrollView {
                LazyVStack(spacing: 0) {
                    // Header row
                    FileHeaderRow()
                    
                    Divider()
                    
                    // File rows
                    ForEach(Array(files.enumerated()), id: \.element.id) { index, file in
                        FileRowView(file: file, isEven: index % 2 == 0, selectedIndex: index, currentDirectory: currentPath ?? "")
                            .contentShape(Rectangle())
                            .onTapGesture {
                                if file.type == .directory {
                                    navigateTo(file: file)
                                }
                            }
                    }
                }
            }
        }
        .task(id: currentPath) {
            await loadFiles()
        }
        .alert("gRPC Error", isPresented: .constant(errorText != nil)) {
            Button("OK") { errorText = nil }
        } message: {
            Text(errorText ?? "")
        }
    }
    
    private var currentPathDisplay: String {
        if let path = currentPath {
            return URL(string: path)?.lastPathComponent ?? path
        }
        return "Files"
    }
    
    private func loadFiles() async {
        do {
            let data = try await fetchFiles(path: currentPath)
            var newFiles: [FileItem] = []
            
            for entry in data {
                if entry.attr == 16 {
                    newFiles.append(FileItem(
                        name: URL(string: entry.name)?.lastPathComponent ?? "",
                        path: entry.name,
                        type: .directory,
                        size: nil,
                        itemCount: nil
                    ))
                } else {
                    newFiles.append(FileItem(
                        name: URL(string: entry.name)?.lastPathComponent ?? "",
                        path: entry.name,
                        type: .audioFile,
                        size: nil,
                        itemCount: nil
                    ))
                }
            }
            
            files = newFiles.sorted { a, b in
                if a.type == b.type {
                    return a.name.localizedCaseInsensitiveCompare(b.name) == .orderedAscending
                }
                return a.type == .directory
            }
            
        } catch {
            errorText = String(describing: error)
        }
    }
    
    private func navigateTo(file: FileItem) {
        pathHistory.append(currentPath)
        currentPath = file.path
    }
    
    private func goBack() {
        guard !pathHistory.isEmpty else { return }
        currentPath = pathHistory.removeLast()
    }
}
