pub mod browse;
pub mod metadata;
pub mod playback;
pub mod playlist;
pub mod server;
pub mod settings;
pub mod sound;
pub mod tagcache;

pub mod api {
    #[path = ""]
    pub mod rockbox {
        use rockbox_sys::Mp3Entry;
        use v1alpha1::CurrentTrackResponse;

        #[path = "rockbox.v1alpha1.rs"]
        pub mod v1alpha1;

        pub(crate) const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!("api/rockbox_descriptor.bin");

        impl From<Mp3Entry> for CurrentTrackResponse {
            fn from(mp3entry: Mp3Entry) -> Self {
                let title = match mp3entry.title.is_null() {
                    true => "No title".to_string(),
                    false => unsafe {
                        std::ffi::CStr::from_ptr(mp3entry.title)
                            .to_string_lossy()
                            .to_string()
                    },
                };
                let artist = match mp3entry.artist.is_null() {
                    true => "No artist".to_string(),
                    false => unsafe {
                        std::ffi::CStr::from_ptr(mp3entry.artist)
                            .to_string_lossy()
                            .to_string()
                    },
                };
                let album = match mp3entry.album.is_null() {
                    true => "No album".to_string(),
                    false => unsafe {
                        std::ffi::CStr::from_ptr(mp3entry.album)
                            .to_string_lossy()
                            .to_string()
                    },
                };
                let genre = match mp3entry.genre_string.is_null() {
                    true => "No genre".to_string(),
                    false => unsafe {
                        std::ffi::CStr::from_ptr(mp3entry.genre_string)
                            .to_string_lossy()
                            .to_string()
                    },
                };
                let disc = match mp3entry.disc_string.is_null() {
                    true => "No disc".to_string(),
                    false => unsafe {
                        std::ffi::CStr::from_ptr(mp3entry.disc_string)
                            .to_string_lossy()
                            .to_string()
                    },
                };
                let track_string = match mp3entry.track_string.is_null() {
                    true => "No track_string".to_string(),
                    false => unsafe {
                        std::ffi::CStr::from_ptr(mp3entry.track_string)
                            .to_string_lossy()
                            .to_string()
                    },
                };
                let year_string = match mp3entry.year_string.is_null() {
                    true => "No year_string".to_string(),
                    false => unsafe {
                        std::ffi::CStr::from_ptr(mp3entry.year_string)
                            .to_string_lossy()
                            .to_string()
                    },
                };
                let composer = match mp3entry.composer.is_null() {
                    true => "No composer".to_string(),
                    false => unsafe {
                        std::ffi::CStr::from_ptr(mp3entry.composer)
                            .to_string_lossy()
                            .to_string()
                    },
                };
                let album_artist = match mp3entry.albumartist.is_null() {
                    true => "No album_artist".to_string(),
                    false => unsafe {
                        std::ffi::CStr::from_ptr(mp3entry.albumartist)
                            .to_string_lossy()
                            .to_string()
                    },
                };
                let comment = match mp3entry.comment.is_null() {
                    true => "No comment".to_string(),
                    false => unsafe {
                        std::ffi::CStr::from_ptr(mp3entry.comment)
                            .to_string_lossy()
                            .to_string()
                    },
                };
                let grouping = match mp3entry.grouping.is_null() {
                    true => "No grouping".to_string(),
                    false => unsafe {
                        std::ffi::CStr::from_ptr(mp3entry.grouping)
                            .to_string_lossy()
                            .to_string()
                    },
                };
                let discnum = mp3entry.discnum;
                let tracknum = mp3entry.tracknum;
                let layer = mp3entry.layer;
                let year = mp3entry.year;
                let bitrate = mp3entry.bitrate;
                let frequency = mp3entry.frequency;
                let filesize = mp3entry.filesize;
                let length = mp3entry.length;
                let elapsed = mp3entry.elapsed;

                CurrentTrackResponse {
                    title,
                    artist,
                    album,
                    genre,
                    disc,
                    track_string,
                    year_string,
                    composer,
                    album_artist,
                    comment,
                    grouping,
                    discnum,
                    tracknum,
                    layer,
                    year,
                    bitrate,
                    frequency,
                    filesize,
                    length,
                    elapsed,
                }
            }
        }
    }
}
