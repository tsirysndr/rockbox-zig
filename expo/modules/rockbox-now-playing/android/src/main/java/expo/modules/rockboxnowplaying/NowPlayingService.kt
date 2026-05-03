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
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext
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

    const val NOTIFICATION_CHANNEL_ID = "rockbox.nowplaying"
    const val NOTIFICATION_ID = 4711

    const val ACTION_UPDATE = "expo.modules.rockboxnowplaying.UPDATE"
    const val ACTION_SET_PLAYBACK = "expo.modules.rockboxnowplaying.SET_PLAYBACK"
    const val ACTION_CLEAR = "expo.modules.rockboxnowplaying.CLEAR"
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

  override fun onBind(intent: Intent?): IBinder? = null

  override fun onCreate() {
    super.onCreate()
    ensureNotificationChannel()

    mediaSession = MediaSessionCompat(this, "RockboxNowPlaying").apply {
      setCallback(object : MediaSessionCompat.Callback() {
        override fun onPlay() = emit("play")
        override fun onPause() = emit("pause")
        override fun onSkipToNext() = emit("next")
        override fun onSkipToPrevious() = emit("prev")
        override fun onStop() = emit("stop")
        override fun onSeekTo(pos: Long) = emit("seek", pos)
      })
      isActive = true
    }
  }

  override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
    val action = intent?.action
    when (action) {
      ACTION_UPDATE -> handleUpdate(intent)
      ACTION_SET_PLAYBACK -> {
        applyPlaybackFromIntent(intent)
        refreshNotification()
      }
      ACTION_CLEAR -> {
        stopSelf()
        return START_NOT_STICKY
      }
      ACTION_BUTTON_PLAY -> emit("play")
      ACTION_BUTTON_PAUSE -> emit("pause")
      ACTION_BUTTON_NEXT -> emit("next")
      ACTION_BUTTON_PREV -> emit("prev")
      ACTION_BUTTON_STOP -> {
        emit("stop")
        stopSelf()
        return START_NOT_STICKY
      }
      else -> {
        // Started without an action (e.g. after process death) — make sure
        // we still get into the foreground so Android doesn't kill us.
        if (currentTrackId != null) refreshNotification()
      }
    }
    return START_STICKY
  }

  override fun onDestroy() {
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

  private fun handleUpdate(intent: Intent) {
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
      Log.e(TAG, "startForeground failed", e)
    }
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

  private fun emit(action: String, positionMs: Long? = null) {
    RockboxNowPlayingModule.dispatchAction(action, positionMs)
  }
}
