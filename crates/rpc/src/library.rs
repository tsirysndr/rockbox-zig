use rockbox_library::repo;
use sqlx::Sqlite;

use crate::api::rockbox::v1alpha1::{
    library_service_server::LibraryService, GetAlbumRequest, GetAlbumResponse, GetAlbumsRequest,
    GetAlbumsResponse, GetArtistRequest, GetArtistResponse, GetArtistsRequest, GetArtistsResponse,
    GetTrackRequest, GetTrackResponse, GetTracksRequest, GetTracksResponse,
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
        Ok(tonic::Response::new(GetAlbumResponse {
            album: album.map(|a| a.into()),
        }))
    }

    async fn get_artist(
        &self,
        request: tonic::Request<GetArtistRequest>,
    ) -> Result<tonic::Response<GetArtistResponse>, tonic::Status> {
        let params = request.into_inner();
        let artist = repo::artist::find(self.pool.clone(), &params.id)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(GetArtistResponse {
            artist: artist.map(|a| a.into()),
        }))
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
}
