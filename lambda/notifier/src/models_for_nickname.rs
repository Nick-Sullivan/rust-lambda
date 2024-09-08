use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct SetNicknameMessage {
    pub nickname: String,
    #[serde(rename = "playerId")]
    pub player_id: String,
}
