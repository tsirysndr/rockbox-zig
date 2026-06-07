// Singleton browser-side HLS audio controller.
//
// When the active rockboxd output is switched to PCM HLS (the CMAF sink),
// this controller is told to attach to the stream URL and the user hears the
// broadcast directly in the browser. Pause/resume are wired to the same
// pause/play button the GraphQL mutations use; the controller answers the
// "what I hear" half locally while the GraphQL call drives the "what the
// upstream player is doing" half. Volume + mute are purely local.
//
// hls.js handles every modern browser (Chromium, Firefox, Safari, Edge,
// mobile). Safari can also play HLS via the native <audio> element, but
// keeping hls.js as the single code path keeps the behavior uniform —
// including the segment-fetch error events we rely on for "stream gone"
// detection.

import Hls from "hls.js";

export class HlsAudioController {
  private audio: HTMLAudioElement;
  private hls: Hls | null = null;
  private currentUrl: string | null = null;
  private volume = 1.0;
  private muted = false;
  private listeners = new Set<(state: HlsAudioState) => void>();
  private retryTimer: ReturnType<typeof setTimeout> | null = null;

  constructor() {
    this.audio = new Audio();
    this.audio.preload = "none";
    this.audio.crossOrigin = "anonymous";
    // Attach to the DOM so the browser treats it as a real media element
    // and doesn't pause it under memory pressure or when nothing visible is
    // pointing at it. The element itself stays invisible — only its audio
    // output matters. `data-rockbox-hls` is purely for debugging in devtools.
    this.audio.setAttribute("data-rockbox-hls", "1");
    this.audio.style.display = "none";
    if (typeof document !== "undefined" && document.body) {
      document.body.appendChild(this.audio);
    }
    this.audio.addEventListener("play", () => this.emit());
    this.audio.addEventListener("pause", () => this.emit());
    this.audio.addEventListener("ended", () => this.emit());
    this.audio.addEventListener("volumechange", () => {
      this.volume = this.audio.volume;
      this.muted = this.audio.muted;
      this.emit();
    });
  }

  /** Connect to the stream URL. Idempotent for the same URL. */
  attach(url: string) {
    if (this.currentUrl === url && this.hls !== null) return;
    this.detach();
    this.currentUrl = url;

    if (Hls.isSupported()) {
      this.hls = new Hls({
        // Try to stay close to the live edge.
        liveSyncDuration: 4,
        liveMaxLatencyDuration: 8,
        // Don't burn CPU on backfill — we want to hear "now".
        backBufferLength: 30,
      });
      this.hls.loadSource(url);
      this.hls.attachMedia(this.audio);
      this.hls.on(Hls.Events.MANIFEST_PARSED, () => {
        this.tryAutoplay();
      });
      this.hls.on(Hls.Events.ERROR, (_, data) => {
        if (!data.fatal) return;
        console.warn("[hls-audio] fatal:", data);
        // Recovery strategy depends on the error class:
        //   - NETWORK_ERROR (incl. levelEmptyError when rockboxd has no
        //     segments yet) → tear everything down and re-attach after a
        //     short backoff. The CMAF encoder now bootstraps with a
        //     silence-segment window so this branch should fire only on
        //     genuine network blips, not on a freshly-started daemon.
        //   - MEDIA_ERROR → hls.js can usually recover in place.
        //   - everything else → re-attach.
        if (data.type === Hls.ErrorTypes.MEDIA_ERROR) {
          try {
            this.hls?.recoverMediaError();
            return;
          } catch (e) {
            console.warn("[hls-audio] recoverMediaError failed:", e);
          }
        }
        this.scheduleReattach();
      });
    } else if (this.audio.canPlayType("application/vnd.apple.mpegurl")) {
      // Native HLS (older Safari w/o MSE support for fMP4).
      this.audio.src = url;
      this.tryAutoplay();
    } else {
      console.error(
        "[hls-audio] browser supports neither MSE nor native HLS",
      );
    }
    this.emit();
  }

  /** Disconnect and tear down the hls.js instance. */
  detach() {
    if (this.retryTimer !== null) {
      clearTimeout(this.retryTimer);
      this.retryTimer = null;
    }
    if (this.hls) {
      this.hls.destroy();
      this.hls = null;
    }
    if (!this.audio.paused) this.audio.pause();
    this.audio.removeAttribute("src");
    this.currentUrl = null;
    this.emit();
  }

  /** Schedule a re-attach to the *current* URL after `delayMs`. Called from
   *  the hls.js fatal-error handler to recover from transient failures —
   *  e.g. the rockboxd daemon restarted, or a brief network blip. */
  private scheduleReattach(delayMs = 1500) {
    if (this.retryTimer !== null || !this.currentUrl) return;
    const url = this.currentUrl;
    this.retryTimer = setTimeout(() => {
      this.retryTimer = null;
      console.info("[hls-audio] reattaching after fatal error:", url);
      this.detach();
      this.attach(url);
    }, delayMs);
  }

  /** Local pause — does NOT touch the upstream player. */
  pause() {
    if (!this.audio.paused) this.audio.pause();
  }

  /** Local resume — does NOT touch the upstream player. */
  resume() {
    // Resume is invoked from a real user click (pause/play button), so the
    // browser's autoplay policy doesn't apply.  Unmute if we'd previously
    // fallen back to muted autoplay.
    if (this.audio.muted) this.audio.muted = false;
    // Jump to the live edge before resuming. While we were paused the CMAF
    // encoder kept producing silence segments at wall-clock cadence and the
    // <audio> element happily buffered them; without a seek the user hears
    // that buffered silence before the broadcaster's real audio reaches the
    // player. Seeking past it makes resume feel instantaneous.
    this.seekToLiveEdge();
    this.audio
      .play()
      .catch((e) => console.warn("[hls-audio] resume blocked:", e));
  }

  /** Snap `audio.currentTime` forward to hls.js's recommended live position.
   *  No-op if we're on a native-HLS code path (Safari) or not live. */
  private seekToLiveEdge() {
    if (!this.hls) return;
    const pos = this.hls.liveSyncPosition;
    if (typeof pos !== "number" || isNaN(pos) || pos <= 0) return;
    // Don't seek backwards — if the player is already ahead of the live
    // sync point we'd insert an unnecessary stall.
    if (this.audio.currentTime >= pos) return;
    try {
      this.audio.currentTime = pos;
    } catch (e) {
      // Setting currentTime can throw if the seek target isn't yet in the
      // buffered range; hls.js will then fetch the appropriate segment.
      console.debug("[hls-audio] seek to live edge:", e);
    }
  }

  /** Attempt autoplay with a muted fallback.
   *
   * All modern browsers (Chrome, Safari, Firefox, Edge) block
   * `audio.play()` if the user hasn't interacted with the page yet — that
   * raises `NotAllowedError`. Every browser DOES allow *muted* autoplay
   * unconditionally though, so we use it as the universal fallback: start
   * silent, and the first time the user adjusts the volume slider or
   * clicks the speaker icon (both genuine user gestures), we unmute.
   *
   * That matches the muted-autoplay pattern YouTube / Twitter / etc. use
   * for embedded media. */
  private tryAutoplay() {
    this.audio
      .play()
      .catch(() => {
        this.audio.muted = true;
        this.audio
          .play()
          .catch((e) => console.warn("[hls-audio] muted autoplay blocked:", e));
      });
  }

  setVolume(v: number) {
    const clamped = Math.max(0, Math.min(1, v));
    this.audio.volume = clamped;
    if (clamped > 0 && this.audio.muted) this.audio.muted = false;
    // Adjusting the volume IS a user gesture — if autoplay was previously
    // blocked we may have a paused stream waiting; try to resume now.
    if (this.audio.paused && this.currentUrl) {
      this.seekToLiveEdge();
      this.audio.play().catch(() => {});
    }
  }

  setMuted(m: boolean) {
    this.audio.muted = m;
    if (!m && this.audio.paused && this.currentUrl) {
      // Unmute is a user gesture — kick playback in case autoplay was blocked.
      this.seekToLiveEdge();
      this.audio.play().catch(() => {});
    }
  }

  toggleMute() {
    this.audio.muted = !this.audio.muted;
    if (!this.audio.muted && this.audio.paused && this.currentUrl) {
      this.seekToLiveEdge();
      this.audio.play().catch(() => {});
    }
  }

  state(): HlsAudioState {
    return {
      attached: this.currentUrl !== null,
      url: this.currentUrl,
      playing: !this.audio.paused && !this.audio.ended,
      volume: this.volume,
      muted: this.muted,
    };
  }

  subscribe(fn: (state: HlsAudioState) => void): () => void {
    this.listeners.add(fn);
    fn(this.state());
    return () => this.listeners.delete(fn);
  }

  private emit() {
    const s = this.state();
    this.listeners.forEach((fn) => fn(s));
  }
}

export interface HlsAudioState {
  attached: boolean;
  url: string | null;
  playing: boolean;
  volume: number;
  muted: boolean;
}

export const hlsAudio = new HlsAudioController();

// React hook over the controller — components re-render on state changes.
import { useEffect, useState } from "react";

export function useHlsAudio(): HlsAudioState {
  const [state, setState] = useState<HlsAudioState>(hlsAudio.state());
  useEffect(() => hlsAudio.subscribe(setState), []);
  return state;
}
