package expo.modules.rockboxrpc

import expo.modules.kotlin.modules.Module
import expo.modules.kotlin.modules.ModuleDefinition
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.cancel
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import org.json.JSONObject
import java.util.concurrent.ConcurrentHashMap

/**
 * Native module wrapping the rockbox-expo Rust crate (built into a .so per ABI
 * and dropped into android/src/main/jniLibs/).
 *
 * The C ABI is a 1:1 mapping of `rb_*` functions in `crates/expo/src/lib.rs`.
 * String returns are heap-owned in Rust and freed by `rb_free_string` on the
 * Kotlin side after copying.
 */
class RockboxRpcModule : Module() {
  companion object {
    init {
      System.loadLibrary("rockbox_expo")
    }

    @JvmStatic external fun rb_set_server_url(url: String): Int
    @JvmStatic external fun rb_ping(): Int
    @JvmStatic external fun rb_play(): Int
    @JvmStatic external fun rb_pause(): Int
    @JvmStatic external fun rb_play_pause(): Int
    @JvmStatic external fun rb_next(): Int
    @JvmStatic external fun rb_prev(): Int
    @JvmStatic external fun rb_seek(positionMs: Int): Int
    @JvmStatic external fun rb_status_json(): String?
    @JvmStatic external fun rb_current_track_json(): String?
    @JvmStatic external fun rb_like_track(id: String): Int
    @JvmStatic external fun rb_unlike_track(id: String): Int

    @JvmStatic external fun rb_subscribe_status(): Int
    @JvmStatic external fun rb_subscribe_current_track(): Int
    @JvmStatic external fun rb_subscribe_playlist(): Int
    @JvmStatic external fun rb_subscribe_library(): Int
    @JvmStatic external fun rb_subscribe_discovery(serviceName: String): Int
    @JvmStatic external fun rb_rockbox_service_name(): String?
    @JvmStatic external fun rb_chromecast_service_name(): String?
    @JvmStatic external fun rb_poll_event(subId: Int, timeoutMs: Int): String?
    @JvmStatic external fun rb_unsubscribe(subId: Int): Int

    @JvmStatic external fun rb_resume_track(): Int
    @JvmStatic external fun rb_playlist_resume(): Int
    @JvmStatic external fun rb_play_all_tracks(): Int
    @JvmStatic external fun rb_play_track(path: String): Int
    @JvmStatic external fun rb_play_album(id: String, shuffle: Int): Int
    @JvmStatic external fun rb_play_artist_tracks(id: String, shuffle: Int): Int
    @JvmStatic external fun rb_play_directory(path: String, shuffle: Int, position: Int): Int

    @JvmStatic external fun rb_jump_to_queue_position(pos: Int): Int
    @JvmStatic external fun rb_shuffle_playlist(): Int
    @JvmStatic external fun rb_insert_tracks(pathsJson: String, position: Int, shuffle: Int): Int
    @JvmStatic external fun rb_insert_track_next(path: String): Int
    @JvmStatic external fun rb_insert_track_last(path: String): Int
    @JvmStatic external fun rb_insert_directory(path: String, position: Int): Int
    @JvmStatic external fun rb_remove_from_queue(pos: Int): Int
    @JvmStatic external fun rb_get_playlist_current_json(): String?

    @JvmStatic external fun rb_get_tracks_json(): String?
    @JvmStatic external fun rb_get_artists_json(): String?
    @JvmStatic external fun rb_get_album_json(id: String): String?
    @JvmStatic external fun rb_get_liked_tracks_json(): String?
    @JvmStatic external fun rb_search_json(term: String): String?

    @JvmStatic external fun rb_adjust_volume(steps: Int): Int
    @JvmStatic external fun rb_sound_current_json(setting: Int): String?

    @JvmStatic external fun rb_save_shuffle(enabled: Int): Int
    @JvmStatic external fun rb_save_repeat(mode: Int): Int
    @JvmStatic external fun rb_get_global_settings_json(): String?
    @JvmStatic external fun rb_get_global_status_json(): String?

    @JvmStatic external fun rb_tree_get_entries_json(path: String?): String?

    @JvmStatic external fun rb_get_saved_playlists_json(): String?
    @JvmStatic external fun rb_create_saved_playlist(name: String, description: String?, idsJson: String?): Int
    @JvmStatic external fun rb_update_saved_playlist(id: String, name: String, description: String?): Int
    @JvmStatic external fun rb_delete_saved_playlist(id: String): Int
    @JvmStatic external fun rb_add_track_to_playlist(playlistId: String, trackId: String): Int
    @JvmStatic external fun rb_remove_track_from_playlist(playlistId: String, trackId: String): Int
    @JvmStatic external fun rb_get_saved_playlist_tracks_json(playlistId: String): String?
    @JvmStatic external fun rb_play_saved_playlist(playlistId: String): Int

    @JvmStatic external fun rb_get_smart_playlists_json(): String?
    @JvmStatic external fun rb_get_smart_playlist_tracks_json(id: String): String?
    @JvmStatic external fun rb_play_smart_playlist(id: String): Int

    @JvmStatic external fun rb_bluetooth_available(): Int
    @JvmStatic external fun rb_get_bluetooth_devices_json(): String?
    @JvmStatic external fun rb_connect_bluetooth(address: String): Int
    @JvmStatic external fun rb_disconnect_bluetooth(address: String): Int
  }

  private val scope = CoroutineScope(Dispatchers.IO)
  private val pollJobs = ConcurrentHashMap<Int, Job>()

  override fun definition() = ModuleDefinition {
    Name("RockboxRpc")

    Events(
      "rockbox.status",
      "rockbox.currentTrack",
      "rockbox.playlist",
      "rockbox.library",
      "rockbox.discovery",
      "rockbox.error",
    )

    Function("rockboxServiceName") {
      rb_rockbox_service_name() ?: "_rockbox._tcp.local."
    }
    Function("chromecastServiceName") {
      rb_chromecast_service_name() ?: "_googlecast._tcp.local."
    }

    Function("setServerUrl") { url: String ->
      rb_set_server_url(url)
    }

    AsyncFunction("ping") {
      rb_ping() == 0
    }

    AsyncFunction("play")      { if (rb_play() != 0)      throw RpcError("play") }
    AsyncFunction("pause")     { if (rb_pause() != 0)     throw RpcError("pause") }
    AsyncFunction("playPause") { if (rb_play_pause() != 0) throw RpcError("playPause") }
    AsyncFunction("next")      { if (rb_next() != 0)      throw RpcError("next") }
    AsyncFunction("prev")      { if (rb_prev() != 0)      throw RpcError("prev") }
    AsyncFunction("seek") { positionMs: Int ->
      if (rb_seek(positionMs) != 0) throw RpcError("seek")
    }

    AsyncFunction("status") {
      val json = rb_status_json() ?: throw RpcError("status")
      jsonToMap(json)
    }

    AsyncFunction("currentTrack") {
      val json = rb_current_track_json() ?: throw RpcError("currentTrack")
      jsonToMap(json)
    }

    AsyncFunction("likeTrack") { id: String ->
      if (rb_like_track(id) != 0) throw RpcError("likeTrack")
    }

    AsyncFunction("unlikeTrack") { id: String ->
      if (rb_unlike_track(id) != 0) throw RpcError("unlikeTrack")
    }

    // ── Comprehensive RPC surface ───────────────────────────────────────────
    AsyncFunction("resumeTrack")    { if (rb_resume_track() != 0)    throw RpcError("resumeTrack") }
    AsyncFunction("playlistResume") { if (rb_playlist_resume() != 0) throw RpcError("playlistResume") }
    AsyncFunction("playAllTracks")  { if (rb_play_all_tracks() != 0) throw RpcError("playAllTracks") }
    AsyncFunction("playTrack") { path: String ->
      if (rb_play_track(path) != 0) throw RpcError("playTrack")
    }
    AsyncFunction("playAlbum") { id: String, shuffle: Boolean ->
      if (rb_play_album(id, if (shuffle) 1 else 0) != 0) throw RpcError("playAlbum")
    }
    AsyncFunction("playArtistTracks") { id: String, shuffle: Boolean ->
      if (rb_play_artist_tracks(id, if (shuffle) 1 else 0) != 0) throw RpcError("playArtistTracks")
    }
    AsyncFunction("playDirectory") { path: String, shuffle: Boolean, position: Int ->
      if (rb_play_directory(path, if (shuffle) 1 else 0, position) != 0) throw RpcError("playDirectory")
    }

    AsyncFunction("jumpToQueuePosition") { pos: Int ->
      if (rb_jump_to_queue_position(pos) != 0) throw RpcError("jumpToQueuePosition")
    }
    AsyncFunction("shufflePlaylist") {
      if (rb_shuffle_playlist() != 0) throw RpcError("shufflePlaylist")
    }
    AsyncFunction("insertTracks") { paths: List<String>, position: Int, shuffle: Boolean ->
      val arr = org.json.JSONArray()
      for (p in paths) arr.put(p)
      if (rb_insert_tracks(arr.toString(), position, if (shuffle) 1 else 0) != 0)
        throw RpcError("insertTracks")
    }
    AsyncFunction("insertTrackNext") { path: String ->
      if (rb_insert_track_next(path) != 0) throw RpcError("insertTrackNext")
    }
    AsyncFunction("insertTrackLast") { path: String ->
      if (rb_insert_track_last(path) != 0) throw RpcError("insertTrackLast")
    }
    AsyncFunction("insertDirectory") { path: String, position: Int ->
      if (rb_insert_directory(path, position) != 0) throw RpcError("insertDirectory")
    }
    AsyncFunction("removeFromQueue") { pos: Int ->
      if (rb_remove_from_queue(pos) != 0) throw RpcError("removeFromQueue")
    }

    AsyncFunction("getPlaylistCurrent") {
      parseJsonOrThrow(rb_get_playlist_current_json(), "getPlaylistCurrent")
    }
    AsyncFunction("getTracks") {
      parseJsonOrThrow(rb_get_tracks_json(), "getTracks")
    }
    AsyncFunction("getArtists") {
      parseJsonOrThrow(rb_get_artists_json(), "getArtists")
    }
    AsyncFunction("getAlbum") { id: String ->
      parseJsonOrThrow(rb_get_album_json(id), "getAlbum")
    }
    AsyncFunction("getLikedTracks") {
      parseJsonOrThrow(rb_get_liked_tracks_json(), "getLikedTracks")
    }
    AsyncFunction("search") { term: String ->
      parseJsonOrThrow(rb_search_json(term), "search")
    }

    AsyncFunction("adjustVolume") { steps: Int ->
      if (rb_adjust_volume(steps) != 0) throw RpcError("adjustVolume")
    }
    AsyncFunction("soundCurrent") { setting: Int ->
      parseJsonOrThrow(rb_sound_current_json(setting), "soundCurrent")
    }

    AsyncFunction("saveShuffle") { enabled: Boolean ->
      if (rb_save_shuffle(if (enabled) 1 else 0) != 0) throw RpcError("saveShuffle")
    }
    AsyncFunction("saveRepeat") { mode: Int ->
      if (rb_save_repeat(mode) != 0) throw RpcError("saveRepeat")
    }
    AsyncFunction("getGlobalSettings") {
      parseJsonOrThrow(rb_get_global_settings_json(), "getGlobalSettings")
    }
    AsyncFunction("getGlobalStatus") {
      parseJsonOrThrow(rb_get_global_status_json(), "getGlobalStatus")
    }

    AsyncFunction("treeGetEntries") { path: String? ->
      parseJsonOrThrow(rb_tree_get_entries_json(path), "treeGetEntries")
    }

    AsyncFunction("getSavedPlaylists") {
      parseJsonOrThrow(rb_get_saved_playlists_json(), "getSavedPlaylists")
    }
    AsyncFunction("createSavedPlaylist") { name: String, description: String?, trackIds: List<String> ->
      val arr = org.json.JSONArray()
      for (id in trackIds) arr.put(id)
      if (rb_create_saved_playlist(name, description, arr.toString()) != 0)
        throw RpcError("createSavedPlaylist")
    }
    AsyncFunction("updateSavedPlaylist") { id: String, name: String, description: String? ->
      if (rb_update_saved_playlist(id, name, description) != 0) throw RpcError("updateSavedPlaylist")
    }
    AsyncFunction("deleteSavedPlaylist") { id: String ->
      if (rb_delete_saved_playlist(id) != 0) throw RpcError("deleteSavedPlaylist")
    }
    AsyncFunction("addTrackToPlaylist") { playlistId: String, trackId: String ->
      if (rb_add_track_to_playlist(playlistId, trackId) != 0) throw RpcError("addTrackToPlaylist")
    }
    AsyncFunction("removeTrackFromPlaylist") { playlistId: String, trackId: String ->
      if (rb_remove_track_from_playlist(playlistId, trackId) != 0) throw RpcError("removeTrackFromPlaylist")
    }
    AsyncFunction("getSavedPlaylistTracks") { playlistId: String ->
      parseJsonOrThrow(rb_get_saved_playlist_tracks_json(playlistId), "getSavedPlaylistTracks")
    }
    AsyncFunction("playSavedPlaylist") { playlistId: String ->
      if (rb_play_saved_playlist(playlistId) != 0) throw RpcError("playSavedPlaylist")
    }

    AsyncFunction("getSmartPlaylists") {
      parseJsonOrThrow(rb_get_smart_playlists_json(), "getSmartPlaylists")
    }
    AsyncFunction("getSmartPlaylistTracks") { id: String ->
      parseJsonOrThrow(rb_get_smart_playlist_tracks_json(id), "getSmartPlaylistTracks")
    }
    AsyncFunction("playSmartPlaylist") { id: String ->
      if (rb_play_smart_playlist(id) != 0) throw RpcError("playSmartPlaylist")
    }

    AsyncFunction("bluetoothAvailable") { rb_bluetooth_available() == 1 }
    AsyncFunction("getBluetoothDevices") {
      parseJsonOrThrow(rb_get_bluetooth_devices_json(), "getBluetoothDevices")
    }
    AsyncFunction("connectBluetooth") { address: String ->
      if (rb_connect_bluetooth(address) != 0) throw RpcError("connectBluetooth")
    }
    AsyncFunction("disconnectBluetooth") { address: String ->
      if (rb_disconnect_bluetooth(address) != 0) throw RpcError("disconnectBluetooth")
    }

    // ── Streaming subscriptions ─────────────────────────────────────────────
    Function("subscribeStatus") {
      val id = rb_subscribe_status()
      startPollLoop(id, "rockbox.status")
      id
    }
    Function("subscribeCurrentTrack") {
      val id = rb_subscribe_current_track()
      startPollLoop(id, "rockbox.currentTrack")
      id
    }
    Function("subscribePlaylist") {
      val id = rb_subscribe_playlist()
      startPollLoop(id, "rockbox.playlist")
      id
    }
    Function("subscribeLibrary") {
      val id = rb_subscribe_library()
      startPollLoop(id, "rockbox.library")
      id
    }
    Function("subscribeDiscovery") { serviceName: String ->
      val id = rb_subscribe_discovery(serviceName)
      startPollLoop(id, "rockbox.discovery")
      id
    }
    Function("unsubscribe") { subId: Int ->
      pollJobs.remove(subId)?.cancel()
      rb_unsubscribe(subId)
    }

    OnDestroy {
      pollJobs.values.forEach { it.cancel() }
      pollJobs.clear()
      scope.cancel()
    }
  }

  private fun startPollLoop(subId: Int, eventName: String) {
    val job = scope.launch {
      while (isActive) {
        val raw = rb_poll_event(subId, 1_000) ?: continue
        try {
          val obj = JSONObject(raw)
          if (obj.has("error")) {
            sendEvent("rockbox.error", mapOf(
              "subId" to subId,
              "error" to obj.getString("error"),
              "stream" to eventName,
            ))
            return@launch
          }
          val map = mutableMapOf<String, Any?>()
          val it = obj.keys()
          while (it.hasNext()) {
            val k = it.next()
            map[k] = if (obj.isNull(k)) null else obj.get(k)
          }
          sendEvent(eventName, map)
        } catch (e: Exception) {
          // Bad JSON — skip and keep polling.
        }
      }
    }
    pollJobs[subId] = job
  }

  private fun parseJsonOrThrow(s: String?, op: String): Any {
    if (s == null) throw RpcError(op)
    val trimmed = s.trim()
    return when {
      trimmed.startsWith("{") -> {
        val obj = JSONObject(trimmed)
        if (obj.has("error")) throw RpcError("$op: ${obj.getString("error")}")
        jsonObjectToMap(obj)
      }
      trimmed.startsWith("[") -> {
        val arr = org.json.JSONArray(trimmed)
        jsonArrayToList(arr)
      }
      else -> trimmed
    }
  }

  private fun jsonObjectToMap(obj: JSONObject): Map<String, Any?> {
    val map = mutableMapOf<String, Any?>()
    val it = obj.keys()
    while (it.hasNext()) {
      val k = it.next()
      map[k] = unwrap(obj.opt(k))
    }
    return map
  }

  private fun jsonArrayToList(arr: org.json.JSONArray): List<Any?> {
    val out = ArrayList<Any?>(arr.length())
    for (i in 0 until arr.length()) out.add(unwrap(arr.opt(i)))
    return out
  }

  private fun unwrap(v: Any?): Any? = when (v) {
    JSONObject.NULL, null -> null
    is JSONObject -> jsonObjectToMap(v)
    is org.json.JSONArray -> jsonArrayToList(v)
    else -> v
  }

  private fun jsonToMap(s: String): Map<String, Any?> {
    val obj = JSONObject(s)
    if (obj.has("error")) throw RpcError(obj.getString("error"))
    val map = mutableMapOf<String, Any?>()
    val it = obj.keys()
    while (it.hasNext()) {
      val k = it.next()
      map[k] = if (obj.isNull(k)) null else obj.get(k)
    }
    return map
  }
}

private class RpcError(msg: String) : RuntimeException("rockbox-rpc: $msg")
