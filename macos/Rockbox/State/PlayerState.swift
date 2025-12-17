//
//  PlayerState.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 14/12/2025.
//

import Foundation
import SwiftUI

@MainActor
class PlayerState: ObservableObject {
    @Published var isPlaying = false
    @Published var currentTime: TimeInterval = 0
    @Published var duration: TimeInterval = 0
    @Published var currentTrack = Song(cuid: "", title: "Not Playing", artist: "", album: "", albumArt: nil, duration: TimeInterval(0), trackNumber: 0, discNumber: 0, color: .gray.opacity(0.3))
    @Published var isConnected = false
    @Published var error: Error?
    
    private var streamTask: Task<Void, Never>?
    private var streamStatusTask: Task<Void, Never>?
    
    var progress: Double {
        get { duration > 0 ? currentTime / duration : 0 }
        set { currentTime = newValue * duration }
    }
        
    func startStreaming() {
        streamTask?.cancel()
        streamTask = Task {
            do {
                isConnected = true
                for try await response in currentTrackStream() {
                    self.currentTrack =  Song(cuid: response.id, title: response.title, artist: response.artist, album: response.album, albumArt: URL(string: "http://localhost:6062/covers/" + response.albumArt), duration: TimeInterval(response.length / 1000), trackNumber: Int(response.tracknum), discNumber: Int(response.discnum), color: .gray.opacity(0.3))
                    self.duration = TimeInterval(response.length / 1000)
                    self.currentTime = TimeInterval(response.elapsed / 1000)
                }
            } catch is CancellationError {
                // Ignored
            } catch {
                self.error = error
            }
            isConnected = false
        }
        
        streamStatusTask?.cancel()
        streamStatusTask = Task {
            do {
                for try await response in playbackStatusStream() {
                    self.isPlaying = response.status == 1
                }
            } catch is CancellationError {
                // Ignoted
            } catch  {
                self.error = error
            }
        }
    }
    
    func stopStreaming() {
        streamTask?.cancel()
        streamTask = nil
        streamStatusTask?.cancel()
        streamStatusTask = nil
    }
}
