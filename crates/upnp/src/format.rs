/// Return the HTTP Content-Type for a given file path, based on the extension.
///
/// Covers every format in Rockbox's `audio_formats[]` array
/// (lib/rbcodec/metadata/metadata.c).
pub fn content_type_for_path(path: &str) -> &'static str {
    let ext = path
        .rsplit('.')
        .next()
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_default();
    content_type_for_ext(&ext)
}

/// Return the UPnP `protocolInfo` source string for a given file path.
/// Format: `http-get:*:<mime>:*`
pub fn protocol_info_for_path(path: &str) -> String {
    format!("http-get:*:{}:*", content_type_for_path(path))
}

fn content_type_for_ext(ext: &str) -> &'static str {
    match ext {
        // ── MPEG Audio (MP1 / MP2 / MP3) ─────────────────────────────────
        "mp3" | "mp2" | "mp1" | "mpa" => "audio/mpeg",

        // ── AIFF ──────────────────────────────────────────────────────────
        "aiff" | "aif" => "audio/aiff",

        // ── WAV / WAVE64 / ATRAC3-in-WAV ─────────────────────────────────
        "wav" | "at3" => "audio/wav",
        "w64" => "audio/x-w64",

        // ── Ogg Vorbis ────────────────────────────────────────────────────
        "ogg" | "oga" => "audio/ogg",

        // ── Opus ──────────────────────────────────────────────────────────
        "opus" => "audio/opus",

        // ── Speex (Ogg container) ─────────────────────────────────────────
        "spx" => "audio/ogg",

        // ── FLAC ──────────────────────────────────────────────────────────
        "flac" => "audio/flac",

        // ── AAC / ALAC / MP4 ──────────────────────────────────────────────
        "m4a" | "m4b" | "mp4" => "audio/mp4",
        "aac" => "audio/aac",

        // ── WavPack ───────────────────────────────────────────────────────
        "wv" => "audio/x-wavpack",

        // ── Monkey's Audio ────────────────────────────────────────────────
        "ape" | "mac" => "audio/x-ape",

        // ── Musepack ──────────────────────────────────────────────────────
        "mpc" => "audio/x-musepack",

        // ── AC3 / A52 ─────────────────────────────────────────────────────
        "ac3" | "a52" => "audio/ac3",

        // ── WMA / WMV / ASF ───────────────────────────────────────────────
        "wma" | "wmv" | "asf" => "audio/x-ms-wma",

        // ── RealMedia ─────────────────────────────────────────────────────
        "rm" | "ra" | "rmvb" => "audio/x-pn-realaudio",

        // ── True Audio ────────────────────────────────────────────────────
        "tta" => "audio/x-tta",

        // ── Shorten ───────────────────────────────────────────────────────
        "shn" => "audio/x-shorten",

        // ── Sun Audio / AU ────────────────────────────────────────────────
        "au" | "snd" => "audio/basic",

        // ── Sony ATRAC3 in OMA container ─────────────────────────────────
        "oma" | "aa3" => "audio/x-sony-oma",

        // ── SMAF (mobile ringtone) ────────────────────────────────────────
        "mmf" => "application/x-smaf",

        // ── Dialogic VOX ─────────────────────────────────────────────────
        "vox" => "audio/vox",

        // ── ADX (CRI Middleware) ──────────────────────────────────────────
        "adx" => "audio/x-adx",

        // ── MOD (Amiga tracker) ───────────────────────────────────────────
        "mod" => "audio/mod",

        // ── Chiptune / game music formats ─────────────────────────────────
        // SID (C64)
        "sid" => "audio/prs.sid",
        // NES Sound Format
        "nsf" | "nsfe" => "audio/x-nsf",
        // SPC (SNES)
        "spc" => "audio/x-spc",
        // Atari SAP / ASAP family
        "sap" | "cmc" | "cm3" | "cmr" | "cms" | "dmc" | "dlt" | "mpt" | "mpd" | "rmt" | "tmc"
        | "tm8" | "tm2" => "audio/x-asap",
        // AY (ZX Spectrum / Amstrad)
        "ay" => "audio/x-ay",
        // VTX (ZX Spectrum)
        "vtx" => "audio/x-vtx",
        // GBS (Game Boy)
        "gbs" => "audio/x-gbs",
        // HES (PC Engine / TurboGrafx)
        "hes" => "audio/x-hes",
        // SGC (Sega Master System / Game Gear / Coleco)
        "sgc" => "audio/x-sgc",
        // VGM (Video Game Music)
        "vgm" | "vgz" => "audio/x-vgm",
        // KSS (MSX)
        "kss" => "audio/x-kss",

        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn common_formats() {
        assert_eq!(content_type_for_path("song.mp3"), "audio/mpeg");
        assert_eq!(content_type_for_path("song.flac"), "audio/flac");
        assert_eq!(content_type_for_path("song.ogg"), "audio/ogg");
        assert_eq!(content_type_for_path("song.opus"), "audio/opus");
        assert_eq!(content_type_for_path("song.m4a"), "audio/mp4");
        assert_eq!(content_type_for_path("song.wav"), "audio/wav");
        assert_eq!(content_type_for_path("song.wv"), "audio/x-wavpack");
        assert_eq!(content_type_for_path("song.ape"), "audio/x-ape");
    }

    #[test]
    fn mpeg_variants() {
        assert_eq!(content_type_for_path("song.mp1"), "audio/mpeg");
        assert_eq!(content_type_for_path("song.mp2"), "audio/mpeg");
        assert_eq!(content_type_for_path("song.mpa"), "audio/mpeg");
    }

    #[test]
    fn game_formats() {
        assert_eq!(content_type_for_path("game.sid"), "audio/prs.sid");
        assert_eq!(content_type_for_path("game.nsf"), "audio/x-nsf");
        assert_eq!(content_type_for_path("game.spc"), "audio/x-spc");
        assert_eq!(content_type_for_path("game.vgm"), "audio/x-vgm");
        assert_eq!(content_type_for_path("game.kss"), "audio/x-kss");
    }

    #[test]
    fn unknown_falls_back() {
        assert_eq!(
            content_type_for_path("file.xyz"),
            "application/octet-stream"
        );
    }
}
