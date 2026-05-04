use rockbox_library::repo;
use sqlx::{Pool, Sqlite};

use crate::api::rockbox::v1alpha1::{
    genre_service_server::GenreService, Genre as ProtoGenre, GetGenreAlbumsRequest,
    GetGenreAlbumsResponse, GetGenreArtistsRequest, GetGenreArtistsResponse, GetGenreRequest,
    GetGenreResponse, GetGenreTracksRequest, GetGenreTracksResponse, GetGenresRequest,
    GetGenresResponse,
};

pub struct Genre {
    pool: Pool<Sqlite>,
}

impl Genre {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }
}

#[tonic::async_trait]
impl GenreService for Genre {
    async fn get_genres(
        &self,
        _request: tonic::Request<GetGenresRequest>,
    ) -> Result<tonic::Response<GetGenresResponse>, tonic::Status> {
        let genres = repo::genre::all(self.pool.clone())
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        let mut out: Vec<ProtoGenre> = Vec::with_capacity(genres.len());
        for g in genres {
            let track_count = repo::genre::find_tracks(self.pool.clone(), &g.id)
                .await
                .map(|t| t.len() as i64)
                .unwrap_or(0);
            out.push(ProtoGenre {
                id: g.id,
                name: g.name,
                description: g.description,
                image: g.image,
                tracks: vec![],
                albums: vec![],
                artists: vec![],
                track_count,
            });
        }

        Ok(tonic::Response::new(GetGenresResponse { genres: out }))
    }

    async fn get_genre(
        &self,
        request: tonic::Request<GetGenreRequest>,
    ) -> Result<tonic::Response<GetGenreResponse>, tonic::Status> {
        let id = request.into_inner().id;
        let genre = repo::genre::find(self.pool.clone(), &id)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        let genre = match genre {
            Some(g) => g,
            None => return Ok(tonic::Response::new(GetGenreResponse { genre: None })),
        };

        let tracks = repo::genre::find_tracks(self.pool.clone(), &id)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        let albums = repo::genre::find_albums(self.pool.clone(), &id)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        let artists = repo::genre::find_artists(self.pool.clone(), &id)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        let track_count = tracks.len() as i64;
        Ok(tonic::Response::new(GetGenreResponse {
            genre: Some(ProtoGenre {
                id: genre.id,
                name: genre.name,
                description: genre.description,
                image: genre.image,
                tracks: tracks.into_iter().map(Into::into).collect(),
                albums: albums.into_iter().map(Into::into).collect(),
                artists: artists.into_iter().map(Into::into).collect(),
                track_count,
            }),
        }))
    }

    async fn get_genre_tracks(
        &self,
        request: tonic::Request<GetGenreTracksRequest>,
    ) -> Result<tonic::Response<GetGenreTracksResponse>, tonic::Status> {
        let id = request.into_inner().id;
        let tracks = repo::genre::find_tracks(self.pool.clone(), &id)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(GetGenreTracksResponse {
            tracks: tracks.into_iter().map(Into::into).collect(),
        }))
    }

    async fn get_genre_albums(
        &self,
        request: tonic::Request<GetGenreAlbumsRequest>,
    ) -> Result<tonic::Response<GetGenreAlbumsResponse>, tonic::Status> {
        let id = request.into_inner().id;
        let albums = repo::genre::find_albums(self.pool.clone(), &id)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(GetGenreAlbumsResponse {
            albums: albums.into_iter().map(Into::into).collect(),
        }))
    }

    async fn get_genre_artists(
        &self,
        request: tonic::Request<GetGenreArtistsRequest>,
    ) -> Result<tonic::Response<GetGenreArtistsResponse>, tonic::Status> {
        let id = request.into_inner().id;
        let artists = repo::genre::find_artists(self.pool.clone(), &id)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(GetGenreArtistsResponse {
            artists: artists.into_iter().map(Into::into).collect(),
        }))
    }
}
