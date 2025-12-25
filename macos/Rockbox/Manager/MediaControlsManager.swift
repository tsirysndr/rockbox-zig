//
//  MediaControlsManager.swift
//  Rockbox
//
//  Created by Tsiry Sandratraina on 25/12/2025.
//
import MediaPlayer
import AppKit

class MediaControlsManager {
    
    static let shared = MediaControlsManager()
    
    var onPlay: (() -> Void)?
    var onPause: (() -> Void)?
    var onNext: (() -> Void)?
    var onPrevious: (() -> Void)?
    var onSeek: ((TimeInterval) -> Void)?
    
    private init() {
        setupRemoteCommandCenter()
    }
    
    func setupRemoteCommandCenter() {
        let commandCenter = MPRemoteCommandCenter.shared()
        
        commandCenter.playCommand.isEnabled = true
        commandCenter.playCommand.addTarget { [weak self] event in
            self?.onPlay?()
            return .success
        }
        
        commandCenter.pauseCommand.isEnabled = true
        commandCenter.pauseCommand.addTarget { [weak self] event in
            self?.onPause?()
            return .success
        }
        
        commandCenter.togglePlayPauseCommand.isEnabled = true
        commandCenter.togglePlayPauseCommand.addTarget { [weak self] event in
            return .success
        }
        
        commandCenter.nextTrackCommand.isEnabled = true
        commandCenter.nextTrackCommand.addTarget { [weak self] event in
            self?.onNext?()
            return .success
        }
        
        commandCenter.previousTrackCommand.isEnabled = true
        commandCenter.previousTrackCommand.addTarget { [weak self] event in
            self?.onPrevious?()
            return .success
        }
        
        commandCenter.changePlaybackPositionCommand.isEnabled = true
        commandCenter.changePlaybackPositionCommand.addTarget { [weak self] event in
            if let event = event as? MPChangePlaybackPositionCommandEvent {
                self?.onSeek?(event.positionTime)
                return .success
            }
            return .commandFailed
        }
    }
    
    func updateNowPlaying(
        title: String,
        artist: String? = nil,
        album: String? = nil,
        artwork: NSImage? = nil, 
        duration: TimeInterval,
        currentTime: TimeInterval,
        isPlaying: Bool
    ) {
        var nowPlayingInfo = [String: Any]()
        
        nowPlayingInfo[MPMediaItemPropertyTitle] = title
        if let artist = artist {
            nowPlayingInfo[MPMediaItemPropertyArtist] = artist
        }
        if let album = album {
            nowPlayingInfo[MPMediaItemPropertyAlbumTitle] = album
        }
        if let artwork = artwork {
            nowPlayingInfo[MPMediaItemPropertyArtwork] = MPMediaItemArtwork(boundsSize: artwork.size) { _ in artwork }
        }
        
        nowPlayingInfo[MPMediaItemPropertyPlaybackDuration] = duration
        nowPlayingInfo[MPNowPlayingInfoPropertyElapsedPlaybackTime] = currentTime
        nowPlayingInfo[MPNowPlayingInfoPropertyPlaybackRate] = isPlaying ? 1.0 : 0.0
        
        MPNowPlayingInfoCenter.default().nowPlayingInfo = nowPlayingInfo
    }
}
