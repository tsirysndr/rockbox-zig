package expo.modules.rockboxnowplaying

import android.Manifest
import android.app.Activity
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.os.Build
import android.util.Log
import androidx.core.app.ActivityCompat
import androidx.core.content.ContextCompat
import expo.modules.kotlin.modules.Module
import expo.modules.kotlin.modules.ModuleDefinition

/**
 * JS-facing entry point for the Now Playing media session. Pushes updates
 * to [NowPlayingService] via Intents, and forwards transport-button taps
 * back to JS as `rockbox.nowplaying.action` events.
 *
 * The service is implicitly spawned by the first `update` call. Subsequent
 * `update` / `setPlayback` calls keep it alive; `clear` shuts it down.
 */
class RockboxNowPlayingModule : Module() {
  companion object {
    private const val TAG = "RockboxNowPlaying"

    @Volatile private var current: RockboxNowPlayingModule? = null
    @Volatile private var permissionRequested: Boolean = false

    /** Called from [NowPlayingService] when a transport button fires. */
    fun dispatchAction(action: String, positionMs: Long?) {
      val mod = current ?: return
      val payload = mutableMapOf<String, Any>("action" to action)
      if (positionMs != null) payload["positionMs"] = positionMs
      try {
        mod.sendEvent("rockbox.nowplaying.action", payload)
      } catch (_: Throwable) {
        // Module torn down between dispatch and emit — drop silently.
      }
    }
  }

  override fun definition() = ModuleDefinition {
    Name("RockboxNowPlaying")

    Events("rockbox.nowplaying.action")

    OnCreate {
      current = this@RockboxNowPlayingModule
    }
    OnDestroy {
      if (current === this@RockboxNowPlayingModule) current = null
    }

    Function("update") { metadata: Map<String, Any?>, playback: Map<String, Any?> ->
      val ctx = appContext.reactContext?.applicationContext
      if (ctx != null) {
        ensureNotificationPermission()
        val intent = Intent(ctx, NowPlayingService::class.java).apply {
          action = NowPlayingService.ACTION_UPDATE
          putExtra(NowPlayingService.EXTRA_TRACK_ID, (metadata["trackId"] as? String).orEmpty())
          putExtra(NowPlayingService.EXTRA_TITLE, (metadata["title"] as? String).orEmpty())
          putExtra(NowPlayingService.EXTRA_ARTIST, (metadata["artist"] as? String).orEmpty())
          putExtra(NowPlayingService.EXTRA_ALBUM, (metadata["album"] as? String).orEmpty())
          putExtra(NowPlayingService.EXTRA_ARTWORK_URL, metadata["artworkUrl"] as? String)
          putExtra(NowPlayingService.EXTRA_COVER_BASE_URL, metadata["coverBaseUrl"] as? String)
          putExtra(NowPlayingService.EXTRA_DURATION_MS, (metadata["durationMs"] as? Number)?.toLong() ?: 0L)
          putExtra(NowPlayingService.EXTRA_IS_PLAYING, (playback["isPlaying"] as? Boolean) ?: false)
          putExtra(NowPlayingService.EXTRA_POSITION_MS, (playback["positionMs"] as? Number)?.toLong() ?: 0L)
          putExtra(NowPlayingService.EXTRA_SPEED, (playback["speed"] as? Number)?.toFloat() ?: 1f)
        }
        startServiceCompat(ctx, intent)
      }
      Unit
    }

    Function("setPlayback") { playback: Map<String, Any?> ->
      val ctx = appContext.reactContext?.applicationContext
      if (ctx != null) {
        val intent = Intent(ctx, NowPlayingService::class.java).apply {
          action = NowPlayingService.ACTION_SET_PLAYBACK
          putExtra(NowPlayingService.EXTRA_IS_PLAYING, (playback["isPlaying"] as? Boolean) ?: false)
          putExtra(NowPlayingService.EXTRA_POSITION_MS, (playback["positionMs"] as? Number)?.toLong() ?: 0L)
          putExtra(NowPlayingService.EXTRA_SPEED, (playback["speed"] as? Number)?.toFloat() ?: 1f)
        }
        startServiceCompat(ctx, intent)
      }
      Unit
    }

    Function("clear") {
      val ctx = appContext.reactContext?.applicationContext
      if (ctx != null) {
        val intent = Intent(ctx, NowPlayingService::class.java).apply {
          action = NowPlayingService.ACTION_CLEAR
        }
        try {
          ctx.startService(intent)
        } catch (_: Throwable) {
          // Service not running — nothing to clear.
        }
      }
      Unit
    }

    /**
     * Bring NowPlayingService into foreground state at app launch so the
     * host process (which also owns the embedded rockbox daemon) survives
     * Android's background-app reaper. The service stays alive until ACTION_CLEAR
     * is sent — the placeholder notification is replaced as soon as a real
     * track update arrives.
     *
     * No-op on iOS / web.
     */
    Function("start") {
      val ctx = appContext.reactContext?.applicationContext
      if (ctx != null) {
        ensureNotificationPermission()
        val intent = Intent(ctx, NowPlayingService::class.java).apply {
          action = NowPlayingService.ACTION_BOOT
        }
        startServiceCompat(ctx, intent)
      }
      Unit
    }

    Function("setCoverBaseUrl") { url: String ->
      val ctx = appContext.reactContext?.applicationContext
      if (ctx != null) {
        val intent = Intent(ctx, NowPlayingService::class.java).apply {
          action = NowPlayingService.ACTION_SET_COVER_BASE
          putExtra(NowPlayingService.EXTRA_COVER_BASE_URL, url)
        }
        startServiceCompat(ctx, intent)
      }
      Unit
    }
  }

  /**
   * Start NowPlayingService in the right mode for the app's current state.
   *
   * Android 14+ (API 34) blocks `startForegroundService` when the app's
   * uidState is SVC / cached / RECEIVER, even with `mediaPlayback` type and
   * the FOREGROUND_SERVICE_MEDIA_PLAYBACK permission. The service's later
   * `startForeground()` call throws `ForegroundServiceStartNotAllowedException`,
   * which the WatchDog escalates to a fatal RemoteServiceException — the
   * try/catch in onStartCommand can't save us because the system still
   * tracks the unfulfilled FGS-promotion deadline.
   *
   * Strategy: use `startForegroundService` only when we're actually allowed
   * (process is in TOP / FGS / FGS_LOCATION / BFGS importance). Otherwise
   * fall back to plain `startService` — the service still receives the
   * intent and updates its in-process state, just without the foreground
   * promotion that would trigger the violation. The notification will pop
   * up the next time the user opens the app and the FGS start succeeds.
   */
  private fun startServiceCompat(ctx: Context, intent: Intent) {
    val canStartFgs = canStartForegroundService(ctx)
    try {
      if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O && canStartFgs) {
        ctx.startForegroundService(intent)
      } else {
        ctx.startService(intent)
      }
    } catch (e: IllegalStateException) {
      // Race: importance changed between our check and the call. Fall back.
      Log.w(TAG, "startForegroundService denied — falling back to startService: ${e.message}")
      try {
        ctx.startService(intent)
      } catch (e2: Throwable) {
        Log.e(TAG, "startService also failed", e2)
      }
    } catch (e: Throwable) {
      Log.e(TAG, "startService failed", e)
    }
  }

  /** True if the app's process state lets us start a foreground service.
   *  Android 14+ requires importance ≥ FOREGROUND_SERVICE; below that we
   *  must use plain startService. */
  private fun canStartForegroundService(ctx: Context): Boolean {
    if (Build.VERSION.SDK_INT < Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
      // Pre-API-34 the BG-start restriction was much weaker — call FGS
      // unconditionally and rely on Service.startForeground's try/catch.
      return true
    }
    val info = android.app.ActivityManager.RunningAppProcessInfo()
    android.app.ActivityManager.getMyMemoryState(info)
    // VISIBLE/FOREGROUND/PERCEPTIBLE/FOREGROUND_SERVICE all let us promote.
    return info.importance <=
      android.app.ActivityManager.RunningAppProcessInfo.IMPORTANCE_FOREGROUND_SERVICE
  }

  /** On Android 13+, the OS silently drops notifications until the user grants
   *  POST_NOTIFICATIONS at runtime. Declaring it in the manifest is necessary
   *  but not sufficient. We request it once per process from the foreground
   *  Activity; the user sees the system prompt next time they open the app. */
  private fun ensureNotificationPermission() {
    if (Build.VERSION.SDK_INT < Build.VERSION_CODES.TIRAMISU) return
    if (permissionRequested) return
    val ctx = appContext.reactContext?.applicationContext ?: return
    val granted = ContextCompat.checkSelfPermission(ctx, Manifest.permission.POST_NOTIFICATIONS) ==
      PackageManager.PERMISSION_GRANTED
    if (granted) {
      permissionRequested = true
      return
    }
    val activity: Activity? = appContext.currentActivity
    if (activity != null) {
      try {
        ActivityCompat.requestPermissions(
          activity,
          arrayOf(Manifest.permission.POST_NOTIFICATIONS),
          REQUEST_NOTIFICATION_PERMISSION,
        )
        permissionRequested = true
      } catch (e: Throwable) {
        Log.e(TAG, "requestPermissions failed", e)
      }
    }
  }
}

private const val REQUEST_NOTIFICATION_PERMISSION = 4712
