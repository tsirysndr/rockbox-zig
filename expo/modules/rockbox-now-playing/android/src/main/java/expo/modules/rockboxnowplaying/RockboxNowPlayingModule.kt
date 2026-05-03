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

  private fun startServiceCompat(ctx: Context, intent: Intent) {
    try {
      if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
        ctx.startForegroundService(intent)
      } else {
        ctx.startService(intent)
      }
    } catch (e: Throwable) {
      Log.e(TAG, "startService failed", e)
    }
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
