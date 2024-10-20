use async_graphql::*;

use crate::rockbox_url;

#[derive(Default)]
pub struct SoundQuery;

#[Object]
impl SoundQuery {
    async fn sound_current(&self) -> String {
        "sound".to_string()
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
        let body = serde_json::json!({
            "steps": steps,
        });
        let url = format!("{}/player/volume", rockbox_url());
        client.put(&url).json(&body).send().await?;

        Ok(0)
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
