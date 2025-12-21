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
    @Published var currentTrack = Song(cuid: "", path: "", title: "Not Playing", artist: "", album: "", albumArt: nil, duration: TimeInterval(0), trackNumber: 0, discNumber: 0, color: .gray.opacity(0.3))
    @Published var queue: [Song] = []
    @Published var currentIndex: Int = 0
    @Published var playlistLength: Int = 0
    @Published var isConnected = false
    @Published var error: Error?
    @Published var status: Int32 = 0
    
    private var streamTask: Task<Void, Never>?
    private var streamStatusTask: Task<Void, Never>?
    private var streamPlaylistTask: Task<Void, Never>?
    
    var progress: Double {
        get { duration > 0 ? currentTime / duration : 0 }
        set { currentTime = newValue * duration }
    }
    
    // Upcoming tracks (after current)
    var upNext: [Song] {
        guard currentIndex + 1 < queue.count else { return [] }
        return Array(queue[(currentIndex + 1)...])
    }
    
    // Previous tracks (before current + current)
    var history: [Song] {
        if currentIndex == 0 && queue.count > 0 {
            return Array(queue[...currentIndex])
        }
        guard currentIndex > 0 else { return [] }
        return Array(queue[...currentIndex])
    }
        
    func startStreaming() {
        getCurrentTrack()
        streamTask?.cancel()
        streamTask = Task {
            do {
                isConnected = true
                for try await response in currentTrackStream() {
                    self.currentTrack = Song(
                        cuid: response.id,
                        path: response.path,
                        title: response.title,
                        artist: response.artist,
                        album: response.album,
                        albumArt: URL(string: "http://localhost:6062/covers/" + response.albumArt),
                        duration: TimeInterval(response.length / 1000),
                        trackNumber: Int(response.tracknum),
                        discNumber: Int(response.discnum),
                        color: .gray.opacity(0.3)
                    )
                    self.duration = TimeInterval(response.length / 1000)
                    self.currentTime = TimeInterval(response.elapsed / 1000)                    
                }
                // Refresh queue when track changes
                await self.fetchQueue()

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
                // Ignored
            } catch {
                self.error = error
            }
        }
        
        streamPlaylistTask?.cancel()
        streamPlaylistTask = Task {
            do {
                for try await data in currentPlaylistStream() {
                    if self.currentIndex == Int(data.index) { continue }
                    self.currentIndex = Int(data.index)
                    self.playlistLength = Int(data.amount)
                    self.queue = data.tracks.map { track in
                        Song(
                            cuid: track.id,
                            path: track.path,
                            title: track.title,
                            artist: track.artist,
                            album: track.album,
                            albumArt: URL(string: "http://localhost:6062/covers/" + track.albumArt),
                            duration: TimeInterval(track.length / 1000),
                            trackNumber: Int(track.tracknum),
                            discNumber: Int(track.discnum),
                            color: .gray.opacity(0.3)
                        )
                    }
                    
                    self.currentTrack = self.queue[self.currentIndex]
                }
            } catch is CancellationError {
                // Ignored
            } catch {
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
                    let index = Int(data.index)
                    self.currentIndex = index
                    self.playlistLength = Int(data.amount)
                    
                    self.queue = data.tracks.map { track in
                        Song(
                            cuid: track.id,
                            path: track.path,
                            title: track.title,
                            artist: track.artist,
                            album: track.album,
                            albumArt: URL(string: "http://localhost:6062/covers/" + track.albumArt),
                            duration: TimeInterval(track.length / 1000),
                            trackNumber: Int(track.tracknum),
                            discNumber: Int(track.discnum),
                            color: .gray.opacity(0.3)
                        )
                    }
                    
                    self.currentTrack = self.queue[index]
                    
                    let globalStatus = try await fetchGlobalStatus()
                    self.currentTime = TimeInterval(globalStatus.resumeElapsed / 1000)
                    self.duration = TimeInterval(data.tracks[index].length / 1000)
                }
            } catch {
                self.error = error
            }
        }
    }
    
    func fetchQueue() async {
        do {
            let data = try await fetchCurrentPlaylist()
            if data.tracks.count > 0 {
                self.currentIndex = Int(data.index)
                self.playlistLength = Int(data.amount)
                self.queue = data.tracks.map { track in
                    Song(
                        cuid: track.id,
                        path: track.path,
                        title: track.title,
                        artist: track.artist,
                        album: track.album,
                        albumArt: URL(string: "http://localhost:6062/covers/" + track.albumArt),
                        duration: TimeInterval(track.length / 1000),
                        trackNumber: Int(track.tracknum),
                        discNumber: Int(track.discnum),
                        color: .gray.opacity(0.3)
                    )
                }
            }
        } catch {
            self.error = error
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
    
    func playFromQueue(at index: Int) {
        Task {
            do {
               try await startPlaylist(position: Int32(index))
                self.currentIndex = index
                self.currentTrack = queue[index]
            } catch {
                self.error = error
            }
        }
    }
    
    func removeFromQueue(at index: Int) {
        Task {
            do {
               try await removeFromPlaylist(position: Int32(index))
               await fetchQueue()
            } catch {
                self.error = error
            }
        }
    }
    
    func clearQueue() {
        Task {
            do {
                try await clearPlaylist()
                await fetchQueue()
            } catch {
                self.error = error
            }
        }
    }
}
