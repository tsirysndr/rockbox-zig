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
    @Published var status: Int32 = 0
    
    private var streamTask: Task<Void, Never>?
    private var streamStatusTask: Task<Void, Never>?
    
    var progress: Double {
        get { duration > 0 ? currentTime / duration : 0 }
        set { currentTime = newValue * duration }
    }
        
    func startStreaming() {
        getCurrentTrack()
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
                        self.status = response.status
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
    
    func getCurrentTrack() {
        Task {
            do {
                let data = try await fetchCurrentPlaylist()
                if data.tracks.count > 0 {
                    let currentIndex: Int = Int(data.index)
                    self.currentTrack =  Song(cuid: data.tracks[currentIndex].id, title: data.tracks[currentIndex].title, artist: data.tracks[currentIndex].artist, album: data.tracks[currentIndex].album, albumArt: URL(string: "http://localhost:6062/covers/" + data.tracks[currentIndex].albumArt), duration: TimeInterval(data.tracks[currentIndex].length / 1000), trackNumber: Int(data.tracks[currentIndex].tracknum), discNumber: Int(data.tracks[currentIndex].discnum), color: .gray.opacity(0.3))
                    let globalStatus = try await fetchGlobalStatus()
                    self.currentTime = TimeInterval(globalStatus.resumeElapsed / 1000)
                    self.duration = TimeInterval(data.tracks[currentIndex].length / 1000)
                }
            } catch {
                self.error = error
            }
        }
    }
    
    func playPreviousTrack() {
        Task {
            do {
                try await previous()
            } catch {
                self.error = error
            }
        }

    }
    
    func playNextTrack() {
        Task {
            do {
                try await next()
            } catch {
                self.error = error
            }
        }
    }
    
    func playOrPause() {
        Task {
            do {
                let globalStatus = try await fetchGlobalStatus()
                if globalStatus.resumeIndex > -1 && status == 0 {
                    try await resumeTrack()
                    return
                }
                
                if isPlaying {
                    try await pause()
                    try? await Task.sleep(for: .seconds(1))
                    return
                }
                
                try await resume()
            } catch {
                self.error = error
            }
        }
    }
    
    func seek(position: Int64) {
        self.currentTime = TimeInterval(position / 1000)
        Task {
            do {
                try await play(elapsed: position)
            } catch {
                self.error = error
            }
        }
    }
    
 }
