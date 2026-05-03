import ExpoModulesCore
import Foundation

// C symbols exported by the rockbox_expo Rust crate (built from
// `crates/expo/`). The static library is linked via the .podspec.
@_silgen_name("rb_set_server_url")  private func rb_set_server_url(_ url: UnsafePointer<CChar>) -> Int32
@_silgen_name("rb_set_http_url")    private func rb_set_http_url(_ url: UnsafePointer<CChar>) -> Int32
@_silgen_name("rb_ping")            private func rb_ping() -> Int32

@_silgen_name("rb_get_devices_json")    private func rb_get_devices_json() -> UnsafeMutablePointer<CChar>?
@_silgen_name("rb_connect_device")      private func rb_connect_device(_ id: UnsafePointer<CChar>) -> Int32
@_silgen_name("rb_disconnect_device")   private func rb_disconnect_device(_ id: UnsafePointer<CChar>) -> Int32
@_silgen_name("rb_play")            private func rb_play() -> Int32
@_silgen_name("rb_pause")           private func rb_pause() -> Int32
@_silgen_name("rb_play_pause")      private func rb_play_pause() -> Int32
@_silgen_name("rb_next")            private func rb_next() -> Int32
@_silgen_name("rb_prev")            private func rb_prev() -> Int32
@_silgen_name("rb_seek")            private func rb_seek(_ positionMs: Int32) -> Int32
@_silgen_name("rb_status_json")     private func rb_status_json() -> UnsafeMutablePointer<CChar>?
@_silgen_name("rb_current_track_json") private func rb_current_track_json() -> UnsafeMutablePointer<CChar>?
@_silgen_name("rb_like_track")      private func rb_like_track(_ id: UnsafePointer<CChar>) -> Int32
@_silgen_name("rb_unlike_track")    private func rb_unlike_track(_ id: UnsafePointer<CChar>) -> Int32
@_silgen_name("rb_free_string")     private func rb_free_string(_ ptr: UnsafeMutablePointer<CChar>?)

@_silgen_name("rb_subscribe_status")        private func rb_subscribe_status() -> Int32
@_silgen_name("rb_subscribe_current_track") private func rb_subscribe_current_track() -> Int32
@_silgen_name("rb_subscribe_playlist")      private func rb_subscribe_playlist() -> Int32
@_silgen_name("rb_subscribe_library")       private func rb_subscribe_library() -> Int32
@_silgen_name("rb_subscribe_discovery")     private func rb_subscribe_discovery(_ name: UnsafePointer<CChar>) -> Int32
@_silgen_name("rb_rockbox_service_name")    private func rb_rockbox_service_name() -> UnsafeMutablePointer<CChar>?
@_silgen_name("rb_chromecast_service_name") private func rb_chromecast_service_name() -> UnsafeMutablePointer<CChar>?
@_silgen_name("rb_poll_event")              private func rb_poll_event(_ subId: Int32, _ timeoutMs: Int32) -> UnsafeMutablePointer<CChar>?
@_silgen_name("rb_unsubscribe")             private func rb_unsubscribe(_ subId: Int32) -> Int32

@_silgen_name("rb_resume_track")        private func rb_resume_track() -> Int32
@_silgen_name("rb_playlist_resume")     private func rb_playlist_resume() -> Int32
@_silgen_name("rb_play_all_tracks")     private func rb_play_all_tracks() -> Int32
@_silgen_name("rb_play_track")          private func rb_play_track(_ p: UnsafePointer<CChar>) -> Int32
@_silgen_name("rb_play_album")          private func rb_play_album(_ id: UnsafePointer<CChar>, _ shuffle: Int32) -> Int32
@_silgen_name("rb_play_artist_tracks")  private func rb_play_artist_tracks(_ id: UnsafePointer<CChar>, _ shuffle: Int32) -> Int32
@_silgen_name("rb_play_directory")      private func rb_play_directory(_ p: UnsafePointer<CChar>, _ shuffle: Int32, _ position: Int32) -> Int32

@_silgen_name("rb_jump_to_queue_position") private func rb_jump_to_queue_position(_ p: Int32) -> Int32
@_silgen_name("rb_shuffle_playlist")       private func rb_shuffle_playlist() -> Int32
@_silgen_name("rb_insert_tracks")          private func rb_insert_tracks(_ paths: UnsafePointer<CChar>, _ pos: Int32, _ shuffle: Int32) -> Int32
@_silgen_name("rb_insert_track_next")      private func rb_insert_track_next(_ p: UnsafePointer<CChar>) -> Int32
@_silgen_name("rb_insert_track_last")      private func rb_insert_track_last(_ p: UnsafePointer<CChar>) -> Int32
@_silgen_name("rb_insert_directory")       private func rb_insert_directory(_ p: UnsafePointer<CChar>, _ pos: Int32) -> Int32
@_silgen_name("rb_remove_from_queue")      private func rb_remove_from_queue(_ pos: Int32) -> Int32
@_silgen_name("rb_get_playlist_current_json") private func rb_get_playlist_current_json() -> UnsafeMutablePointer<CChar>?

@_silgen_name("rb_get_tracks_json")         private func rb_get_tracks_json() -> UnsafeMutablePointer<CChar>?
@_silgen_name("rb_get_artists_json")        private func rb_get_artists_json() -> UnsafeMutablePointer<CChar>?
@_silgen_name("rb_get_albums_json")         private func rb_get_albums_json() -> UnsafeMutablePointer<CChar>?
@_silgen_name("rb_get_liked_albums_json")   private func rb_get_liked_albums_json() -> UnsafeMutablePointer<CChar>?
@_silgen_name("rb_get_artist_json")         private func rb_get_artist_json(_ id: UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar>?
@_silgen_name("rb_get_album_json")          private func rb_get_album_json(_ id: UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar>?
@_silgen_name("rb_get_liked_tracks_json")   private func rb_get_liked_tracks_json() -> UnsafeMutablePointer<CChar>?
@_silgen_name("rb_search_json")             private func rb_search_json(_ term: UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar>?

@_silgen_name("rb_adjust_volume")           private func rb_adjust_volume(_ steps: Int32) -> Int32
@_silgen_name("rb_sound_current_json")      private func rb_sound_current_json(_ setting: Int32) -> UnsafeMutablePointer<CChar>?

@_silgen_name("rb_save_shuffle")            private func rb_save_shuffle(_ enabled: Int32) -> Int32
@_silgen_name("rb_save_repeat")             private func rb_save_repeat(_ mode: Int32) -> Int32
@_silgen_name("rb_get_global_settings_json") private func rb_get_global_settings_json() -> UnsafeMutablePointer<CChar>?
@_silgen_name("rb_get_global_status_json")  private func rb_get_global_status_json() -> UnsafeMutablePointer<CChar>?

@_silgen_name("rb_tree_get_entries_json")   private func rb_tree_get_entries_json(_ p: UnsafePointer<CChar>?) -> UnsafeMutablePointer<CChar>?

@_silgen_name("rb_get_saved_playlists_json")        private func rb_get_saved_playlists_json() -> UnsafeMutablePointer<CChar>?
@_silgen_name("rb_create_saved_playlist")           private func rb_create_saved_playlist(_ name: UnsafePointer<CChar>, _ desc: UnsafePointer<CChar>?, _ ids: UnsafePointer<CChar>?) -> Int32
@_silgen_name("rb_update_saved_playlist")           private func rb_update_saved_playlist(_ id: UnsafePointer<CChar>, _ name: UnsafePointer<CChar>, _ desc: UnsafePointer<CChar>?) -> Int32
@_silgen_name("rb_delete_saved_playlist")           private func rb_delete_saved_playlist(_ id: UnsafePointer<CChar>) -> Int32
@_silgen_name("rb_add_track_to_playlist")           private func rb_add_track_to_playlist(_ pid: UnsafePointer<CChar>, _ tid: UnsafePointer<CChar>) -> Int32
@_silgen_name("rb_remove_track_from_playlist")      private func rb_remove_track_from_playlist(_ pid: UnsafePointer<CChar>, _ tid: UnsafePointer<CChar>) -> Int32
@_silgen_name("rb_get_saved_playlist_tracks_json")  private func rb_get_saved_playlist_tracks_json(_ pid: UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar>?
@_silgen_name("rb_play_saved_playlist")             private func rb_play_saved_playlist(_ pid: UnsafePointer<CChar>) -> Int32

@_silgen_name("rb_get_smart_playlists_json")        private func rb_get_smart_playlists_json() -> UnsafeMutablePointer<CChar>?
@_silgen_name("rb_get_smart_playlist_tracks_json")  private func rb_get_smart_playlist_tracks_json(_ id: UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar>?
@_silgen_name("rb_play_smart_playlist")             private func rb_play_smart_playlist(_ id: UnsafePointer<CChar>) -> Int32

@_silgen_name("rb_bluetooth_available")             private func rb_bluetooth_available() -> Int32
@_silgen_name("rb_scan_bluetooth")                  private func rb_scan_bluetooth() -> Int32
@_silgen_name("rb_get_bluetooth_devices_json")      private func rb_get_bluetooth_devices_json() -> UnsafeMutablePointer<CChar>?
@_silgen_name("rb_connect_bluetooth")               private func rb_connect_bluetooth(_ a: UnsafePointer<CChar>) -> Int32
@_silgen_name("rb_disconnect_bluetooth")            private func rb_disconnect_bluetooth(_ a: UnsafePointer<CChar>) -> Int32

private func takeString(_ ptr: UnsafeMutablePointer<CChar>?) -> String? {
    guard let ptr = ptr else { return nil }
    let s = String(cString: ptr)
    rb_free_string(ptr)
    return s
}

private func parseJson(_ s: String?) throws -> Any {
    guard let s = s, let data = s.data(using: .utf8) else {
        throw NSError(domain: "RockboxRpc", code: -1, userInfo: [NSLocalizedDescriptionKey: "no response"])
    }
    let obj = try JSONSerialization.jsonObject(with: data, options: [])
    if let dict = obj as? [String: Any], let err = dict["error"] as? String {
        throw NSError(domain: "RockboxRpc", code: -2, userInfo: [NSLocalizedDescriptionKey: err])
    }
    return obj
}

public class RockboxRpcModule: Module {
    private let pollQueue = DispatchQueue(label: "rockbox.rpc.poll", qos: .utility, attributes: .concurrent)
    private var pollTokens: [Int32: Bool] = [:]   // subId → keepRunning
    private let pollLock = NSLock()

    public func definition() -> ModuleDefinition {
        Name("RockboxRpc")

        Events(
            "rockbox.status",
            "rockbox.currentTrack",
            "rockbox.playlist",
            "rockbox.library",
            "rockbox.discovery",
            "rockbox.error"
        )

        Function("rockboxServiceName") { () -> String in
            takeString(rb_rockbox_service_name()) ?? "_rockbox._tcp.local."
        }
        Function("chromecastServiceName") { () -> String in
            takeString(rb_chromecast_service_name()) ?? "_googlecast._tcp.local."
        }

        Function("setServerUrl") { (url: String) in
            url.withCString { _ = rb_set_server_url($0) }
        }
        Function("setHttpUrl") { (url: String) in
            url.withCString { _ = rb_set_http_url($0) }
        }
        AsyncFunction("getDevices") { () -> Any in
            try self.parseJsonOrThrow(takeString(rb_get_devices_json()), op: "getDevices")
        }
        AsyncFunction("connectDevice") { (id: String) -> Void in
            let rc = id.withCString { rb_connect_device($0) }
            if rc != 0 { throw self.playbackError("connectDevice") }
        }
        AsyncFunction("disconnectDevice") { (id: String) -> Void in
            let rc = id.withCString { rb_disconnect_device($0) }
            if rc != 0 { throw self.playbackError("disconnectDevice") }
        }

        AsyncFunction("ping") { () -> Bool in
            return rb_ping() == 0
        }

        AsyncFunction("play")      { () -> Void in if rb_play() != 0      { throw playbackError("play") } }
        AsyncFunction("pause")     { () -> Void in if rb_pause() != 0     { throw playbackError("pause") } }
        AsyncFunction("playPause") { () -> Void in if rb_play_pause() != 0 { throw playbackError("playPause") } }
        AsyncFunction("next")      { () -> Void in if rb_next() != 0      { throw playbackError("next") } }
        AsyncFunction("prev")      { () -> Void in if rb_prev() != 0      { throw playbackError("prev") } }
        AsyncFunction("seek") { (positionMs: Int) -> Void in
            if rb_seek(Int32(positionMs)) != 0 { throw playbackError("seek") }
        }

        AsyncFunction("status") { () -> [String: Any] in
            let s = takeString(rb_status_json())
            guard let dict = try parseJson(s) as? [String: Any] else {
                throw playbackError("status")
            }
            return dict
        }

        AsyncFunction("currentTrack") { () -> [String: Any] in
            let s = takeString(rb_current_track_json())
            guard let dict = try parseJson(s) as? [String: Any] else {
                throw playbackError("currentTrack")
            }
            return dict
        }

        AsyncFunction("likeTrack") { (id: String) -> Void in
            let rc = id.withCString { rb_like_track($0) }
            if rc != 0 { throw playbackError("likeTrack") }
        }

        AsyncFunction("unlikeTrack") { (id: String) -> Void in
            let rc = id.withCString { rb_unlike_track($0) }
            if rc != 0 { throw playbackError("unlikeTrack") }
        }

        // ── Comprehensive RPC surface ───────────────────────────────────────
        AsyncFunction("resumeTrack")    { () -> Void in if rb_resume_track() != 0     { throw self.playbackError("resumeTrack") } }
        AsyncFunction("playlistResume") { () -> Void in if rb_playlist_resume() != 0  { throw self.playbackError("playlistResume") } }
        AsyncFunction("playAllTracks")  { () -> Void in if rb_play_all_tracks() != 0  { throw self.playbackError("playAllTracks") } }

        AsyncFunction("playTrack") { (path: String) -> Void in
            let rc = path.withCString { rb_play_track($0) }
            if rc != 0 { throw self.playbackError("playTrack") }
        }
        AsyncFunction("playAlbum") { (id: String, shuffle: Bool) -> Void in
            let rc = id.withCString { rb_play_album($0, shuffle ? 1 : 0) }
            if rc != 0 { throw self.playbackError("playAlbum") }
        }
        AsyncFunction("playArtistTracks") { (id: String, shuffle: Bool) -> Void in
            let rc = id.withCString { rb_play_artist_tracks($0, shuffle ? 1 : 0) }
            if rc != 0 { throw self.playbackError("playArtistTracks") }
        }
        AsyncFunction("playDirectory") { (path: String, shuffle: Bool, position: Int) -> Void in
            let rc = path.withCString { rb_play_directory($0, shuffle ? 1 : 0, Int32(position)) }
            if rc != 0 { throw self.playbackError("playDirectory") }
        }

        AsyncFunction("jumpToQueuePosition") { (pos: Int) -> Void in
            if rb_jump_to_queue_position(Int32(pos)) != 0 { throw self.playbackError("jumpToQueuePosition") }
        }
        AsyncFunction("shufflePlaylist") { () -> Void in
            if rb_shuffle_playlist() != 0 { throw self.playbackError("shufflePlaylist") }
        }
        AsyncFunction("insertTracks") { (paths: [String], position: Int, shuffle: Bool) -> Void in
            let json = (try? JSONSerialization.data(withJSONObject: paths))
                .flatMap { String(data: $0, encoding: .utf8) } ?? "[]"
            let rc = json.withCString { rb_insert_tracks($0, Int32(position), shuffle ? 1 : 0) }
            if rc != 0 { throw self.playbackError("insertTracks") }
        }
        AsyncFunction("insertTrackNext") { (path: String) -> Void in
            let rc = path.withCString { rb_insert_track_next($0) }
            if rc != 0 { throw self.playbackError("insertTrackNext") }
        }
        AsyncFunction("insertTrackLast") { (path: String) -> Void in
            let rc = path.withCString { rb_insert_track_last($0) }
            if rc != 0 { throw self.playbackError("insertTrackLast") }
        }
        AsyncFunction("insertDirectory") { (path: String, position: Int) -> Void in
            let rc = path.withCString { rb_insert_directory($0, Int32(position)) }
            if rc != 0 { throw self.playbackError("insertDirectory") }
        }
        AsyncFunction("removeFromQueue") { (pos: Int) -> Void in
            if rb_remove_from_queue(Int32(pos)) != 0 { throw self.playbackError("removeFromQueue") }
        }

        AsyncFunction("getPlaylistCurrent") { () -> Any in
            try self.parseJsonOrThrow(takeString(rb_get_playlist_current_json()), op: "getPlaylistCurrent")
        }
        AsyncFunction("getTracks") { () -> Any in
            try self.parseJsonOrThrow(takeString(rb_get_tracks_json()), op: "getTracks")
        }
        AsyncFunction("getArtists") { () -> Any in
            try self.parseJsonOrThrow(takeString(rb_get_artists_json()), op: "getArtists")
        }
        AsyncFunction("getAlbums") { () -> Any in
            try self.parseJsonOrThrow(takeString(rb_get_albums_json()), op: "getAlbums")
        }
        AsyncFunction("getLikedAlbums") { () -> Any in
            try self.parseJsonOrThrow(takeString(rb_get_liked_albums_json()), op: "getLikedAlbums")
        }
        AsyncFunction("getArtist") { (id: String) -> Any in
            let raw = id.withCString { rb_get_artist_json($0) }
            return try self.parseJsonOrThrow(takeString(raw), op: "getArtist")
        }
        AsyncFunction("getAlbum") { (id: String) -> Any in
            let raw = id.withCString { rb_get_album_json($0) }
            return try self.parseJsonOrThrow(takeString(raw), op: "getAlbum")
        }
        AsyncFunction("getLikedTracks") { () -> Any in
            try self.parseJsonOrThrow(takeString(rb_get_liked_tracks_json()), op: "getLikedTracks")
        }
        AsyncFunction("search") { (term: String) -> Any in
            let raw = term.withCString { rb_search_json($0) }
            return try self.parseJsonOrThrow(takeString(raw), op: "search")
        }

        AsyncFunction("adjustVolume") { (steps: Int) -> Void in
            if rb_adjust_volume(Int32(steps)) != 0 { throw self.playbackError("adjustVolume") }
        }
        AsyncFunction("soundCurrent") { (setting: Int) -> Any in
            try self.parseJsonOrThrow(takeString(rb_sound_current_json(Int32(setting))), op: "soundCurrent")
        }

        AsyncFunction("saveShuffle") { (enabled: Bool) -> Void in
            if rb_save_shuffle(enabled ? 1 : 0) != 0 { throw self.playbackError("saveShuffle") }
        }
        AsyncFunction("saveRepeat") { (mode: Int) -> Void in
            if rb_save_repeat(Int32(mode)) != 0 { throw self.playbackError("saveRepeat") }
        }
        AsyncFunction("getGlobalSettings") { () -> Any in
            try self.parseJsonOrThrow(takeString(rb_get_global_settings_json()), op: "getGlobalSettings")
        }
        AsyncFunction("getGlobalStatus") { () -> Any in
            try self.parseJsonOrThrow(takeString(rb_get_global_status_json()), op: "getGlobalStatus")
        }

        AsyncFunction("treeGetEntries") { (path: String?) -> Any in
            let raw: UnsafeMutablePointer<CChar>?
            if let p = path {
                raw = p.withCString { rb_tree_get_entries_json($0) }
            } else {
                raw = rb_tree_get_entries_json(nil)
            }
            return try self.parseJsonOrThrow(takeString(raw), op: "treeGetEntries")
        }

        AsyncFunction("getSavedPlaylists") { () -> Any in
            try self.parseJsonOrThrow(takeString(rb_get_saved_playlists_json()), op: "getSavedPlaylists")
        }
        AsyncFunction("createSavedPlaylist") { (name: String, description: String?, trackIds: [String]) -> Void in
            let idsJson = (try? JSONSerialization.data(withJSONObject: trackIds))
                .flatMap { String(data: $0, encoding: .utf8) } ?? "[]"
            let rc: Int32 = name.withCString { namePtr in
                idsJson.withCString { idsPtr in
                    if let d = description {
                        return d.withCString { descPtr in rb_create_saved_playlist(namePtr, descPtr, idsPtr) }
                    } else {
                        return rb_create_saved_playlist(namePtr, nil, idsPtr)
                    }
                }
            }
            if rc != 0 { throw self.playbackError("createSavedPlaylist") }
        }
        AsyncFunction("updateSavedPlaylist") { (id: String, name: String, description: String?) -> Void in
            let rc: Int32 = id.withCString { idPtr in
                name.withCString { namePtr in
                    if let d = description {
                        return d.withCString { descPtr in rb_update_saved_playlist(idPtr, namePtr, descPtr) }
                    } else {
                        return rb_update_saved_playlist(idPtr, namePtr, nil)
                    }
                }
            }
            if rc != 0 { throw self.playbackError("updateSavedPlaylist") }
        }
        AsyncFunction("deleteSavedPlaylist") { (id: String) -> Void in
            let rc = id.withCString { rb_delete_saved_playlist($0) }
            if rc != 0 { throw self.playbackError("deleteSavedPlaylist") }
        }
        AsyncFunction("addTrackToPlaylist") { (playlistId: String, trackId: String) -> Void in
            let rc: Int32 = playlistId.withCString { pid in
                trackId.withCString { tid in rb_add_track_to_playlist(pid, tid) }
            }
            if rc != 0 { throw self.playbackError("addTrackToPlaylist") }
        }
        AsyncFunction("removeTrackFromPlaylist") { (playlistId: String, trackId: String) -> Void in
            let rc: Int32 = playlistId.withCString { pid in
                trackId.withCString { tid in rb_remove_track_from_playlist(pid, tid) }
            }
            if rc != 0 { throw self.playbackError("removeTrackFromPlaylist") }
        }
        AsyncFunction("getSavedPlaylistTracks") { (playlistId: String) -> Any in
            let raw = playlistId.withCString { rb_get_saved_playlist_tracks_json($0) }
            return try self.parseJsonOrThrow(takeString(raw), op: "getSavedPlaylistTracks")
        }
        AsyncFunction("playSavedPlaylist") { (playlistId: String) -> Void in
            let rc = playlistId.withCString { rb_play_saved_playlist($0) }
            if rc != 0 { throw self.playbackError("playSavedPlaylist") }
        }

        AsyncFunction("getSmartPlaylists") { () -> Any in
            try self.parseJsonOrThrow(takeString(rb_get_smart_playlists_json()), op: "getSmartPlaylists")
        }
        AsyncFunction("getSmartPlaylistTracks") { (id: String) -> Any in
            let raw = id.withCString { rb_get_smart_playlist_tracks_json($0) }
            return try self.parseJsonOrThrow(takeString(raw), op: "getSmartPlaylistTracks")
        }
        AsyncFunction("playSmartPlaylist") { (id: String) -> Void in
            let rc = id.withCString { rb_play_smart_playlist($0) }
            if rc != 0 { throw self.playbackError("playSmartPlaylist") }
        }

        AsyncFunction("bluetoothAvailable") { () -> Bool in
            return rb_bluetooth_available() == 1
        }
        AsyncFunction("scanBluetooth") { () -> Void in
            if rb_scan_bluetooth() != 0 { throw self.playbackError("scanBluetooth") }
        }
        AsyncFunction("getBluetoothDevices") { () -> Any in
            try self.parseJsonOrThrow(takeString(rb_get_bluetooth_devices_json()), op: "getBluetoothDevices")
        }
        AsyncFunction("connectBluetooth") { (address: String) -> Void in
            let rc = address.withCString { rb_connect_bluetooth($0) }
            if rc != 0 { throw self.playbackError("connectBluetooth") }
        }
        AsyncFunction("disconnectBluetooth") { (address: String) -> Void in
            let rc = address.withCString { rb_disconnect_bluetooth($0) }
            if rc != 0 { throw self.playbackError("disconnectBluetooth") }
        }

        // ── Streaming subscriptions ─────────────────────────────────────────
        Function("subscribeStatus") { () -> Int32 in
            let id = rb_subscribe_status()
            self.startPollLoop(subId: id, eventName: "rockbox.status")
            return id
        }
        Function("subscribeCurrentTrack") { () -> Int32 in
            let id = rb_subscribe_current_track()
            self.startPollLoop(subId: id, eventName: "rockbox.currentTrack")
            return id
        }
        Function("subscribePlaylist") { () -> Int32 in
            let id = rb_subscribe_playlist()
            self.startPollLoop(subId: id, eventName: "rockbox.playlist")
            return id
        }
        Function("subscribeLibrary") { () -> Int32 in
            let id = rb_subscribe_library()
            self.startPollLoop(subId: id, eventName: "rockbox.library")
            return id
        }
        Function("subscribeDiscovery") { (serviceName: String) -> Int32 in
            let id = serviceName.withCString { rb_subscribe_discovery($0) }
            self.startPollLoop(subId: id, eventName: "rockbox.discovery")
            return id
        }
        Function("unsubscribe") { (subId: Int32) -> Void in
            self.pollLock.lock()
            self.pollTokens[subId] = false
            self.pollLock.unlock()
            _ = rb_unsubscribe(subId)
        }
    }

    private func startPollLoop(subId: Int32, eventName: String) {
        pollLock.lock()
        pollTokens[subId] = true
        pollLock.unlock()

        pollQueue.async { [weak self] in
            while true {
                guard let self = self else { return }
                self.pollLock.lock()
                let alive = self.pollTokens[subId] ?? false
                self.pollLock.unlock()
                if !alive { return }

                guard let raw = rb_poll_event(subId, 1_000) else {
                    // Timeout (no event in 1s) — keep polling.
                    continue
                }
                let s = String(cString: raw)
                rb_free_string(raw)
                guard let data = s.data(using: .utf8),
                      let obj = try? JSONSerialization.jsonObject(with: data) else {
                    continue
                }
                if let dict = obj as? [String: Any] {
                    if let err = dict["error"] as? String {
                        self.sendEvent("rockbox.error", ["subId": subId, "error": err, "stream": eventName])
                        // Stop polling after an error.
                        self.pollLock.lock()
                        self.pollTokens[subId] = false
                        self.pollLock.unlock()
                        return
                    }
                    self.sendEvent(eventName, dict)
                }
            }
        }
    }

    private func playbackError(_ op: String) -> NSError {
        return NSError(domain: "RockboxRpc", code: -1,
                       userInfo: [NSLocalizedDescriptionKey: "rockbox-rpc: \(op) failed"])
    }

    fileprivate func parseJsonOrThrow(_ s: String?, op: String) throws -> Any {
        guard let s = s, let data = s.data(using: .utf8) else {
            throw playbackError(op)
        }
        let obj = try JSONSerialization.jsonObject(with: data, options: .allowFragments)
        if let dict = obj as? [String: Any], let err = dict["error"] as? String {
            throw NSError(domain: "RockboxRpc", code: -2,
                          userInfo: [NSLocalizedDescriptionKey: "rockbox-rpc: \(op): \(err)"])
        }
        return obj
    }
}
