//
//  PlayerState.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 14/12/2025.
//
import Foundation
import SwiftUI

enum RepeatMode {
  case off
  case all
  case one
}

@MainActor
class PlayerState: ObservableObject {
  @Published var isPlaying = false
  @Published var currentTime: TimeInterval = 0
  @Published var duration: TimeInterval = 0
  @Published var currentTrack = Song(
    cuid: "", path: "", title: "Not Playing", artist: "", album: "", albumArt: nil,
    duration: TimeInterval(0), trackNumber: 0, discNumber: 0, albumID: "", artistID: "",
    color: .gray.opacity(0.3))
  @Published var queue: [Song] = []
  @Published var currentIndex: Int = 0
  @Published var playlistLength: Int = 0
  @Published var isConnected = false
  @Published var error: Error?
  @Published var status: Int32 = 0
  @Published var isShuffleEnabled: Bool = false
  @Published var repeatMode: RepeatMode = .off

  private var streamTask: Task<Void, Never>?
  private var streamStatusTask: Task<Void, Never>?
  private var streamPlaylistTask: Task<Void, Never>?

  var progress: Double {
    get { duration > 0 ? currentTime / duration : 0 }
    set { currentTime = newValue * duration }
  }

  init() {
    setupMediaControls()
    setInitialNowPlayingInfo()
  }

  private func setupMediaControls() {
    let manager = MediaControlsManager.shared
    manager.onPlay = {
      Task {
        try? await resume()
      }
    }

    manager.onPause = {
      Task {
        try? await pause()
      }
    }

    manager.onNext = {
      Task {
        try? await next()
      }
    }

    manager.onPrevious = {
      Task {
        try? await previous()
      }
    }

    manager.onSeek = { position in
      Task {
        try? await play(elapsed: Int64(position) * 1000)
      }
    }

  }

  private func setInitialNowPlayingInfo() {
    MediaControlsManager.shared.updateNowPlaying(
      title: "Not Playing",
      artist: "Rockbox",
      album: nil,
      artwork: nil,
      duration: 0,
      currentTime: 0,
      isPlaying: false
    )
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
          let previousTrack = self.currentTrack
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
            albumID: response.albumID,
            artistID: response.artistID,
            color: .gray.opacity(0.3)
          )
          self.duration = TimeInterval(response.length / 1000)
          self.currentTime = TimeInterval(response.elapsed / 1000)

          if previousTrack.cuid != self.currentTrack.cuid {
            self.updateNowPlayingInfo()
          }
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
          let previousIsPlaying = self.isPlaying
          self.isPlaying = response.status == 1
          self.status = response.status

          if previousIsPlaying != self.isPlaying {
            self.updateNowPlayingInfo()
          }
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
              albumID: track.albumID,
              artistID: track.artistID,
              color: .gray.opacity(0.3)
            )
          }

          self.currentTrack = self.queue[self.currentIndex]
          self.updateNowPlayingInfo()
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
              albumID: track.albumID,
              artistID: track.artistID,
              color: .gray.opacity(0.3)
            )
          }

          self.currentTrack = self.queue[index]

          let globalStatus = try await fetchGlobalStatus()
          self.currentTime = TimeInterval(globalStatus.resumeElapsed / 1000)
          self.duration = TimeInterval(data.tracks[index].length / 1000)

          self.updateNowPlayingInfo()
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
            albumID: track.albumID,
            artistID: track.artistID,
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
        self.updateNowPlayingInfo()
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
        self.updateNowPlayingInfo()
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

  func toggleShuffle() {
    isShuffleEnabled.toggle()
    Task {
      do {
        try await updatePlaylistShuffle(enabled: isShuffleEnabled)
        fetchSettings()
        await fetchQueue()
      } catch {
        self.error = error
        isShuffleEnabled.toggle()
      }
    }
  }

  func toggleRepeat() {
    var mode: Int32 = 0
    switch repeatMode {
    case .off:
      mode = 2
    case .all:
      mode = 1
    case .one:
      mode = 0
    }

    Task {
      do {
        try await updateRepeatMode(repeatMode: mode)
        fetchSettings()
      } catch {
        self.error = error
      }
    }
  }

  func fetchSettings() {
    Task {
      do {
        let data = try await fetchGlobalSettings()
        switch data.repeatMode {
        case 0:
          repeatMode = .off
        case 1:
          repeatMode = .one
        case 2:
          repeatMode = .all
        default:
          repeatMode = .off
        }
        isShuffleEnabled = data.playlistShuffle
      } catch {
        self.error = error
      }
    }
  }

  // MARK: - Update Now Playing Info

  func updateNowPlayingInfo() {
    // Load artwork asynchronously
    loadArtwork(from: currentTrack.albumArt) { [weak self] artwork in
      guard let self = self else { return }

      MediaControlsManager.shared.updateNowPlaying(
        title: self.currentTrack.title,
        artist: self.currentTrack.artist,
        album: self.currentTrack.album,
        artwork: artwork,
        duration: self.duration,
        currentTime: self.currentTime,
        isPlaying: self.isPlaying
      )
    }
  }

  private func loadArtwork(from url: URL?, completion: @escaping (NSImage?) -> Void) {
    guard let url = url else {
      completion(nil)
      return
    }

    DispatchQueue.global(qos: .userInitiated).async {
      let image: NSImage?

      if url.isFileURL {
        image = NSImage(contentsOf: url)
      } else {
        if let data = try? Data(contentsOf: url) {
          image = NSImage(data: data)
        } else {
          image = nil
        }
      }

      DispatchQueue.main.async {
        completion(image)
      }
    }
  }
}
