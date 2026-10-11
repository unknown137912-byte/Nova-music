//! Listen Together protocol types and client

use serde::{Deserialize, Serialize};
use futures_util::{SinkExt, StreamExt};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub id: String,
    pub name: String,
    pub host: User,
    pub users: Vec<User>,
    pub current_track: Option<Track>,
    pub queue: Vec<Track>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub name: String,
    pub avatar: Option<String>,
    pub is_host: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub thumbnail: Option<String>,
    pub duration: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Message {
    #[serde(rename = "join")]
    Join { room_id: String, user: User },
    #[serde(rename = "leave")]
    Leave { room_id: String, user_id: String },
    #[serde(rename = "play")]
    Play { room_id: String, track: Track, position: f64 },
    #[serde(rename = "pause")]
    Pause { room_id: String, position: f64 },
    #[serde(rename = "seek")]
    Seek { room_id: String, position: f64 },
    #[serde(rename = "queue")]
    Queue { room_id: String, tracks: Vec<Track> },
    #[serde(rename = "chat")]
    Chat { room_id: String, user: User, message: String },
}

pub struct ListenClient {
    room_id: Option<String>,
    user: User,
    ws: Option<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>,
}

impl ListenClient {
    pub fn new(user: User) -> Self {
        Self {
            room_id: None,
            user,
            ws: None,
        }
    }

    pub async fn connect(&mut self, url: &str) -> Result<(), Box<dyn std::error::Error>> {
        let (ws_stream, _) = tokio_tungstenite::connect_async(url).await?;
        self.ws = Some(ws_stream);
        Ok(())
    }

    pub async fn join_room(&mut self, room_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let message = Message::Join {
            room_id: room_id.to_string(),
            user: self.user.clone(),
        };
        self.send_message(message).await?;
        self.room_id = Some(room_id.to_string());
        Ok(())
    }

    pub async fn send_message(&mut self, message: Message) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ws) = &mut self.ws {
            let json = serde_json::to_string(&message)?;
            ws.send(tokio_tungstenite::tungstenite::Message::Text(json)).await?;
        }
        Ok(())
    }

    pub async fn receive_message(&mut self) -> Result<Option<Message>, Box<dyn std::error::Error>> {
        if let Some(ws) = &mut self.ws {
            while let Some(msg) = ws.next().await {
                let msg = msg?;
                if let tokio_tungstenite::tungstenite::Message::Text(text) = msg {
                    let message: Message = serde_json::from_str(&text)?;
                    return Ok(Some(message));
                }
            }
        }
        Ok(None)
    }
}