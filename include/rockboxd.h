#ifndef ROCKBOXD_H
#define ROCKBOXD_H

#ifdef __cplusplus
extern "C" {
#endif

/* ── Daemon lifecycle ─────────────────────────────────────────────────────────
 *
 * rb_daemon_start() boots the full Rockbox firmware + gRPC/GraphQL/HTTP/MPD
 * servers + headless cpal audio sink in-process on a dedicated thread.
 * It blocks until the gRPC server binds (up to 30 s) then returns.
 *
 * Returns: gRPC port (positive) on success, or a negative error code:
 *   -22   invalid input (null device_name)
 *   -110  timeout — firmware did not bind within 30 s
 *   -114  already starting / running
 *
 * music_dir_ptr  Path to the music library. Pass NULL to fall back to
 *                $HOME/Music.
 * device_name_ptr  Name shown in mDNS advertisements. Must be non-null.
 * -------------------------------------------------------------------------- */
int rb_daemon_start(const char *music_dir_ptr, const char *device_name_ptr);

/* Best-effort stop signal. Returns 0 if it was running. */
int rb_daemon_stop(void);

/* Returns the gRPC port of the running daemon, or 0 if not running. */
int rb_daemon_port(void);

/* Returns the daemon state: 0 = stopped, 1 = starting, 2 = running. */
int rb_daemon_state(void);

/* Trigger a full background rescan of the music library. Returns 0 immediately.
 * Returns -1 if the daemon is not running. */
int rb_rescan_library(void);

/* ── Memory ───────────────────────────────────────────────────────────────────
 *
 * All rb_*_json() functions return heap-allocated NUL-terminated C strings.
 * The caller MUST free them via rb_free_string(); do NOT use free() directly.
 * Passing NULL is safe. */
void rb_free_string(char *ptr);

/* ── Configuration ────────────────────────────────────────────────────────────
 *
 * Call before any other rb_* function to override the default localhost URLs.
 * Defaults: gRPC = http://127.0.0.1:6061, HTTP = http://127.0.0.1:6063 */
int rb_set_server_url(const char *url_ptr);
int rb_set_http_url(const char *url_ptr);

/* Health check — returns 0 if the gRPC server responds. */
int rb_ping(void);

/* ── Playback control ─────────────────────────────────────────────────────────
 * All return 0 on success, -1 on RPC failure. */
int rb_play(void);
int rb_pause(void);
int rb_play_pause(void);
int rb_next(void);
int rb_prev(void);
int rb_seek(int position_ms);
int rb_resume_track(void);
int rb_playlist_resume(void);
int rb_play_all_tracks(void);
int rb_shuffle_playlist(void);
int rb_play_track(const char *path_ptr);
int rb_play_album(const char *album_id_ptr, int shuffle);
int rb_play_artist_tracks(const char *artist_id_ptr, int shuffle);
int rb_play_directory(const char *path_ptr, int shuffle, int position);
int rb_play_saved_playlist(const char *id_ptr);
int rb_play_smart_playlist(const char *id_ptr);

/* ── Status (returns JSON, caller must rb_free_string) ────────────────────── */
char *rb_status_json(void);
char *rb_current_track_json(void);
char *rb_get_playlist_current_json(void);

/* ── Queue ─────────────────────────────────────────────────────────────────── */
int rb_jump_to_queue_position(int pos);
int rb_insert_tracks(const char *paths_json_ptr, int position, int shuffle);
int rb_insert_track_next(const char *path_ptr);
int rb_insert_track_last(const char *path_ptr);
int rb_insert_directory(const char *path_ptr, int position);
int rb_remove_from_queue(int position);

/* ── Library (JSON; caller must rb_free_string) ───────────────────────────── */
char *rb_get_tracks_json(void);
char *rb_get_artists_json(void);
char *rb_get_albums_json(void);
char *rb_get_liked_tracks_json(void);
char *rb_get_liked_albums_json(void);
char *rb_get_artist_json(const char *id_ptr);
char *rb_get_album_json(const char *id_ptr);
char *rb_search_json(const char *term_ptr);

/* ── Like / unlike ─────────────────────────────────────────────────────────── */
int rb_like_track(const char *track_id_ptr);
int rb_unlike_track(const char *track_id_ptr);

/* ── Genres (JSON; caller must rb_free_string) ────────────────────────────── */
char *rb_get_genres_json(void);
char *rb_get_genre_json(const char *id_ptr);
char *rb_get_genre_tracks_json(const char *id_ptr);
char *rb_get_genre_albums_json(const char *id_ptr);
char *rb_get_genre_artists_json(const char *id_ptr);

/* ── Sound / settings ─────────────────────────────────────────────────────── */
int rb_adjust_volume(int steps);
char *rb_sound_current_json(int setting);
int rb_save_shuffle(int enabled);
int rb_save_repeat(int repeat_mode);
char *rb_get_global_settings_json(void);
char *rb_get_global_status_json(void);

/* ── Browse ─────────────────────────────────────────────────────────────────
 * path_ptr may be NULL to fetch the music root. */
char *rb_tree_get_entries_json(const char *path_ptr);

/* ── Saved / smart playlists (JSON; caller must rb_free_string) ───────────── */
char *rb_get_saved_playlists_json(void);
int rb_create_saved_playlist(const char *name_ptr, const char *description_ptr,
                              const char *track_ids_json_ptr);
int rb_update_saved_playlist(const char *id_ptr, const char *name_ptr,
                              const char *description_ptr);
int rb_delete_saved_playlist(const char *id_ptr);
int rb_add_track_to_playlist(const char *playlist_id_ptr, const char *track_id_ptr);
int rb_remove_track_from_playlist(const char *playlist_id_ptr, const char *track_id_ptr);
char *rb_get_saved_playlist_tracks_json(const char *id_ptr);
char *rb_get_smart_playlists_json(void);
char *rb_get_smart_playlist_tracks_json(const char *id_ptr);

/* ── Devices (Chromecast / AirPlay / Snapcast via HTTP REST) ─────────────── */
char *rb_get_devices_json(void);
int rb_connect_device(const char *id_ptr);
int rb_disconnect_device(const char *id_ptr);

/* ── Bluetooth ──────────────────────────────────────────────────────────────
 * Returns 1 if the Bluetooth service is available, 0 otherwise. */
int rb_scan_bluetooth(void);
int rb_bluetooth_available(void);
char *rb_get_bluetooth_devices_json(void);
int rb_connect_bluetooth(const char *addr_ptr);
int rb_disconnect_bluetooth(const char *addr_ptr);

/* ── Streaming subscriptions ─────────────────────────────────────────────────
 *
 * Each rb_subscribe_*() call spawns a background task that drains the
 * corresponding server-streaming RPC into an internal queue, and returns an
 * opaque subscription id (positive integer).
 *
 * Call rb_poll_event(id, timeout_ms) in a loop to receive events as JSON
 * C strings (free each with rb_free_string). Returns NULL on timeout.
 *
 * Call rb_unsubscribe(id) to cancel and free the subscription. */
int rb_subscribe_status(void);
int rb_subscribe_current_track(void);
int rb_subscribe_playlist(void);
int rb_subscribe_library(void);
int rb_subscribe_discovery(const char *service_name_ptr);
char *rb_poll_event(int sub_id, int timeout_ms);
int rb_unsubscribe(int sub_id);

/* Well-known mDNS service names (heap strings; free with rb_free_string). */
char *rb_rockbox_service_name(void);

#ifdef __cplusplus
}
#endif

#endif /* ROCKBOXD_H */
