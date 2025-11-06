use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    http::StatusCode,
    response::Response,
    routing::get,
    Router,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::Arc,
};
use tokio::sync::{broadcast, RwLock};
use tower_http::cors::CorsLayer;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignalingMessage {
    /// Client wants to join a game room
    JoinRoom {
        room_id: String,
        variant: String,
        player_name: String,
    },
    /// Client is leaving a room
    LeaveRoom {
        room_id: String,
    },
    /// WebRTC offer
    Offer {
        room_id: String,
        target_player: String,
        sdp: String,
    },
    /// WebRTC answer
    Answer {
        room_id: String,
        target_player: String,
        sdp: String,
    },
    /// ICE candidate
    IceCandidate {
        room_id: String,
        target_player: String,
        candidate: String,
    },
    /// Game move
    GameMove {
        room_id: String,
        from: (i32, i32),
        to: (i32, i32),
    },
    /// Game state sync
    GameState {
        room_id: String,
        state: String,
    },
    /// Error message
    Error {
        message: String,
    },
    /// Success message
    Success {
        message: String,
    },
}

#[derive(Debug, Clone)]
pub struct Player {
    pub id: String,
    pub name: String,
    pub variant: String,
    pub sender: broadcast::Sender<SignalingMessage>,
}

#[derive(Debug, Clone)]
pub struct GameRoom {
    pub id: String,
    pub variant: String,
    pub players: HashMap<String, Player>,
    pub max_players: usize,
}

impl GameRoom {
    pub fn new(id: String, variant: String) -> Self {
        Self {
            id,
            variant,
            players: HashMap::new(),
            max_players: 2,
        }
    }

    pub fn add_player(&mut self, player: Player) -> Result<(), String> {
        if self.players.len() >= self.max_players {
            return Err("Room is full".to_string());
        }
        if self.players.contains_key(&player.id) {
            return Err("Player already in room".to_string());
        }
        self.players.insert(player.id.clone(), player);
        Ok(())
    }

    pub fn remove_player(&mut self, player_id: &str) {
        self.players.remove(player_id);
    }

    pub fn is_full(&self) -> bool {
        self.players.len() >= self.max_players
    }
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub rooms: Arc<RwLock<HashMap<String, GameRoom>>>,
    pub players: Arc<RwLock<HashMap<String, String>>>, // player_id -> room_id
}

impl AppState {
    pub fn new() -> Self {
        Self {
            rooms: Arc::new(RwLock::new(HashMap::new())),
            players: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app_state = AppState::new();

    let app = Router::new()
        .route("/", get(health_check))
        .route("/ws", get(websocket_handler))
        .route("/rooms", get(list_rooms))
        .route("/rooms/:room_id", get(get_room))
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
    tracing::info!("Signaling server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
    "Hex Chess Signaling Server is running"
}

async fn list_rooms(State(state): State<AppState>) -> Result<Response<String>, StatusCode> {
    let rooms = state.rooms.read().await;
    let room_list: Vec<_> = rooms
        .values()
        .map(|room| {
            serde_json::json!({
                "id": room.id,
                "variant": room.variant,
                "player_count": room.players.len(),
                "max_players": room.max_players,
                "is_full": room.is_full()
            })
        })
        .collect();

    let response = serde_json::to_string(&room_list)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json")
        .body(response)
        .unwrap())
}

async fn get_room(
    State(state): State<AppState>,
    axum::extract::Path(room_id): axum::extract::Path<String>,
) -> Result<Response<String>, StatusCode> {
    let rooms = state.rooms.read().await;
    let room = rooms
        .get(&room_id)
        .ok_or(StatusCode::NOT_FOUND)?;

    let room_info = serde_json::json!({
        "id": room.id,
        "variant": room.variant,
        "player_count": room.players.len(),
        "max_players": room.max_players,
        "is_full": room.is_full(),
        "players": room.players.values().map(|p| {
            serde_json::json!({
                "id": p.id,
                "name": p.name
            })
        }).collect::<Vec<_>>()
    });

    let response = serde_json::to_string(&room_info)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json")
        .body(response)
        .unwrap())
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(|socket| websocket_connection(socket, state))
}

async fn websocket_connection(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = broadcast::channel(100);
    let player_id = Uuid::new_v4().to_string();

    // Send messages from the broadcast channel to the WebSocket
    let tx_clone = tx.clone();
    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if let Ok(json) = serde_json::to_string(&msg) {
                if sender.send(Message::Text(json)).await.is_err() {
                    break;
                }
            }
        }
    });

    // Handle incoming messages
    while let Some(msg) = receiver.next().await {
        let msg = match msg {
            Ok(Message::Text(text)) => text,
            Ok(Message::Close(_)) => break,
            _ => continue,
        };

        if let Ok(signaling_msg) = serde_json::from_str::<SignalingMessage>(&msg) {
            if let Err(e) = handle_signaling_message(
                &state,
                &player_id,
                &tx_clone,
                signaling_msg,
            ).await {
                let error_msg = SignalingMessage::Error {
                    message: e.to_string(),
                };
                let _ = tx_clone.send(error_msg);
            }
        }
    }

    // Cleanup when connection closes
    cleanup_player(&state, &player_id).await;
    send_task.abort();
}

async fn handle_signaling_message(
    state: &AppState,
    player_id: &str,
    tx: &broadcast::Sender<SignalingMessage>,
    msg: SignalingMessage,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match msg {
        SignalingMessage::JoinRoom {
            room_id,
            variant,
            player_name,
        } => {
            let mut rooms = state.rooms.write().await;
            let mut players = state.players.write().await;

            // Remove player from any existing room
            if let Some(old_room_id) = players.get(player_id) {
                if let Some(room) = rooms.get_mut(old_room_id) {
                    room.remove_player(player_id);
                }
            }

            // Get or create room
            let room = rooms
                .entry(room_id.clone())
                .or_insert_with(|| GameRoom::new(room_id.clone(), variant.clone()));

            // Create player
            let player = Player {
                id: player_id.to_string(),
                name: player_name.clone(),
                variant: variant.clone(),
                sender: tx.clone(),
            };

            // Add player to room
            room.add_player(player)?;
            players.insert(player_id.to_string(), room_id.clone());

            // Notify other players in the room
            for (other_player_id, other_player) in &room.players {
                if other_player_id != player_id {
                    let join_msg = SignalingMessage::Success {
                        message: format!("Player {} joined the room", player_name),
                    };
                    let _ = other_player.sender.send(join_msg);
                }
            }

            // Send success message to joining player
            let success_msg = SignalingMessage::Success {
                message: "Successfully joined room".to_string(),
            };
            let _ = tx.send(success_msg);
        }

        SignalingMessage::LeaveRoom { room_id } => {
            let mut rooms = state.rooms.write().await;
            let mut players = state.players.write().await;

            if let Some(room) = rooms.get_mut(&room_id) {
                room.remove_player(player_id);
                
                // Notify other players
                for other_player in room.players.values() {
                    let leave_msg = SignalingMessage::Success {
                        message: "A player left the room".to_string(),
                    };
                    let _ = other_player.sender.send(leave_msg);
                }
            }

            players.remove(player_id);
        }

        SignalingMessage::Offer {
            room_id,
            target_player,
            sdp,
        } => {
            let rooms = state.rooms.read().await;
            if let Some(room) = rooms.get(&room_id) {
                if let Some(target) = room.players.get(&target_player) {
                    let offer_msg = SignalingMessage::Offer {
                        room_id,
                        target_player: player_id.to_string(),
                        sdp,
                    };
                    let _ = target.sender.send(offer_msg);
                }
            }
        }

        SignalingMessage::Answer {
            room_id,
            target_player,
            sdp,
        } => {
            let rooms = state.rooms.read().await;
            if let Some(room) = rooms.get(&room_id) {
                if let Some(target) = room.players.get(&target_player) {
                    let answer_msg = SignalingMessage::Answer {
                        room_id,
                        target_player: player_id.to_string(),
                        sdp,
                    };
                    let _ = target.sender.send(answer_msg);
                }
            }
        }

        SignalingMessage::IceCandidate {
            room_id,
            target_player,
            candidate,
        } => {
            let rooms = state.rooms.read().await;
            if let Some(room) = rooms.get(&room_id) {
                if let Some(target) = room.players.get(&target_player) {
                    let candidate_msg = SignalingMessage::IceCandidate {
                        room_id,
                        target_player: player_id.to_string(),
                        candidate,
                    };
                    let _ = target.sender.send(candidate_msg);
                }
            }
        }

        SignalingMessage::GameMove {
            room_id,
            from,
            to,
        } => {
            let rooms = state.rooms.read().await;
            if let Some(room) = rooms.get(&room_id) {
                for other_player in room.players.values() {
                    if other_player.id != player_id {
                        let move_msg = SignalingMessage::GameMove {
                            room_id: room_id.clone(),
                            from,
                            to,
                        };
                        let _ = other_player.sender.send(move_msg);
                    }
                }
            }
        }

        SignalingMessage::GameState { room_id, state: game_state } => {
            let rooms = state.rooms.read().await;
            if let Some(room) = rooms.get(&room_id) {
                for other_player in room.players.values() {
                    if other_player.id != player_id {
                        let state_msg = SignalingMessage::GameState {
                            room_id: room_id.clone(),
                            state: game_state.clone(),
                        };
                        let _ = other_player.sender.send(state_msg);
                    }
                }
            }
        }

        _ => {
            let error_msg = SignalingMessage::Error {
                message: "Unknown message type".to_string(),
            };
            let _ = tx.send(error_msg);
        }
    }

    Ok(())
}

async fn cleanup_player(state: &AppState, player_id: &str) {
    let mut rooms = state.rooms.write().await;
    let mut players = state.players.write().await;

    if let Some(room_id) = players.get(player_id) {
        if let Some(room) = rooms.get_mut(room_id) {
            room.remove_player(player_id);
            
            // Notify other players
            for other_player in room.players.values() {
                let leave_msg = SignalingMessage::Success {
                    message: "A player disconnected".to_string(),
                };
                let _ = other_player.sender.send(leave_msg);
            }
        }
    }

    players.remove(player_id);
}
