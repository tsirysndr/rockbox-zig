use rockbox_playlists::{Playlist, PlaylistFolder, PlaylistStore};

use crate::api::rockbox::v1alpha1::{
    saved_playlist_service_server::SavedPlaylistService, AddTracksToSavedPlaylistRequest,
    AddTracksToSavedPlaylistResponse, CreatePlaylistFolderRequest, CreatePlaylistFolderResponse,
    CreateSavedPlaylistRequest, CreateSavedPlaylistResponse, DeletePlaylistFolderRequest,
    DeletePlaylistFolderResponse, DeleteSavedPlaylistRequest, DeleteSavedPlaylistResponse,
    GetPlaylistFoldersRequest, GetPlaylistFoldersResponse, GetSavedPlaylistRequest,
    GetSavedPlaylistResponse, GetSavedPlaylistTracksRequest, GetSavedPlaylistTracksResponse,
    GetSavedPlaylistsRequest, GetSavedPlaylistsResponse, PlaySavedPlaylistRequest,
    PlaySavedPlaylistResponse, PlaylistFolder as ProtoFolder, RemoveTrackFromSavedPlaylistRequest,
    RemoveTrackFromSavedPlaylistResponse, SavedPlaylist as ProtoPlaylist,
    UpdateSavedPlaylistRequest, UpdateSavedPlaylistResponse,
};
use crate::rockbox_url;

pub struct SavedPlaylist {
    store: PlaylistStore,
    client: reqwest::Client,
}

impl SavedPlaylist {
    pub fn new(store: PlaylistStore, client: reqwest::Client) -> Self {
        Self { store, client }
    }
}

fn to_proto_folder(f: PlaylistFolder) -> ProtoFolder {
    ProtoFolder {
        id: f.id,
        name: f.name,
        created_at: f.created_at,
        updated_at: f.updated_at,
    }
}

fn to_proto_playlist(p: Playlist) -> ProtoPlaylist {
    ProtoPlaylist {
        id: p.id,
        name: p.name,
        description: p.description,
        image: p.image,
        folder_id: p.folder_id,
        track_count: p.track_count,
        created_at: p.created_at,
        updated_at: p.updated_at,
    }
}

#[tonic::async_trait]
impl SavedPlaylistService for SavedPlaylist {
    async fn create_playlist_folder(
        &self,
        request: tonic::Request<CreatePlaylistFolderRequest>,
    ) -> Result<tonic::Response<CreatePlaylistFolderResponse>, tonic::Status> {
        let name = request.into_inner().name;
        let folder = self
            .store
            .create_folder(&name)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(CreatePlaylistFolderResponse {
            folder: Some(to_proto_folder(folder)),
        }))
    }

    async fn get_playlist_folders(
        &self,
        _request: tonic::Request<GetPlaylistFoldersRequest>,
    ) -> Result<tonic::Response<GetPlaylistFoldersResponse>, tonic::Status> {
        let folders = self
            .store
            .list_folders()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(GetPlaylistFoldersResponse {
            folders: folders.into_iter().map(to_proto_folder).collect(),
        }))
    }

    async fn delete_playlist_folder(
        &self,
        request: tonic::Request<DeletePlaylistFolderRequest>,
    ) -> Result<tonic::Response<DeletePlaylistFolderResponse>, tonic::Status> {
        let id = request.into_inner().id;
        self.store
            .delete_folder(&id)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(DeletePlaylistFolderResponse {}))
    }

    async fn get_saved_playlists(
        &self,
        request: tonic::Request<GetSavedPlaylistsRequest>,
    ) -> Result<tonic::Response<GetSavedPlaylistsResponse>, tonic::Status> {
        let folder_id = request.into_inner().folder_id;
        let playlists = if let Some(fid) = folder_id.as_deref().filter(|s| !s.is_empty()) {
            self.store
                .list_by_folder(fid)
                .await
                .map_err(|e| tonic::Status::internal(e.to_string()))?
        } else {
            self.store
                .list()
                .await
                .map_err(|e| tonic::Status::internal(e.to_string()))?
        };
        Ok(tonic::Response::new(GetSavedPlaylistsResponse {
            playlists: playlists.into_iter().map(to_proto_playlist).collect(),
        }))
    }

    async fn get_saved_playlist(
        &self,
        request: tonic::Request<GetSavedPlaylistRequest>,
    ) -> Result<tonic::Response<GetSavedPlaylistResponse>, tonic::Status> {
        let id = request.into_inner().id;
        let playlist = self
            .store
            .get(&id)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(GetSavedPlaylistResponse {
            playlist: playlist.map(to_proto_playlist),
        }))
    }

    async fn create_saved_playlist(
        &self,
        request: tonic::Request<CreateSavedPlaylistRequest>,
    ) -> Result<tonic::Response<CreateSavedPlaylistResponse>, tonic::Status> {
        let req = request.into_inner();
        let playlist = self
            .store
            .create(
                &req.name,
                req.description.as_deref(),
                req.image.as_deref(),
                req.folder_id.as_deref(),
            )
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        if !req.track_ids.is_empty() {
            self.store
                .add_tracks(&playlist.id, &req.track_ids)
                .await
                .map_err(|e| tonic::Status::internal(e.to_string()))?;
        }
        Ok(tonic::Response::new(CreateSavedPlaylistResponse {
            playlist: Some(to_proto_playlist(playlist)),
        }))
    }

    async fn update_saved_playlist(
        &self,
        request: tonic::Request<UpdateSavedPlaylistRequest>,
    ) -> Result<tonic::Response<UpdateSavedPlaylistResponse>, tonic::Status> {
        let req = request.into_inner();
        self.store
            .update(
                &req.id,
                &req.name,
                req.description.as_deref(),
                req.image.as_deref(),
                req.folder_id.as_deref(),
            )
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(UpdateSavedPlaylistResponse {}))
    }

    async fn delete_saved_playlist(
        &self,
        request: tonic::Request<DeleteSavedPlaylistRequest>,
    ) -> Result<tonic::Response<DeleteSavedPlaylistResponse>, tonic::Status> {
        let id = request.into_inner().id;
        self.store
            .delete(&id)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(DeleteSavedPlaylistResponse {}))
    }

    async fn get_saved_playlist_tracks(
        &self,
        request: tonic::Request<GetSavedPlaylistTracksRequest>,
    ) -> Result<tonic::Response<GetSavedPlaylistTracksResponse>, tonic::Status> {
        let playlist_id = request.into_inner().playlist_id;
        let track_ids = self
            .store
            .get_track_ids(&playlist_id)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(GetSavedPlaylistTracksResponse {
            track_ids,
        }))
    }

    async fn add_tracks_to_saved_playlist(
        &self,
        request: tonic::Request<AddTracksToSavedPlaylistRequest>,
    ) -> Result<tonic::Response<AddTracksToSavedPlaylistResponse>, tonic::Status> {
        let req = request.into_inner();
        self.store
            .add_tracks(&req.playlist_id, &req.track_ids)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(AddTracksToSavedPlaylistResponse {}))
    }

    async fn remove_track_from_saved_playlist(
        &self,
        request: tonic::Request<RemoveTrackFromSavedPlaylistRequest>,
    ) -> Result<tonic::Response<RemoveTrackFromSavedPlaylistResponse>, tonic::Status> {
        let req = request.into_inner();
        self.store
            .remove_track(&req.playlist_id, &req.track_id)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(
            RemoveTrackFromSavedPlaylistResponse {},
        ))
    }

    async fn play_saved_playlist(
        &self,
        request: tonic::Request<PlaySavedPlaylistRequest>,
    ) -> Result<tonic::Response<PlaySavedPlaylistResponse>, tonic::Status> {
        let playlist_id = request.into_inner().playlist_id;
        let url = format!("{}/saved-playlists/{}/play", rockbox_url(), playlist_id);
        self.client
            .post(&url)
            .send()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(PlaySavedPlaylistResponse {}))
    }
}
