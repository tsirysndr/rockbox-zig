pub mod rockbox {
    pub mod search {
        pub mod v1alpha1 {
            include!("./pb/rockbox.search.v1alpha1.rs");
        }
    }
}

use anyhow::Error;
use prost::Message;
use std::ffi::CString;

use rockbox::search::v1alpha1::*;

extern "C" {
    fn IndexAlbum(data: *const u8, size: i32);
    fn IndexArtist(data: *const u8, size: i32);
    fn IndexFile(data: *const u8, size: i32);
    fn IndexLikedTrack(data: *const u8, size: i32);
    fn IndexLikedAlbum(data: *const u8, size: i32);
    fn IndexTrack(data: *const u8, size: i32);
    fn IndexAlbums(data: *const u8, size: i32);
    fn IndexArtists(data: *const u8, size: i32);
    fn IndexFiles(data: *const u8, size: i32);
    fn IndexLikedTracks(data: *const u8, size: i32);
    fn IndexLikedAlbums(data: *const u8, size: i32);
    fn IndexTracks(data: *const u8, size: i32);
    fn SearchAlbum(term: *const u8) -> *const u8;
    fn SearchArtist(term: *const u8) -> *const u8;
    fn SearchFile(term: *const u8) -> *const u8;
    fn SearchTrack(term: *const u8) -> *const u8;
    fn SearchLikedTrack(term: *const u8) -> *const u8;
    fn SearchLikedAlbum(term: *const u8) -> *const u8;
}

pub fn index_album(album: Album) -> Result<(), Error> {
    let mut buf = Vec::new();
    album.encode(&mut buf)?;
    unsafe {
        IndexAlbum(buf.as_ptr() as *const u8, buf.len() as i32);
    }
    Ok(())
}

pub fn index_artist(artist: Artist) -> Result<(), Error> {
    let mut buf = Vec::new();
    artist.encode(&mut buf)?;
    unsafe {
        IndexArtist(buf.as_ptr() as *const u8, buf.len() as i32);
    }
    Ok(())
}

pub fn index_file(file: File) -> Result<(), Error> {
    let mut buf = Vec::new();
    file.encode(&mut buf)?;
    unsafe {
        IndexFile(buf.as_ptr() as *const u8, buf.len() as i32);
    }
    Ok(())
}

pub fn index_liked_track(liked_track: LikedTrack) -> Result<(), Error> {
    let mut buf = Vec::new();
    liked_track.encode(&mut buf)?;
    unsafe {
        IndexLikedTrack(buf.as_ptr() as *const u8, buf.len() as i32);
    }
    Ok(())
}

pub fn index_liked_album(liked_album: LikedAlbum) -> Result<(), Error> {
    let mut buf = Vec::new();
    liked_album.encode(&mut buf)?;
    unsafe {
        IndexLikedAlbum(buf.as_ptr() as *const u8, buf.len() as i32);
    }
    Ok(())
}

pub fn index_track(track: Track) -> Result<(), Error> {
    let mut buf = Vec::new();
    track.encode(&mut buf)?;
    unsafe {
        IndexTrack(buf.as_ptr() as *const u8, buf.len() as i32);
    }
    Ok(())
}

pub fn index_albums(albums: AlbumList) -> Result<(), Error> {
    let mut buf = Vec::new();
    albums.encode(&mut buf)?;
    unsafe {
        IndexAlbums(buf.as_ptr() as *const u8, buf.len() as i32);
    }
    Ok(())
}

pub fn index_artists(artists: ArtistList) -> Result<(), Error> {
    let mut buf = Vec::new();
    artists.encode(&mut buf)?;
    unsafe {
        IndexArtists(buf.as_ptr() as *const u8, buf.len() as i32);
    }
    Ok(())
}

pub fn index_files(files: FileList) -> Result<(), Error> {
    let mut buf = Vec::new();
    files.encode(&mut buf)?;
    unsafe {
        IndexFiles(buf.as_ptr() as *const u8, buf.len() as i32);
    }
    Ok(())
}

pub fn index_liked_tracks(liked_tracks: LikedTrackList) -> Result<(), Error> {
    let mut buf = Vec::new();
    liked_tracks.encode(&mut buf)?;
    unsafe {
        IndexLikedTracks(buf.as_ptr() as *const u8, buf.len() as i32);
    }
    Ok(())
}

pub fn index_liked_albums(liked_albums: LikedAlbumList) -> Result<(), Error> {
    let mut buf = Vec::new();
    liked_albums.encode(&mut buf)?;
    unsafe {
        IndexLikedAlbums(buf.as_ptr() as *const u8, buf.len() as i32);
    }
    Ok(())
}

pub fn index_tracks(tracks: TrackList) -> Result<(), Error> {
    let mut buf = Vec::new();
    tracks.encode(&mut buf)?;
    unsafe {
        IndexTracks(buf.as_ptr() as *const u8, buf.len() as i32);
    }
    Ok(())
}

pub fn search_album(term: &str) -> Result<AlbumList, Error> {
    let term = CString::new(term).unwrap();
    let result = unsafe { SearchAlbum(term.as_ptr() as *const u8) };
    let result = unsafe { CString::from_raw(result as *mut i8) };
    let albums = AlbumList::decode(result.as_bytes())?;
    Ok(albums)
}

pub fn search_artist(term: &str) -> Result<ArtistList, Error> {
    let term = CString::new(term).unwrap();
    let result = unsafe { SearchArtist(term.as_ptr() as *const u8) };
    let result = unsafe { CString::from_raw(result as *mut i8) };
    let artists = ArtistList::decode(result.as_bytes())?;
    Ok(artists)
}

pub fn search_file(term: &str) -> Result<FileList, Error> {
    let term = CString::new(term).unwrap();
    let result = unsafe { SearchFile(term.as_ptr() as *const u8) };
    let result = unsafe { CString::from_raw(result as *mut i8) };
    let files = FileList::decode(result.as_bytes())?;
    Ok(files)
}

pub fn search_track(term: &str) -> Result<TrackList, Error> {
    let term = CString::new(term).unwrap();
    let result = unsafe { SearchTrack(term.as_ptr() as *const u8) };
    let result = unsafe { CString::from_raw(result as *mut i8) };
    let tracks = TrackList::decode(result.as_bytes())?;
    Ok(tracks)
}

pub fn search_liked_track(term: &str) -> Result<LikedTrackList, Error> {
    let term = CString::new(term).unwrap();
    let result = unsafe { SearchLikedTrack(term.as_ptr() as *const u8) };
    let result = unsafe { CString::from_raw(result as *mut i8) };
    let liked_tracks = LikedTrackList::decode(result.as_bytes())?;
    Ok(liked_tracks)
}

pub fn search_liked_album(term: &str) -> Result<LikedAlbumList, Error> {
    let term = CString::new(term).unwrap();
    let result = unsafe { SearchLikedAlbum(term.as_ptr() as *const u8) };
    let result = unsafe { CString::from_raw(result as *mut i8) };
    let liked_albums = LikedAlbumList::decode(result.as_bytes())?;
    Ok(liked_albums)
}
