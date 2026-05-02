(ns rockbox.settings
  "Read and write global rockboxd settings (volume, EQ, repeat mode, ...)."
  (:refer-clojure :exclude [get])
  (:require [rockbox.transport :as t]))

(def ^:private settings-fields
  "musicDir volume balance bass treble channelConfig stereoWidth
   eqEnabled eqPrecut
   eqBandSettings { cutoff q gain }
   replaygainSettings { noclip type preamp }
   compressorSettings { threshold makeupGain ratio knee releaseTime attackTime }
   crossfadeEnabled crossfadeFadeInDelay crossfadeFadeInDuration
   crossfadeFadeOutDelay crossfadeFadeOutDuration crossfadeFadeOutMixmode
   crossfeedEnabled crossfeedDirectGain crossfeedCrossGain
   crossfeedHfAttenuation crossfeedHfCutoff
   repeatMode singleMode partyMode shuffle playerName")

(defn get
  "All current settings as a kebab-case map."
  [client]
  (:global-settings
   (t/execute client (str "query GlobalSettings { globalSettings { " settings-fields " } }"))))

(defn save
  "Persist a partial settings map. Only the keys you pass are written.
  Keys may be kebab-case keywords (e.g. `:eq-enabled`, `:repeat-mode`).

  Nested settings (`:eq-band-settings`, `:replaygain-settings`,
  `:compressor-settings`) accept the obvious shapes; see README for examples."
  [client settings]
  (t/execute client
             "mutation SaveSettings($settings: NewGlobalSettings!) { saveSettings(settings: $settings) }"
             {:settings settings})
  client)
