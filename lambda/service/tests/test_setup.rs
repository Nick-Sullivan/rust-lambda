#[cfg(test)]
use std::env;

#[cfg(test)]
pub fn setup() {
    env::set_var("WEBSOCKET_TABLE_NAME", "WEBSOCKET");
    env::set_var("GAME_TABLE_NAME", "GAME");
}
