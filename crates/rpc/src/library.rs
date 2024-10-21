use std::env;

use rockbox_library::{audio_scan::scan_audio_files, entity::favourites::Favourites, repo};
use sqlx::Sqlite;

use crate::api::rockbox::v1alpha1::{
    library_service_server::LibraryService, Album, Artist, GetAlbumRequest, GetAlbumResponse,
    GetAlbumsRequest, GetAlbumsResponse, GetArtistRequest, GetArtistResponse, GetArtistsRequest,
    GetArtistsResponse, GetLikedAlbumsRequest, GetLikedAlbumsResponse, GetLikedTracksRequest,
    GetLikedTracksResponse, GetTrackRequest, GetTrackResponse, GetTracksRequest, GetTracksResponse,
    LikeAlbumRequest, LikeAlbumResponse, LikeTrackRequest, LikeTrackResponse, ScanLibraryRequest,
    ScanLibraryResponse, UnlikeAlbumRequest, UnlikeAlbumResponse, UnlikeTrackRequest,
    UnlikeTrackResponse,
};

pub struct Library {
    pool: sqlx::Pool<Sqlite>,
}

impl Library {
    pub fn new(pool: sqlx::Pool<Sqlite>) -> Self {
        Self { pool }
    }
}

#[tonic::async_trait]
impl LibraryService for Library {
    async fn get_albums(
        &self,
        _request: tonic::Request<GetAlbumsRequest>,
    ) -> Result<tonic::Response<GetAlbumsResponse>, tonic::Status> {
        let albums = repo::album::all(self.pool.clone())
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(GetAlbumsResponse {
            albums: albums.into_iter().map(|a| a.into()).collect(),
        }))
    }

    async fn get_artists(
        &self,
        _request: tonic::Request<GetArtistsRequest>,
    ) -> Result<tonic::Response<GetArtistsResponse>, tonic::Status> {
        let artists = repo::artist::all(self.pool.clone())
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(GetArtistsResponse {
            artists: artists.into_iter().map(|a| a.into()).collect(),
        }))
    }

    async fn get_tracks(
        &self,
        _request: tonic::Request<GetTracksRequest>,
    ) -> Result<tonic::Response<GetTracksResponse>, tonic::Status> {
        let tracks = repo::track::all(self.pool.clone())
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(GetTracksResponse {
            tracks: tracks.into_iter().map(|t| t.into()).collect(),
        }))
    }

    async fn get_album(
        &self,
        request: tonic::Request<GetAlbumRequest>,
    ) -> Result<tonic::Response<GetAlbumResponse>, tonic::Status> {
        let params = request.into_inner();
        let album = repo::album::find(self.pool.clone(), &params.id)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        let mut album: Option<Album> = album.map(|a| a.into());
        let tracks = repo::album_tracks::find_by_album(self.pool.clone(), &params.id)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        if let Some(album) = album.as_mut() {
            album.tracks = tracks.into_iter().map(|t| t.into()).collect();
        }

        let response = GetAlbumResponse { album };
        Ok(tonic::Response::new(response))
    }

    async fn get_artist(
        &self,
        request: tonic::Request<GetArtistRequest>,
    ) -> Result<tonic::Response<GetArtistResponse>, tonic::Status> {
        let params = request.into_inner();
        let artist = repo::artist::find(self.pool.clone(), &params.id)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        let mut artist: Option<Artist> = artist.map(|a| a.into());
        let albums = repo::album::find_by_artist(self.pool.clone(), &params.id)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        let tracks = repo::artist_tracks::find_by_artist(self.pool.clone(), &params.id)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        if let Some(artist) = artist.as_mut() {
            artist.albums = albums.into_iter().map(|a| a.into()).collect();
            artist.tracks = tracks.into_iter().map(|t| t.into()).collect();
        }

        Ok(tonic::Response::new(GetArtistResponse { artist }))
    }

    async fn get_track(
        &self,
        request: tonic::Request<GetTrackRequest>,
    ) -> Result<tonic::Response<GetTrackResponse>, tonic::Status> {
        let params = request.into_inner();
        let track = repo::track::find(self.pool.clone(), &params.id)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(GetTrackResponse {
            track: track.map(|t| t.into()),
        }))
    }

    async fn like_track(
        &self,
        request: tonic::Request<LikeTrackRequest>,
    ) -> Result<tonic::Response<LikeTrackResponse>, tonic::Status> {
        let params = request.into_inner();
        repo::favourites::save(
            self.pool.clone(),
            Favourites {
                id: cuid::cuid1().map_err(|e| tonic::Status::internal(e.to_string()))?,
                track_id: Some(params.id),
                created_at: chrono::Utc::now(),
                album_id: None,
            },
        )
        .await
        .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(LikeTrackResponse {}))
    }

    async fn like_album(
        &self,
        request: tonic::Request<LikeAlbumRequest>,
    ) -> Result<tonic::Response<LikeAlbumResponse>, tonic::Status> {
        let params = request.into_inner();
        repo::favourites::save(
            self.pool.clone(),
            Favourites {
                id: cuid::cuid1().map_err(|e| tonic::Status::internal(e.to_string()))?,
                track_id: None,
                created_at: chrono::Utc::now(),
                album_id: Some(params.id),
            },
        )
        .await
        .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(LikeAlbumResponse {}))
    }

    async fn unlike_track(
        &self,
        request: tonic::Request<UnlikeTrackRequest>,
    ) -> Result<tonic::Response<UnlikeTrackResponse>, tonic::Status> {
        let params = request.into_inner();
        repo::favourites::delete(self.pool.clone(), &params.id)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(UnlikeTrackResponse {}))
    }

    async fn unlike_album(
        &self,
        request: tonic::Request<UnlikeAlbumRequest>,
    ) -> Result<tonic::Response<UnlikeAlbumResponse>, tonic::Status> {
        let params = request.into_inner();
        repo::favourites::delete(self.pool.clone(), &params.id)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(UnlikeAlbumResponse {}))
    }

    async fn get_liked_tracks(
        &self,
        _request: tonic::Request<GetLikedTracksRequest>,
    ) -> Result<tonic::Response<GetLikedTracksResponse>, tonic::Status> {
        let tracks = repo::favourites::all_tracks(self.pool.clone())
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(GetLikedTracksResponse {
            tracks: tracks.into_iter().map(|t| t.into()).collect(),
        }))
    }

    async fn get_liked_albums(
        &self,
        _request: tonic::Request<GetLikedAlbumsRequest>,
    ) -> Result<tonic::Response<GetLikedAlbumsResponse>, tonic::Status> {
        let albums = repo::favourites::all_albums(self.pool.clone())
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(GetLikedAlbumsResponse {
            albums: albums.into_iter().map(|a| a.into()).collect(),
        }))
    }

    async fn scan_library(
        &self,
        _request: tonic::Request<ScanLibraryRequest>,
    ) -> Result<tonic::Response<ScanLibraryResponse>, tonic::Status> {
        let home = env::var("HOME").map_err(|e| tonic::Status::internal(e.to_string()))?;
        let path = env::var("ROCKBOX_LIBRARY").unwrap_or(format!("{}/Music", home));

        scan_audio_files(self.pool.clone(), path.into())
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        Ok(tonic::Response::new(ScanLibraryResponse {}))
    }
}
