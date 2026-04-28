use async_graphql::*;

use crate::rockbox_url;

#[derive(SimpleObject)]
struct VolumeInfo {
    volume: i32,
    min: i32,
    max: i32,
}

#[derive(Default)]
pub struct SoundQuery;

#[Object]
impl SoundQuery {
    async fn volume(&self, ctx: &Context<'_>) -> Result<VolumeInfo, Error> {
        let client = ctx.data::<reqwest::Client>().unwrap();
        let url = format!("{}/player/volume", rockbox_url());
        let resp = client.get(&url).send().await?;
        let body: serde_json::Value = resp.json().await?;
        Ok(VolumeInfo {
            volume: body["volume"].as_i64().unwrap_or(0) as i32,
            min: body["min"].as_i64().unwrap_or(-80) as i32,
            max: body["max"].as_i64().unwrap_or(0) as i32,
        })
    }

    async fn sound_default(&self) -> String {
        "sound default".to_string()
    }

    async fn sound_val_2_phys(&self) -> String {
        "sound val 2 phys".to_string()
    }

    async fn get_pitch(&self) -> String {
        "get pitch".to_string()
    }
}

#[derive(Default)]
pub struct SoundMutation;

#[Object]
impl SoundMutation {
    async fn adjust_volume(&self, ctx: &Context<'_>, steps: i32) -> Result<i32, Error> {
        let client = ctx.data::<reqwest::Client>().unwrap();
        let body = serde_json::json!({ "steps": steps });
        let url = format!("{}/player/volume", rockbox_url());
        client.put(&url).json(&body).send().await?;

        let resp = client.get(&url).send().await?;
        let info: serde_json::Value = resp.json().await?;
        Ok(info["volume"].as_i64().unwrap_or(0) as i32)
    }

    async fn sound_set(&self) -> String {
        "sound set".to_string()
    }

    async fn sound_min(&self) -> String {
        "sound min".to_string()
    }

    async fn sound_max(&self) -> String {
        "sound max".to_string()
    }

    async fn sound_unit(&self) -> String {
        "sound unit".to_string()
    }

    async fn set_pitch(&self) -> String {
        "set pitch".to_string()
    }

    async fn beep_play(&self) -> String {
        "beep play".to_string()
    }

    async fn pcmbuf_play(&self) -> String {
        "pcmbuf play".to_string()
    }

    async fn pcmbuf_fade(&self) -> String {
        "pcmbuf fade".to_string()
    }

    async fn pcmbuf_set_low_latency(&self) -> String {
        "pcmbuf set low latency".to_string()
    }

    async fn system_sound_play(&self) -> String {
        "system sound play".to_string()
    }

    async fn keyclick_click(&self) -> String {
        "keyclick click".to_string()
    }
}
