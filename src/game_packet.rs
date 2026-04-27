#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum GamePacket {
    JoinRequest,
    JoinResponse { assigned_id: u64 },
    PlayerPos { id: u64, x: f32, y: f32, dir: f32 },
    Action { id: u64, kind: String, dir: f32 },
    Leave { id: u64 },
}
