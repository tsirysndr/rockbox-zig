use rockbox_library::{entity::favourites::Favourites, repo};
use rockbox_search::search_entities;
use sqlx::Sqlite;

use crate::{
    api::rockbox::v1alpha1::{
        library_service_server::LibraryService, Album, Artist, GetAlbumRequest, GetAlbumResponse,
        GetAlbumsRequest, GetAlbumsResponse, GetArtistRequest, GetArtistResponse,
        GetArtistsRequest, GetArtistsResponse, GetLikedAlbumsRequest, GetLikedAlbumsResponse,
        GetLikedTracksRequest, GetLikedTracksResponse, GetTrackRequest, GetTrackResponse,
        GetTracksRequest, GetTracksResponse, LikeAlbumRequest, LikeAlbumResponse, LikeTrackRequest,
        LikeTrackResponse, ScanLibraryRequest, ScanLibraryResponse, SearchRequest, SearchResponse,
        UnlikeAlbumRequest, UnlikeAlbumResponse, UnlikeTrackRequest, UnlikeTrackResponse,
    },
    rockbox_url,
};

pub struct Library {
    pool: sqlx::Pool<Sqlite>,
    client: reqwest::Client,
    indexes: rockbox_search::Indexes,
}

impl Library {
    pub fn new(
        pool: sqlx::Pool<Sqlite>,
        client: reqwest::Client,
        indexes: rockbox_search::Indexes,
    ) -> Self {
        Self {
            pool,
            client,
            indexes,
        }
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
                track_id: Some(params.id.clone()),
                created_at: chrono::Utc::now(),
                album_id: None,
            },
        )
        .await
        .map_err(|e| tonic::Status::internal(e.to_string()))?;

        let track = repo::track::find(self.pool.clone(), &params.id)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        if let Some(track) = track {
            let album = repo::album::find(self.pool.clone(), &track.album_id)
                .await
                .map_err(|e| tonic::Status::internal(e.to_string()))?;
            if let Some(album) = album {
                match rockbox_rocksky::like(track, album).await {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("Error liking track: {:?}", e);
                    }
                }
            }
        }

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

        let track = repo::track::find(self.pool.clone(), &params.id)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        if let Some(track) = track {
            match rockbox_rocksky::unlike(track).await {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Error unliking track: {:?}", e);
                }
            }
        }

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
        request: tonic::Request<ScanLibraryRequest>,
    ) -> Result<tonic::Response<ScanLibraryResponse>, tonic::Status> {
        let request = request.into_inner();
        let params = match request.path {
            Some(path) => format!("?path={}", path),
            None => "".to_string(),
        };
        let url = format!("{}/scan-library{}", rockbox_url(), params);
        self.client
            .put(&url)
            .send()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(ScanLibraryResponse {}))
    }

    async fn search(
        &self,
        request: tonic::Request<SearchRequest>,
    ) -> Result<tonic::Response<SearchResponse>, tonic::Status> {
        let request = request.into_inner();
        let term = request.term;

        let albums = search_entities(
            &self.indexes.albums,
            &term,
            &rockbox_search::album::Album::default(),
        )
        .map_err(|e| tonic::Status::internal(e.to_string()))?;
        let artists = search_entities(
            &self.indexes.artists,
            &term,
            &rockbox_search::artist::Artist::default(),
        )
        .map_err(|e| tonic::Status::internal(e.to_string()))?;
        let tracks = search_entities(
            &self.indexes.tracks,
            &term,
            &rockbox_search::track::Track::default(),
        )
        .map_err(|e| tonic::Status::internal(e.to_string()))?;

        Ok(tonic::Response::new(SearchResponse {
            albums: albums.into_iter().map(|(_, x)| x.into()).collect(),
            artists: artists.into_iter().map(|(_, x)| x.into()).collect(),
            tracks: tracks.into_iter().map(|(_, x)| x.into()).collect(),
        }))
    }
}
