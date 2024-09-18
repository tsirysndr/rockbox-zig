use crate::cast_ptr;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Mp3Entry {
    pub path: String,
    pub title: String,                     // char* title
    pub artist: String,                    // char* artist
    pub album: String,                     // char* album
    pub genre_string: String,              // char* genre_string
    pub disc_string: String,               // char* disc_string
    pub track_string: String,              // char* track_string
    pub year_string: String,               // char* year_string
    pub composer: String,                  // char* composer
    pub comment: String,                   // char* comment
    pub albumartist: String,               // char* albumartist
    pub grouping: String,                  // char* grouping
    pub discnum: i32,                      // int discnum
    pub tracknum: i32,                     // int tracknum
    pub layer: i32,                        // int layer
    pub year: i32,                         // int year
    pub id3version: i32,                   // unsigned char id3version
    pub codectype: u32,                    // unsigned int codectype
    pub bitrate: u32,                      // unsigned int bitrate
    pub frequency: u64,                    // unsigned long frequency
    pub id3v2len: u64,                     // unsigned long id3v2len
    pub id3v1len: u64,                     // unsigned long id3v1len
    pub first_frame_offset: u64,           // unsigned long first_frame_offset
    pub filesize: u64,                     // unsigned long filesize
    pub length: u64,                       // unsigned long length
    pub elapsed: u64,                      // unsigned long elapsed
    pub lead_trim: i32,                    // int lead_trim
    pub tail_trim: i32,                    // int tail_trim
    pub samples: u64,                      // uint64_t samples
    pub frame_count: u64,                  // unsigned long frame_count
    pub bytesperframe: u64,                // unsigned long bytesperframe
    pub vbr: bool,                         // bool vbr
    pub has_toc: bool,                     // bool has_toc
    pub toc: String,                       // unsigned char toc[100]
    pub needs_upsampling_correction: bool, // bool needs_upsampling_correction
    pub offset: u64,                       // unsigned long offset
    pub index: i32,                        // int index
    pub skip_resume_adjustments: bool,     // bool skip_resume_adjustments
    pub autoresumable: i32,                // unsigned char autoresumable
    pub tagcache_idx: i64,                 // long tagcache_idx
    pub rating: i32,                       // int rating
    pub score: i32,                        // int score
    pub playcount: i64,                    // long playcount
    pub lastplayed: i64,                   // long lastplayed
    pub playtime: i64,                     // long playtime
    pub track_level: i64,                  // long track_level
    pub album_level: i64,                  // long album_level
    pub track_gain: i64,                   // long track_gain
    pub album_gain: i64,                   // long album_gain
    pub track_peak: i64,                   // long track_peak
    pub album_peak: i64,                   // long album_peak
    pub has_embedded_albumart: bool,       // bool has_embedded_albumart
    pub mb_track_id: String,               // char* mb_track_id
    pub is_asf_stream: bool,               // bool is_asf_stream
}

impl From<crate::Mp3Entry> for Mp3Entry {
    fn from(entry: crate::Mp3Entry) -> Self {
        Self {
            path: unsafe {
                std::ffi::CStr::from_ptr(cast_ptr!(entry.path.as_ptr()))
                    .to_string_lossy()
                    .into_owned()
            },
            title: unsafe {
                std::ffi::CStr::from_ptr(entry.title)
                    .to_string_lossy()
                    .into_owned()
            },
            artist: unsafe {
                std::ffi::CStr::from_ptr(entry.artist)
                    .to_string_lossy()
                    .into_owned()
            },
            album: unsafe {
                std::ffi::CStr::from_ptr(entry.album)
                    .to_string_lossy()
                    .into_owned()
            },
            genre_string: unsafe {
                match entry.genre_string.is_null() {
                    true => String::new(),
                    false => std::ffi::CStr::from_ptr(entry.genre_string)
                        .to_string_lossy()
                        .into_owned(),
                }
            },
            disc_string: unsafe {
                match entry.disc_string.is_null() {
                    true => String::new(),
                    false => std::ffi::CStr::from_ptr(entry.disc_string)
                        .to_string_lossy()
                        .into_owned(),
                }
            },
            track_string: unsafe {
                match entry.track_string.is_null() {
                    true => String::new(),
                    false => std::ffi::CStr::from_ptr(entry.track_string)
                        .to_string_lossy()
                        .into_owned(),
                }
            },
            year_string: unsafe {
                match entry.year_string.is_null() {
                    true => String::new(),
                    false => std::ffi::CStr::from_ptr(entry.year_string)
                        .to_string_lossy()
                        .into_owned(),
                }
            },
            composer: unsafe {
                match entry.composer.is_null() {
                    true => String::new(),
                    false => std::ffi::CStr::from_ptr(entry.composer)
                        .to_string_lossy()
                        .into_owned(),
                }
            },
            comment: unsafe {
                match entry.comment.is_null() {
                    true => String::new(),
                    false => std::ffi::CStr::from_ptr(entry.comment)
                        .to_string_lossy()
                        .into_owned(),
                }
            },
            albumartist: unsafe {
                match entry.albumartist.is_null() {
                    true => String::new(),
                    false => std::ffi::CStr::from_ptr(entry.albumartist)
                        .to_string_lossy()
                        .into_owned(),
                }
            },
            grouping: unsafe {
                match entry.grouping.is_null() {
                    true => String::new(),
                    false => std::ffi::CStr::from_ptr(entry.grouping)
                        .to_string_lossy()
                        .into_owned(),
                }
            },
            discnum: entry.discnum,
            tracknum: entry.tracknum,
            layer: entry.layer,
            year: entry.year,
            id3version: entry.id3version as i32,
            codectype: entry.codectype,
            bitrate: entry.bitrate,
            frequency: entry.frequency,
            id3v2len: entry.id3v2len,
            id3v1len: entry.id3v1len,
            first_frame_offset: entry.first_frame_offset,
            filesize: entry.filesize,
            length: entry.length,
            elapsed: entry.elapsed,
            lead_trim: entry.lead_trim,
            tail_trim: entry.tail_trim,
            samples: entry.samples,
            frame_count: entry.frame_count,
            bytesperframe: entry.bytesperframe,
            vbr: entry.vbr,
            has_toc: entry.has_toc,
            toc: unsafe {
                std::ffi::CStr::from_ptr(cast_ptr!(entry.toc.as_ptr()))
                    .to_string_lossy()
                    .into_owned()
            },
            needs_upsampling_correction: entry.needs_upsampling_correction,
            offset: entry.offset,
            index: entry.index,
            skip_resume_adjustments: entry.skip_resume_adjustments,
            autoresumable: entry.autoresumable as i32,
            tagcache_idx: entry.tagcache_idx,
            rating: entry.rating,
            score: entry.score,
            playcount: entry.playcount,
            lastplayed: entry.lastplayed,
            playtime: entry.playtime,
            track_level: entry.track_level,
            album_level: entry.album_level,
            track_gain: entry.track_gain,
            album_gain: entry.album_gain,
            track_peak: entry.track_peak,
            album_peak: entry.album_peak,
            has_embedded_albumart: entry.has_embedded_albumart,
            mb_track_id: unsafe {
                match entry.mb_track_id.is_null() {
                    true => String::new(),
                    false => std::ffi::CStr::from_ptr(entry.mb_track_id)
                        .to_string_lossy()
                        .into_owned(),
                }
            },
            is_asf_stream: entry.is_asf_stream,
        }
    }
}
