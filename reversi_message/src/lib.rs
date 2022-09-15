use reversi_core::{Position, ReversiData};
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
pub enum ClientMessage {
    SessionList,
    CreateGame,
    SelectGame(GameID),
    Put(Position),
    Reset,
}

#[derive(Serialize, Deserialize, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct GameID(pub u32);

#[derive(Serialize, Deserialize, Clone)]
pub struct GameSummary {
    pub id: GameID,
    pub members: u32,
    pub your: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum ServerMessage {
    GameList(Vec<GameSummary>),
    View(ReversiData),
}
