use reversi_core::*;
use reversi_message::*;
use std::{
    collections::HashMap,
    io::Error as IoError,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use futures_channel::mpsc::{unbounded, UnboundedSender};
use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};

use tokio::net::{TcpListener, TcpStream};
use tungstenite::protocol::Message;

type Tx = UnboundedSender<Message>;

fn send_message(tx: &Tx, message: &ServerMessage) {
    tx.unbounded_send(Message::Text(serde_json::to_string(&*message).unwrap()))
        .unwrap();
}

struct ServerData {
    connections: HashMap<SocketAddr, ConnectionData>,
    games: HashMap<GameID, ReversiData>,
}

impl ServerData {
    fn new() -> Self {
        return ServerData {
            connections: HashMap::new(),
            games: HashMap::new(),
        };
    }
    fn add_connectin(&mut self, addr: SocketAddr, connection: ConnectionData) {
        self.connections.insert(addr, connection);
    }
}

struct ConnectionData {
    game_id: Option<GameID>,
    sender: Tx,
}

fn get_game_summary_list(server_data: &ServerData, addr: &SocketAddr) -> Vec<GameSummary> {
    let mut gs = server_data
        .games
        .iter()
        .map(|(k, _)| GameSummary {
            id: *k,
            members: server_data
                .connections
                .iter()
                .filter(|(_, v)| v.game_id == Some(*k))
                .count() as u32,
            your: server_data.connections.get(addr).unwrap().game_id == Some(*k),
        })
        .collect::<Vec<GameSummary>>();
    gs.sort_by_key(|k| k.id);
    gs
}

fn add_new_game(games: &mut HashMap<GameID, ReversiData>) -> Result<GameID, ()> {
    for i in 1..u32::MAX {
        if let None = games.get(&(GameID(i))) {
            games.insert(GameID(i), ReversiData::new());
            return Ok(GameID(i));
        }
    }
    return Err(());
}

fn handle_sessionlist(addr: SocketAddr, server_data: &ServerData) {
    send_message(
        &server_data.connections.get(&addr).unwrap().sender,
        &ServerMessage::GameList(get_game_summary_list(server_data, &addr)),
    );
}

fn handle_creategame(addr: SocketAddr, server_data: &mut ServerData) {
    let new_game_id = add_new_game(&mut server_data.games).unwrap();
    server_data.connections.get_mut(&addr).unwrap().game_id = Some(new_game_id);
    send_message(
        &server_data.connections.get(&addr).unwrap().sender,
        &ServerMessage::View((server_data.games.get(&new_game_id).unwrap()).to_owned()),
    );
    server_data.connections.iter().for_each(|(addr, conn)| {
        send_message(
            &conn.sender,
            &ServerMessage::GameList(get_game_summary_list(server_data, &addr)),
        )
    });
}

fn handle_selectgame(addr: SocketAddr, server_data: &mut ServerData, game_id: GameID) {
    server_data.connections.get_mut(&addr).unwrap().game_id = Some(game_id);
    send_message(
        &server_data.connections.get(&addr).unwrap().sender,
        &ServerMessage::View((*server_data.games.get(&game_id).unwrap()).to_owned()),
    );
    server_data.connections.iter().for_each(|(addr, conn)| {
        send_message(
            &conn.sender,
            &ServerMessage::GameList(get_game_summary_list(server_data, &addr)),
        )
    });
}

fn handle_put(addr: SocketAddr, server_data: &mut ServerData, position: Position) {
    let gameid = server_data.connections.get(&addr).unwrap().game_id.unwrap();
    server_data.games.get_mut(&gameid).unwrap().cursor = position;
    try_put(server_data.games.get_mut(&gameid).unwrap());
    server_data
        .connections
        .iter()
        .filter(|&(_, v)| v.game_id == Some(gameid))
        .for_each(|(_, v)| {
            send_message(
                &v.sender,
                &ServerMessage::View((*server_data.games.get(&gameid).unwrap()).to_owned()),
            );
        });
}

fn handle_reset(addr: SocketAddr, server_data: &mut ServerData) {
    let gameid = match server_data.connections.get(&addr).unwrap().game_id {
        Some(x) => x,
        None => return,
    };
    *server_data.games.get_mut(&gameid).unwrap() = ReversiData::new();
    server_data
        .connections
        .iter()
        .filter(|&(_, v)| v.game_id == Some(gameid))
        .for_each(|(_, v)| {
            send_message(
                &v.sender,
                &ServerMessage::View((*server_data.games.get(&gameid).unwrap()).to_owned()),
            );
        });
}

fn handle_message(addr: SocketAddr, server_data: &mut ServerData, message: ClientMessage) {
    match message {
        ClientMessage::SessionList => handle_sessionlist(addr, server_data),
        ClientMessage::CreateGame => handle_creategame(addr, server_data),
        ClientMessage::SelectGame(x) => handle_selectgame(addr, server_data, x),
        ClientMessage::Put(x) => handle_put(addr, server_data, x),
        ClientMessage::Reset => handle_reset(addr, server_data),
    }
}

async fn handle_connection(
    raw_stream: TcpStream,
    addr: SocketAddr,
    server_data: Arc<Mutex<ServerData>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (outgoing, incoming) = tokio_tungstenite::accept_async(raw_stream).await?.split();
    println!("WebSocket connection established: {}", addr);
    let (tx, rx) = unbounded();
    let connection = ConnectionData {
        game_id: None,
        sender: tx,
    };
    server_data.lock().unwrap().add_connectin(addr, connection);
    let broadcast_incoming = incoming.try_for_each(|msg| async {
        let client_message = serde_json::from_str(&msg.into_text()?)
            .map_err(|_| tungstenite::Error::ConnectionClosed)?;
        println!("client message reveive");
        handle_message(addr, &mut *server_data.lock().unwrap(), client_message);
        Ok(())
    });
    let receive_from_others = rx.map(Ok).forward(outgoing);
    pin_mut!(broadcast_incoming, receive_from_others);
    future::select(broadcast_incoming, receive_from_others).await;
    println!("{} disconnected", &addr);
    server_data.lock().unwrap().connections.remove(&addr);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), IoError> {
    let server_data = Arc::new(Mutex::new(ServerData::new()));
    while let Ok((stream, addr)) = TcpListener::bind(&"127.0.0.1:9001".to_string())
        .await?
        .accept()
        .await
    {
        tokio::spawn(handle_connection(stream, addr, server_data.clone()));
    }
    Ok(())
}
