use crate::db::Db;
use innertube::InnerTube;
use player::Player;
use std::sync::Arc;

pub struct Orchestrator {
    db: Arc<Db>,
    #[allow(dead_code)]
    innertube: Arc<InnerTube>,
    player: Arc<Player>,
}

impl Orchestrator {
    pub fn new(
        db: Arc<Db>,
        innertube: Arc<InnerTube>,
        player: Arc<Player>,
    ) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            db,
            innertube,
            player,
        })
    }

    pub async fn get_queue(
        &self,
    ) -> std::result::Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
        let queue = self.db.get_queue().await?;
        Ok(queue)
    }

    pub async fn get_playback_state(
        &self,
    ) -> std::result::Result<serde_json::Value, Box<dyn std::error::Error>> {
        let state = self.player.get_playback_state();
        Ok(state)
    }
}

pub fn is_youtube_stream(url: &str) -> bool {
    url.contains("googlevideo.com") || url.contains("youtube.com")
}
