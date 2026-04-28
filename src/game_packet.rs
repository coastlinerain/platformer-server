#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum GamePacket {
    JoinRequest,
    JoinResponse {
        assigned_id: u8,
    },
    PlayerPos {
        id: u8,
        x: f32,
        y: f32,
        dir: f32,
        level_x: usize,
        level_y: usize,
    },
    Action {
        id: u8,
        kind: String,
        dir: f32,
    },
    Leave {
        id: u8,
    },
    Hit {
        id: u8,
    },
}
