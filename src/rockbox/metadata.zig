pub const MAX_PATH = 260;
pub const ID3V2_BUF_SIZE = 1800;

const mp3_aa_type = enum(c_int) {
    AA_TYPE_UNSYNC = -1,
    AA_TYPE_UNKNOWN = 0,
    AA_TYPE_BMP,
    AA_TYPE_PNG,
    AA_TYPE_JPG,
};

const mp3_albumart = extern struct {
    type: mp3_aa_type,
    size: c_int,
    pos: c_long, // Use c_long to match the typical size of off_t, but adjust as necessary
};

pub const mp3entry = extern struct {
    path: [MAX_PATH]u8,
    title: ?*[*c]u8,
    artist: ?*[*c]u8,
    album: ?*[*c]u8,
    genre_string: ?*[*c]u8,
    disc_string: ?*[*c]u8,
    track_string: ?*[*c]u8,
    year_string: ?*[*c]u8,
    composer: ?*[*c]u8,
    comment: ?*[*c]u8,
    albumartist: ?*[*c]u8,
    grouping: ?*[*c]u8,
    discnum: c_int,
    tracknum: c_int,
    layer: c_int,
    year: c_int,
    id3version: u8,
    codectype: u32,
    bitrate: u32,
    frequency: u32,
    id3v2len: u32,
    id3v1len: u32,
    first_frame_offset: u32,
    filesize: u32,
    length: u32,
    elapsed: u32,
    lead_trim: c_int,
    tail_trim: c_int,
    samples: u64,
    frame_count: u32,
    bytesperframe: u32,
    vbr: bool,
    has_toc: bool,
    toc: [100]u8,
    needs_upsampling_correction: bool,
    id3v2buf: [ID3V2_BUF_SIZE]u8,
    id3v1buf: [4][92]u8,
    offset: u32,
    index: c_int,
    skip_resume_adjustments: bool,
    autoresumable: u8,
    tagcache_idx: c_long,
    rating: c_int,
    score: c_int,
    playcount: c_long,
    lastplayed: c_long,
    playtime: c_long,
    track_level: c_long,
    album_level: c_long,
    track_gain: c_long,
    album_gain: c_long,
    track_peak: c_long,
    album_peak: c_long,
    has_embedded_albumart: bool,
    albumart: mp3_albumart,
    has_embedded_cuesheet: bool,
    mb_track_id: ?*[*c]u8,
    is_asf_stream: bool,
};

extern fn get_metadata(id3: *mp3entry, fd: c_int, trackname: [*]const u8) bool;

pub fn _get_metadata(fd: c_int, trackname: [*]const u8) mp3entry {
    var id3: mp3entry = undefined;
    _ = get_metadata(&id3, fd, trackname);
    return id3;
}
