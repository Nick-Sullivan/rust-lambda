use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct RoundStateMessage {
    pub complete: bool,
}

#[derive(Serialize, Debug)]
pub struct SpectatorStateMessage {
    pub id: String,
    pub nickname: String,
}

#[derive(Serialize, Debug)]
pub struct PlayerStateMessage {
    pub id: String,
    pub nickname: String,
    #[serde(rename = "turnFinished")]
    pub turn_finished: bool,
    #[serde(rename = "winCount")]
    pub win_count: i32,
    #[serde(rename = "rollResult")]
    pub roll_result: String,
    #[serde(rename = "connectionStatus")]
    pub connection_status: String,
    #[serde(rename = "rollTotal")]
    pub roll_total: i32,
    #[serde(rename = "diceValue")]
    pub dice_value: String,
}

#[derive(Serialize, Debug)]
pub struct GameStateMessage {
    #[serde(rename = "gameId")]
    pub game_id: String,
    // pub players: Vec<PlayerStateMessage>,
    // pub spectators: Vec<SpectatorStateMessage>,
    pub round: RoundStateMessage,
}
