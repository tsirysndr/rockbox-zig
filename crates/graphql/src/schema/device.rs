use async_graphql::*;

use crate::rockbox_url;

use super::objects::device::Device;

#[derive(Default)]
pub struct DeviceQuery;

#[Object]
impl DeviceQuery {
    async fn devices(&self, _ctx: &Context<'_>) -> Result<Vec<Device>, Error> {
        let client = reqwest::Client::new();
        let url = format!("{}/devices", rockbox_url());
        let response = client.get(&url).send().await?;
        let response = response.json::<Vec<Device>>().await?;
        Ok(response)
    }

    async fn device(&self, _ctx: &Context<'_>, id: String) -> Result<Option<Device>, Error> {
        let client = reqwest::Client::new();
        let url = format!("{}/devices/{}", rockbox_url(), id);
        let response = client.get(&url).send().await?;

        if response.status() == 404 {
            return Ok(None);
        }

        let response = response.json::<Option<Device>>().await?;
        Ok(response)
    }
}

#[derive(Default)]
pub struct DeviceMutation;

#[Object]
impl DeviceMutation {
    async fn connect(&self, _ctx: &Context<'_>, id: String) -> Result<bool, Error> {
        let client = reqwest::Client::new();
        let url = format!("{}/devices/{}/connect", rockbox_url(), id);
        client.put(&url).send().await?;
        Ok(true)
    }

    async fn disconnect(&self, _ctx: &Context<'_>, id: String) -> Result<bool, Error> {
        let client = reqwest::Client::new();
        let url = format!("{}/devices/{}/disconnect", rockbox_url(), id);
        client.put(&url).send().await?;
        Ok(true)
    }
}
