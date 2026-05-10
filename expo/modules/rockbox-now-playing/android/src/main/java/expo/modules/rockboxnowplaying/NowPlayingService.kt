package expo.modules.rockboxnowplaying

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Context
import android.content.Intent
import android.content.pm.ServiceInfo
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.os.Build
import android.os.IBinder
import android.util.Log
import android.support.v4.media.MediaMetadataCompat
import android.support.v4.media.session.MediaSessionCompat
import android.support.v4.media.session.PlaybackStateCompat
import androidx.core.app.NotificationCompat
import androidx.media.app.NotificationCompat.MediaStyle
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext
import expo.modules.rockboxrpc.RockboxRpcModule
import org.json.JSONObject
import java.net.URL

/**
 * Foreground service that owns the [MediaSessionCompat] and the MediaStyle
 * notification. JS pushes updates via [RockboxNowPlayingModule]; transport
 * button taps are converted into Intents on this service which forward to
 * the module's event emitter.
 *
 * The service stays alive across screen-off so lock-screen controls keep
 * working, and shuts itself down when [ACTION_CLEAR] is received.
 */
class NowPlayingService : Service() {
  companion object {
    private const val TAG = "RockboxNowPlaying"

    /**
     * The JNI bridge symbols are mangled per-class (`Java_<pkg>_<class>_<method>`),
     * so we can't redeclare `external fun rb_*` here — the dynamic linker
     * would fail to resolve them. Instead we delegate to the static
     * declarations already present on [RockboxRpcModule], which point at the
     * real symbols emitted by `crates/expo/src/jni_bridge.rs`.
     */

    const val NOTIFICATION_CHANNEL_ID = "rockbox.nowplaying"
    const val NOTIFICATION_ID = 4711

    const val ACTION_UPDATE = "expo.modules.rockboxnowplaying.UPDATE"
    const val ACTION_SET_PLAYBACK = "expo.modules.rockboxnowplaying.SET_PLAYBACK"
    const val ACTION_SET_COVER_BASE = "expo.modules.rockboxnowplaying.SET_COVER_BASE"
    const val ACTION_CLEAR = "expo.modules.rockboxnowplaying.CLEAR"
    /** No-op action used to bring the service into foreground state at app
     *  launch — keeps the host process alive (and the daemon with it) after
     *  the user backgrounds the app. The placeholder notification is replaced
     *  the moment ACTION_UPDATE arrives with real track info. */
    const val ACTION_BOOT = "expo.modules.rockboxnowplaying.BOOT"
    const val ACTION_BUTTON_PLAY = "expo.modules.rockboxnowplaying.PLAY"
    const val ACTION_BUTTON_PAUSE = "expo.modules.rockboxnowplaying.PAUSE"
    const val ACTION_BUTTON_NEXT = "expo.modules.rockboxnowplaying.NEXT"
    const val ACTION_BUTTON_PREV = "expo.modules.rockboxnowplaying.PREV"
    const val ACTION_BUTTON_STOP = "expo.modules.rockboxnowplaying.STOP"

    const val EXTRA_TRACK_ID = "trackId"
    const val EXTRA_TITLE = "title"
    const val EXTRA_ARTIST = "artist"
    const val EXTRA_ALBUM = "album"
    const val EXTRA_ARTWORK_URL = "artworkUrl"
    const val EXTRA_COVER_BASE_URL = "coverBaseUrl"
    const val EXTRA_DURATION_MS = "durationMs"
    const val EXTRA_POSITION_MS = "positionMs"
    const val EXTRA_IS_PLAYING = "isPlaying"
    const val EXTRA_SPEED = "speed"
  }

  private lateinit var mediaSession: MediaSessionCompat
  private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
  private val artLoadMutex = Mutex()

  /** Cache last-applied state so partial updates (setPlayback) can re-emit
   *  the metadata without the JS side resending it. */
  private var currentTrackId: String? = null
  private var currentTitle: String = ""
  private var currentArtist: String = ""
  private var currentAlbum: String = ""
  private var currentArtworkUrl: String? = null
  private var currentArtwork: Bitmap? = null
  private var currentDurationMs: Long = 0
  private var currentPositionMs: Long = 0
  private var currentIsPlaying: Boolean = false
  private var currentSpeed: Float = 1f
  private var artLoadJob: Job? = null
  private var coverBaseUrl: String? = null

  /** Native subscription state — independent of JS so track / play-pause
   *  changes still update the lock-screen card while JS is suspended. */
  private var statusSubId: Int = -1
  private var trackSubId: Int = -1
  private var statusJob: Job? = null
  private var trackJob: Job? = null

  override fun onBind(intent: Intent?): IBinder? = null

  override fun onTaskRemoved(rootIntent: Intent?) {
    // User swiped the app away — stop the foreground service so Android can
    // kill the process cleanly. Every reopen is then a fresh process with
    // clean gRPC state. Without this the service keeps the process alive,
    // the old Rust streaming tasks compete with new ones on reopen, and
    // the UI can't make gRPC calls until a second swipe kills the process.
    stopSelf()
  }

  override fun onCreate() {
    super.onCreate()
    ensureNotificationChannel()
    /* Daemon now boots from RockboxRpcModule.OnCreate (app start) instead
     * of here (first-playback). Keep bootEmbeddedDaemon() as a no-op-ish
     * fallback that just logs the daemon state — useful for debugging. */
    bootEmbeddedDaemon()

    mediaSession = MediaSessionCompat(this, "RockboxNowPlaying").apply {
      setCallback(object : MediaSessionCompat.Callback() {
        override fun onPlay() = handleAction("play")
        override fun onPause() = handleAction("pause")
        override fun onSkipToNext() = handleAction("next")
        override fun onSkipToPrevious() = handleAction("prev")
        override fun onStop() = handleAction("stop")
        override fun onSeekTo(pos: Long) = handleAction("seek", pos)
      })
      isActive = true
    }
  }

  override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
    // Android 12+: every startForegroundService() must be followed by
    // startForeground() within ~5s or the system raises
    // ForegroundServiceDidNotStartInTimeException and kills the process.
    // CLEAR / SET_COVER_BASE / NEXT / PREV / unknown-action paths used to
    // skip refreshNotification() and crash the app — emit a (placeholder)
    // notification first thing so the promise is always satisfied.
    refreshNotification()

    val action = intent?.action
    when (action) {
      ACTION_UPDATE -> {
        handleUpdate(intent)
        startNativeSubscriptions()
      }
      ACTION_SET_PLAYBACK -> {
        applyPlaybackFromIntent(intent)
        refreshNotification()
      }
      ACTION_SET_COVER_BASE -> {
        coverBaseUrl = intent.getStringExtra(EXTRA_COVER_BASE_URL)
      }
      ACTION_CLEAR -> {
        stopSelf()
        return START_NOT_STICKY
      }
      ACTION_BUTTON_PLAY -> handleAction("play")
      ACTION_BUTTON_PAUSE -> handleAction("pause")
      ACTION_BUTTON_NEXT -> handleAction("next")
      ACTION_BUTTON_PREV -> handleAction("prev")
      ACTION_BUTTON_STOP -> {
        handleAction("stop")
        stopSelf()
        return START_NOT_STICKY
      }
    }
    return START_NOT_STICKY
  }

  override fun onDestroy() {
    stopNativeSubscriptions()
    artLoadJob?.cancel()
    scope.cancel()
    mediaSession.isActive = false
    mediaSession.release()
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.N) {
      stopForeground(STOP_FOREGROUND_REMOVE)
    } else {
      @Suppress("DEPRECATION")
      stopForeground(true)
    }
    super.onDestroy()
  }

  /**
   * Boot the in-process rockbox firmware on the very first onCreate. The
   * native fn is a no-op (-38) when librockbox_expo.so was built without
   * the `embedded-daemon` cargo feature (e.g. on iOS or older builds), so
   * the rest of the JS app still works against a remote daemon.
   *
   * On success, the daemon binds gRPC on 127.0.0.1:6061 and the existing
   * RockboxRpcModule.rb_set_server_url call (made by JS at app start) is
   * automatically retargeted to localhost by the daemon module itself.
   */
  private fun bootEmbeddedDaemon() {
    val configDir = applicationContext.filesDir.absolutePath
    val musicDir = android.os.Environment
      .getExternalStoragePublicDirectory(android.os.Environment.DIRECTORY_MUSIC)
      .absolutePath
    val deviceName = android.os.Build.MODEL ?: "rockbox-android"

    // Off the main thread — native init blocks until the gRPC server binds
    // (up to 5s). Don't ANR the system_server on slow devices.
    //
    // Wrap in try/catch so a crash in native code (segfault → SIGABRT,
    // missing JNI symbol → UnsatisfiedLinkError) doesn't take down the
    // foreground service / whole app. The remote-only fallback path
    // (RockboxRpcModule's tonic client to LAN peers) keeps working.
    scope.launch {
      try {
        val rc = RockboxRpcModule.rb_daemon_start(configDir, musicDir, deviceName)
        when {
          rc > 0 -> Log.i(TAG, "embedded daemon started, gRPC :$rc")
          rc == -38 -> Log.i(TAG, "embedded daemon not built into this .so (remote-only)")
          rc == -114 -> Log.i(TAG, "embedded daemon already running")
          else -> Log.w(TAG, "embedded daemon start failed rc=$rc")
        }
      } catch (t: Throwable) {
        Log.e(TAG, "embedded daemon threw: ${t.javaClass.simpleName}: ${t.message}", t)
      }
    }
  }

  private fun handleUpdate(intent: Intent) {
    intent.getStringExtra(EXTRA_COVER_BASE_URL)?.let { coverBaseUrl = it }

    val newTrackId = intent.getStringExtra(EXTRA_TRACK_ID) ?: ""
    val newArtworkUrl = intent.getStringExtra(EXTRA_ARTWORK_URL)
    val artworkChanged = newTrackId != currentTrackId || newArtworkUrl != currentArtworkUrl

    currentTrackId = newTrackId
    currentTitle = intent.getStringExtra(EXTRA_TITLE).orEmpty()
    currentArtist = intent.getStringExtra(EXTRA_ARTIST).orEmpty()
    currentAlbum = intent.getStringExtra(EXTRA_ALBUM).orEmpty()
    currentDurationMs = intent.getLongExtra(EXTRA_DURATION_MS, 0)
    currentArtworkUrl = newArtworkUrl
    applyPlaybackFromIntent(intent)

    if (artworkChanged) {
      currentArtwork = null
      artLoadJob?.cancel()
      val url = newArtworkUrl
      val trackId = newTrackId
      if (!url.isNullOrEmpty()) {
        artLoadJob = scope.launch {
          val bitmap = loadArtwork(url)
          // If the user skipped tracks while we were downloading, drop the
          // stale bitmap on the floor.
          if (bitmap != null && trackId == currentTrackId) {
            currentArtwork = bitmap
            withContext(Dispatchers.Main) { refreshNotification() }
          }
        }
      }
    }

    refreshNotification()
  }

  private fun applyPlaybackFromIntent(intent: Intent) {
    if (intent.hasExtra(EXTRA_POSITION_MS)) {
      currentPositionMs = intent.getLongExtra(EXTRA_POSITION_MS, 0)
    }
    if (intent.hasExtra(EXTRA_IS_PLAYING)) {
      currentIsPlaying = intent.getBooleanExtra(EXTRA_IS_PLAYING, false)
    }
    if (intent.hasExtra(EXTRA_SPEED)) {
      currentSpeed = intent.getFloatExtra(EXTRA_SPEED, 1f)
    }
  }

  /**
   * Subscribe to the rockboxd current-track + status streams directly from
   * the service. JS does the same subscribe via [RockboxRpcModule], but its
   * delivery path passes through the JS thread which Android can suspend
   * once the screen is locked. Subscribing here keeps the lock-screen card
   * fresh independent of the JS event loop.
   */
  private fun startNativeSubscriptions() {
    if (trackSubId == -1) {
      val id = try { RockboxRpcModule.rb_subscribe_current_track() } catch (e: Throwable) {
        Log.e(TAG, "rb_subscribe_current_track failed", e); -1
      }
      if (id >= 0) {
        trackSubId = id
        trackJob = scope.launch { pollLoop(id, ::onTrackEvent) }
      }
    }
    if (statusSubId == -1) {
      val id = try { RockboxRpcModule.rb_subscribe_status() } catch (e: Throwable) {
        Log.e(TAG, "rb_subscribe_status failed", e); -1
      }
      if (id >= 0) {
        statusSubId = id
        statusJob = scope.launch { pollLoop(id, ::onStatusEvent) }
      }
    }
  }

  private fun stopNativeSubscriptions() {
    val sId = statusSubId
    val tId = trackSubId
    statusSubId = -1
    trackSubId = -1
    statusJob?.cancel()
    trackJob?.cancel()
    statusJob = null
    trackJob = null
    if (sId >= 0) try { RockboxRpcModule.rb_unsubscribe(sId) } catch (_: Throwable) {}
    if (tId >= 0) try { RockboxRpcModule.rb_unsubscribe(tId) } catch (_: Throwable) {}
  }

  private suspend fun pollLoop(subId: Int, handler: (JSONObject) -> Unit) {
    while (scope.isActive && (subId == statusSubId || subId == trackSubId)) {
      val payload = try {
        RockboxRpcModule.rb_poll_event(subId, 5000)
      } catch (e: Throwable) {
        Log.e(TAG, "rb_poll_event($subId) failed", e); null
      } ?: continue
      try {
        val json = JSONObject(payload)
        // Ignore the "rockbox.error" envelopes the Rust side emits when the
        // stream resets — the next reconnect will deliver fresh events.
        if (json.has("error")) continue
        withContext(Dispatchers.Main) { handler(json) }
      } catch (e: Throwable) {
        Log.e(TAG, "poll handler failed", e)
      }
    }
  }

  private fun onTrackEvent(json: JSONObject) {
    val newTrackId = json.optString("id", "")
    val newAlbumArt = json.optString("album_art").takeIf { it.isNotEmpty() }
    val newArtworkUrl = newAlbumArt?.let { resolveArtworkUrl(it) }
    val artworkChanged = newTrackId != currentTrackId || newArtworkUrl != currentArtworkUrl

    currentTrackId = newTrackId
    currentTitle = json.optString("title", "")
    currentArtist = json.optString("artist", "")
    currentAlbum = json.optString("album", "")
    currentDurationMs = json.optLong("duration_ms", 0L)
    currentPositionMs = json.optLong("elapsed_ms", currentPositionMs)
    currentArtworkUrl = newArtworkUrl

    if (artworkChanged) {
      currentArtwork = null
      artLoadJob?.cancel()
      val url = newArtworkUrl
      val trackId = newTrackId
      if (!url.isNullOrEmpty()) {
        artLoadJob = scope.launch {
          val bitmap = loadArtwork(url)
          if (bitmap != null && trackId == currentTrackId) {
            currentArtwork = bitmap
            withContext(Dispatchers.Main) { refreshNotification() }
          }
        }
      }
    }
    refreshNotification()
  }

  private fun onStatusEvent(json: JSONObject) {
    val s = json.optInt("status", -1)
    if (s == 1) currentIsPlaying = true
    else if (s == 2) currentIsPlaying = false
    refreshNotification()
  }

  private fun resolveArtworkUrl(albumArt: String): String? {
    if (albumArt.startsWith("http://") || albumArt.startsWith("https://")) return albumArt
    val base = coverBaseUrl ?: return null
    val joined = if (base.endsWith("/")) "$base$albumArt" else "$base/$albumArt"
    return joined
  }

  private fun refreshNotification() {
    val state = PlaybackStateCompat.Builder()
      .setActions(
        PlaybackStateCompat.ACTION_PLAY or
          PlaybackStateCompat.ACTION_PAUSE or
          PlaybackStateCompat.ACTION_PLAY_PAUSE or
          PlaybackStateCompat.ACTION_SKIP_TO_NEXT or
          PlaybackStateCompat.ACTION_SKIP_TO_PREVIOUS or
          PlaybackStateCompat.ACTION_STOP or
          PlaybackStateCompat.ACTION_SEEK_TO,
      )
      .setState(
        if (currentIsPlaying) PlaybackStateCompat.STATE_PLAYING else PlaybackStateCompat.STATE_PAUSED,
        currentPositionMs,
        currentSpeed,
      )
      .build()
    mediaSession.setPlaybackState(state)

    val metadataBuilder = MediaMetadataCompat.Builder()
      .putString(MediaMetadataCompat.METADATA_KEY_TITLE, currentTitle)
      .putString(MediaMetadataCompat.METADATA_KEY_ARTIST, currentArtist)
      .putString(MediaMetadataCompat.METADATA_KEY_ALBUM, currentAlbum)
      .putLong(MediaMetadataCompat.METADATA_KEY_DURATION, currentDurationMs)
    currentArtwork?.let {
      metadataBuilder.putBitmap(MediaMetadataCompat.METADATA_KEY_ALBUM_ART, it)
    }
    mediaSession.setMetadata(metadataBuilder.build())

    val style = MediaStyle()
      .setMediaSession(mediaSession.sessionToken)
      .setShowActionsInCompactView(0, 1, 2)

    val notification = NotificationCompat.Builder(this, NOTIFICATION_CHANNEL_ID)
      .setSmallIcon(android.R.drawable.ic_media_play)
      .setContentTitle(currentTitle.ifEmpty { "Rockbox" })
      .setContentText(listOfNotNull(
        currentArtist.takeIf { it.isNotEmpty() },
        currentAlbum.takeIf { it.isNotEmpty() },
      ).joinToString(" • "))
      .setLargeIcon(currentArtwork)
      .setStyle(style)
      .setVisibility(NotificationCompat.VISIBILITY_PUBLIC)
      .setOnlyAlertOnce(true)
      .setShowWhen(false)
      .addAction(action(ACTION_BUTTON_PREV, "Previous", android.R.drawable.ic_media_previous))
      .addAction(
        if (currentIsPlaying)
          action(ACTION_BUTTON_PAUSE, "Pause", android.R.drawable.ic_media_pause)
        else
          action(ACTION_BUTTON_PLAY, "Play", android.R.drawable.ic_media_play),
      )
      .addAction(action(ACTION_BUTTON_NEXT, "Next", android.R.drawable.ic_media_next))
      .setContentIntent(launchIntent())
      .setDeleteIntent(buildPendingIntent(ACTION_BUTTON_STOP))
      .build()

    // Android 14+: only promote to foreground when the app's process state
    // permits it. Calling startForeground from a backgrounded process throws
    // ForegroundServiceStartNotAllowedException, which the WatchDog escalates
    // to a fatal RemoteServiceException — the local try/catch can't save us.
    // When we can't promote, just leave the MediaSession active (system Media
    // Controls still pick it up); we'll re-attempt the promotion on the next
    // intent that arrives while the app is in foreground.
    if (canPromoteToForeground()) {
      try {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
          startForeground(
            NOTIFICATION_ID,
            notification,
            ServiceInfo.FOREGROUND_SERVICE_TYPE_MEDIA_PLAYBACK,
          )
        } else {
          startForeground(NOTIFICATION_ID, notification)
        }
      } catch (e: Throwable) {
        // Race: importance changed between check and call. Just log.
        Log.w(TAG, "startForeground denied (race): ${e.javaClass.simpleName}: ${e.message}")
      }
    } else {
      Log.i(TAG, "startForeground skipped — app not in foregroundable state")
    }
  }

  /** True if the app's process state allows entering the foreground.
   *  Mirrors the gate in RockboxNowPlayingModule.canStartForegroundService. */
  private fun canPromoteToForeground(): Boolean {
    if (Build.VERSION.SDK_INT < Build.VERSION_CODES.UPSIDE_DOWN_CAKE) return true
    val info = android.app.ActivityManager.RunningAppProcessInfo()
    android.app.ActivityManager.getMyMemoryState(info)
    return info.importance <=
      android.app.ActivityManager.RunningAppProcessInfo.IMPORTANCE_FOREGROUND_SERVICE
  }

  private fun action(intentAction: String, title: String, icon: Int): NotificationCompat.Action =
    NotificationCompat.Action.Builder(icon, title, buildPendingIntent(intentAction)).build()

  private fun buildPendingIntent(intentAction: String): PendingIntent {
    val intent = Intent(this, NowPlayingService::class.java).setAction(intentAction)
    val flags = PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
    return PendingIntent.getService(this, intentAction.hashCode(), intent, flags)
  }

  private fun launchIntent(): PendingIntent? {
    val launch = packageManager.getLaunchIntentForPackage(packageName) ?: return null
    val flags = PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
    return PendingIntent.getActivity(this, 0, launch, flags)
  }

  private fun ensureNotificationChannel() {
    if (Build.VERSION.SDK_INT < Build.VERSION_CODES.O) return
    val mgr = getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
    if (mgr.getNotificationChannel(NOTIFICATION_CHANNEL_ID) != null) return
    val channel = NotificationChannel(
      NOTIFICATION_CHANNEL_ID,
      "Now playing",
      NotificationManager.IMPORTANCE_LOW,
    ).apply {
      description = "Lock-screen and notification controls for the current track."
      setShowBadge(false)
      lockscreenVisibility = Notification.VISIBILITY_PUBLIC
      enableVibration(false)
    }
    mgr.createNotificationChannel(channel)
  }

  private suspend fun loadArtwork(url: String): Bitmap? = withContext(Dispatchers.IO) {
    try {
      artLoadMutex.withLock {
        URL(url).openStream().use { BitmapFactory.decodeStream(it) }
      }
    } catch (_: Throwable) {
      null
    }
  }

  /**
   * Run the transport command directly against the daemon via JNI, then
   * notify JS so any in-app UI (mini-player, full player) stays in sync.
   * Going through JS for the actual RPC call would queue up while the
   * screen is locked — see issue: pause worked, resume didn't.
   */
  private fun handleAction(action: String, positionMs: Long? = null) {
    scope.launch(Dispatchers.IO) {
      try {
        when (action) {
          "play" -> RockboxRpcModule.rb_play()
          "pause" -> RockboxRpcModule.rb_pause()
          "playPause" -> RockboxRpcModule.rb_play_pause()
          "next" -> RockboxRpcModule.rb_next()
          "prev" -> RockboxRpcModule.rb_prev()
          "stop" -> RockboxRpcModule.rb_pause()
          "seek" -> if (positionMs != null) RockboxRpcModule.rb_seek(positionMs.toInt())
        }
      } catch (e: Throwable) {
        Log.e(TAG, "rb_$action failed", e)
      }
    }
    // Optimistic local update so the notification flips state instantly,
    // even before the daemon's status stream confirms it.
    when (action) {
      "play" -> {
        currentIsPlaying = true
        refreshNotification()
      }
      "pause", "stop" -> {
        currentIsPlaying = false
        refreshNotification()
      }
    }
    // Forward to JS so the in-app UI updates promptly when foregrounded.
    RockboxNowPlayingModule.dispatchAction(action, positionMs)
  }
}
