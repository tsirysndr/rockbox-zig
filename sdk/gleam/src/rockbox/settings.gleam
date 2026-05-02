//// Read and patch the global daemon settings (volume, EQ, crossfeed, …).
////
//// Use the `update_*` builders to construct a partial update — only the
//// fields you set are sent to the server:
////
//// ```gleam
//// let patch =
////   settings.patch()
////   |> settings.set_volume(-20)
////   |> settings.set_shuffle(True)
////
//// let assert Ok(_) = settings.save(client, patch)
//// ```

import gleam/dynamic/decode
import gleam/json.{type Json}
import gleam/list
import rockbox.{type Client}
import rockbox/error.{type Error}
import rockbox/types.{
  type CompressorSettings, type EqBandSetting, type ReplaygainSettings,
  type UserSettings,
}

/// Read every settings field.
pub fn get(client: Client) -> Result(UserSettings, Error) {
  let decoder = {
    use settings <- decode.field("globalSettings", types.user_settings_decoder())
    decode.success(settings)
  }
  rockbox.query(
    client,
    "query GlobalSettings {
       globalSettings {
         musicDir volume balance bass treble channelConfig stereoWidth
         eqEnabled eqPrecut
         eqBandSettings { cutoff q gain }
         replaygainSettings { noclip type preamp }
         compressorSettings { threshold makeupGain ratio knee releaseTime attackTime }
         crossfadeEnabled crossfadeFadeInDelay crossfadeFadeInDuration
         crossfadeFadeOutDelay crossfadeFadeOutDuration crossfadeFadeOutMixmode
         crossfeedEnabled crossfeedDirectGain crossfeedCrossGain
         crossfeedHfAttenuation crossfeedHfCutoff
         repeatMode singleMode partyMode shuffle playerName
       }
     }",
    json.object([]),
    decoder,
  )
}

// ---------------------------------------------------------------------------
// Partial-update builder
// ---------------------------------------------------------------------------

pub opaque type Patch {
  Patch(fields: List(#(String, Json)))
}

/// Empty patch — chain `set_*` to populate.
pub fn patch() -> Patch {
  Patch(fields: [])
}

fn set(patch: Patch, key: String, value: Json) -> Patch {
  Patch(fields: [#(key, value), ..patch.fields])
}

pub fn set_music_dir(patch: Patch, value: String) -> Patch {
  set(patch, "musicDir", json.string(value))
}

pub fn set_volume(patch: Patch, value: Int) -> Patch {
  set(patch, "volume", json.int(value))
}

pub fn set_balance(patch: Patch, value: Int) -> Patch {
  set(patch, "balance", json.int(value))
}

pub fn set_bass(patch: Patch, value: Int) -> Patch {
  set(patch, "bass", json.int(value))
}

pub fn set_treble(patch: Patch, value: Int) -> Patch {
  set(patch, "treble", json.int(value))
}

pub fn set_channel_config(patch: Patch, value: Int) -> Patch {
  set(patch, "channelConfig", json.int(value))
}

pub fn set_stereo_width(patch: Patch, value: Int) -> Patch {
  set(patch, "stereoWidth", json.int(value))
}

pub fn set_eq_enabled(patch: Patch, value: Bool) -> Patch {
  set(patch, "eqEnabled", json.bool(value))
}

pub fn set_eq_precut(patch: Patch, value: Int) -> Patch {
  set(patch, "eqPrecut", json.int(value))
}

pub fn set_eq_bands(patch: Patch, value: List(EqBandSetting)) -> Patch {
  let encoder = fn(band: EqBandSetting) {
    json.object([
      #("cutoff", json.int(band.cutoff)),
      #("q", json.int(band.q)),
      #("gain", json.int(band.gain)),
    ])
  }
  set(patch, "eqBandSettings", json.array(value, encoder))
}

pub fn set_replaygain(patch: Patch, value: ReplaygainSettings) -> Patch {
  set(
    patch,
    "replaygainSettings",
    json.object([
      #("noclip", json.bool(value.noclip)),
      #("type", json.int(value.type_)),
      #("preamp", json.int(value.preamp)),
    ]),
  )
}

pub fn set_compressor(patch: Patch, value: CompressorSettings) -> Patch {
  set(
    patch,
    "compressorSettings",
    json.object([
      #("threshold", json.int(value.threshold)),
      #("makeupGain", json.int(value.makeup_gain)),
      #("ratio", json.int(value.ratio)),
      #("knee", json.int(value.knee)),
      #("releaseTime", json.int(value.release_time)),
      #("attackTime", json.int(value.attack_time)),
    ]),
  )
}

pub fn set_crossfade_enabled(patch: Patch, value: Int) -> Patch {
  set(patch, "crossfadeEnabled", json.int(value))
}

pub fn set_crossfade_fade_in_delay(patch: Patch, value: Int) -> Patch {
  set(patch, "crossfadeFadeInDelay", json.int(value))
}

pub fn set_crossfade_fade_in_duration(patch: Patch, value: Int) -> Patch {
  set(patch, "crossfadeFadeInDuration", json.int(value))
}

pub fn set_crossfade_fade_out_delay(patch: Patch, value: Int) -> Patch {
  set(patch, "crossfadeFadeOutDelay", json.int(value))
}

pub fn set_crossfade_fade_out_duration(patch: Patch, value: Int) -> Patch {
  set(patch, "crossfadeFadeOutDuration", json.int(value))
}

pub fn set_crossfade_fade_out_mixmode(patch: Patch, value: Int) -> Patch {
  set(patch, "crossfadeFadeOutMixmode", json.int(value))
}

pub fn set_crossfeed_enabled(patch: Patch, value: Bool) -> Patch {
  set(patch, "crossfeedEnabled", json.bool(value))
}

pub fn set_crossfeed_direct_gain(patch: Patch, value: Int) -> Patch {
  set(patch, "crossfeedDirectGain", json.int(value))
}

pub fn set_crossfeed_cross_gain(patch: Patch, value: Int) -> Patch {
  set(patch, "crossfeedCrossGain", json.int(value))
}

pub fn set_crossfeed_hf_attenuation(patch: Patch, value: Int) -> Patch {
  set(patch, "crossfeedHfAttenuation", json.int(value))
}

pub fn set_crossfeed_hf_cutoff(patch: Patch, value: Int) -> Patch {
  set(patch, "crossfeedHfCutoff", json.int(value))
}

pub fn set_repeat_mode(patch: Patch, value: Int) -> Patch {
  set(patch, "repeatMode", json.int(value))
}

pub fn set_single_mode(patch: Patch, value: Bool) -> Patch {
  set(patch, "singleMode", json.bool(value))
}

pub fn set_party_mode(patch: Patch, value: Bool) -> Patch {
  set(patch, "partyMode", json.bool(value))
}

pub fn set_shuffle(patch: Patch, value: Bool) -> Patch {
  set(patch, "shuffle", json.bool(value))
}

pub fn set_player_name(patch: Patch, value: String) -> Patch {
  set(patch, "playerName", json.string(value))
}

/// Push the patch to the daemon.
pub fn save(client: Client, patch: Patch) -> Result(Nil, Error) {
  let payload = json.object(list.reverse(patch.fields))
  rockbox.execute(
    client,
    "mutation SaveSettings($settings: NewGlobalSettings!) { saveSettings(settings: $settings) }",
    json.object([#("settings", payload)]),
  )
}
