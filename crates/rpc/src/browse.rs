use crate::api::rockbox::v1alpha1::{browse_service_server::BrowseService, *};

#[derive(Default)]
pub struct Browse;

#[tonic::async_trait]
impl BrowseService for Browse {
    async fn rockbox_browse(
        &self,
        request: tonic::Request<RockboxBrowseRequest>,
    ) -> Result<tonic::Response<RockboxBrowseResponse>, tonic::Status> {
        Ok(tonic::Response::new(RockboxBrowseResponse::default()))
    }

    async fn tree_get_context(
        &self,
        request: tonic::Request<TreeGetContextRequest>,
    ) -> Result<tonic::Response<TreeGetContextResponse>, tonic::Status> {
        Ok(tonic::Response::new(TreeGetContextResponse::default()))
    }

    async fn tree_get_entries(
        &self,
        request: tonic::Request<TreeGetEntriesRequest>,
    ) -> Result<tonic::Response<TreeGetEntriesResponse>, tonic::Status> {
        Ok(tonic::Response::new(TreeGetEntriesResponse::default()))
    }

    async fn tree_get_entry_at(
        &self,
        request: tonic::Request<TreeGetEntryAtRequest>,
    ) -> Result<tonic::Response<TreeGetEntryAtResponse>, tonic::Status> {
        Ok(tonic::Response::new(TreeGetEntryAtResponse::default()))
    }

    async fn browse_id3(
        &self,
        request: tonic::Request<BrowseId3Request>,
    ) -> Result<tonic::Response<BrowseId3Response>, tonic::Status> {
        Ok(tonic::Response::new(BrowseId3Response::default()))
    }
}
