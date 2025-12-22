//
//  Sidebar.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 14/12/2025.
//

import SwiftUI

struct Sidebar: View {
  @Binding var selection: SidebarItem?
  @EnvironmentObject var searchManager: SearchManager

  var body: some View {
    ZStack {
      VisualEffectView(material: .sidebar)
        .ignoresSafeArea()

      VStack(spacing: 0) {
        // Search input
        HStack(spacing: 8) {
          Image(systemName: "magnifyingglass")
            .font(.system(size: 12))
            .foregroundStyle(.secondary)

          TextField("Search", text: $searchManager.searchText)
            .textFieldStyle(.plain)
            .font(.system(size: 13))
            .onSubmit {
              searchManager.search()
            }

          if !searchManager.searchText.isEmpty {
            Button(action: { searchManager.clear() }) {
              Image(systemName: "xmark.circle.fill")
                .font(.system(size: 12))
                .foregroundStyle(.secondary)
            }
            .buttonStyle(.plain)
          }
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 6)
        .background(Color.black.opacity(0.05))
        .cornerRadius(8)
        .padding(.horizontal, 12)
        .padding(.top, 12)
        .padding(.bottom, 8)
        .onChange(of: searchManager.searchText) {
          searchManager.search()
        }

        List(selection: $selection) {
          Section("Library") {
            ForEach(SidebarItem.allCases) { item in
              Label(item.rawValue, systemImage: item.icon)
                .tag(item)
            }
          }
        }
        .listStyle(.sidebar)
        .scrollContentBackground(.hidden)
      }
    }
  }
}

struct VisualEffectView: NSViewRepresentable {
  var material: NSVisualEffectView.Material = .sidebar
  var blendingMode: NSVisualEffectView.BlendingMode = .behindWindow
  var state: NSVisualEffectView.State = .active

  func makeNSView(context: Context) -> NSVisualEffectView {
    let v = NSVisualEffectView()
    v.material = material
    v.blendingMode = blendingMode
    v.state = state
    return v
  }

  func updateNSView(_ nsView: NSVisualEffectView, context: Context) {
    nsView.material = material
    nsView.blendingMode = blendingMode
    nsView.state = state
  }
}
