use crate::{
    api::rockbox::v1alpha1::{browse_service_server::BrowseService, *},
    rockbox_url,
};
use rockbox_sys as rb;

#[derive(Default)]
pub struct Browse {
    client: reqwest::Client,
}

impl Browse {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }
}

#[tonic::async_trait]
impl BrowseService for Browse {
    async fn tree_get_entries(
        &self,
        request: tonic::Request<TreeGetEntriesRequest>,
    ) -> Result<tonic::Response<TreeGetEntriesResponse>, tonic::Status> {
        let path = request.into_inner().path;
        let url = format!("{}/tree_entries?q={}", rockbox_url(), path);
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        let data = response
            .json::<Vec<rb::types::tree::Entry>>()
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        let entries = data
            .into_iter()
            .map(|entry| Entry::from(entry))
            .collect::<Vec<Entry>>();
        Ok(tonic::Response::new(TreeGetEntriesResponse { entries }))
    }
}
