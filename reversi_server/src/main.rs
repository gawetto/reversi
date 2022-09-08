use reversi_core::*;
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

fn send_reversi_data(tx: &Tx, reversi_data: &ReversiData) {
    tx.unbounded_send(Message::Text(
        serde_json::to_string(&*reversi_data).unwrap(),
    ))
    .unwrap();
}

async fn handle_connection(
    senders: Arc<Mutex<HashMap<SocketAddr, Tx>>>,
    raw_stream: TcpStream,
    addr: SocketAddr,
    reversi_data: Arc<Mutex<ReversiData>>,
) {
    //let ws_stream = tokio_tungstenite::accept_async(raw_stream)
    //    .await
    //    .expect("Error during the websocket handshake occurred");
    let (outgoing, incoming) = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Error during the websocket handshake occurred")
        .split();
    println!("WebSocket connection established: {}", addr);

    let (tx, rx) = unbounded();
    send_reversi_data(&tx, &*reversi_data.lock().unwrap());
    senders.lock().unwrap().insert(addr, tx);

    //let (outgoing, incoming) = ws_stream.split();
    let broadcast_incoming = incoming.try_for_each(|msg| {
        println!(
            "Received a message from {}: {}",
            addr,
            msg.to_text().unwrap()
        );
        let mut reversi_data = reversi_data.lock().unwrap();
        let cursor;
        match serde_json::from_str(msg.to_text().unwrap()) {
            Ok(x) => cursor = x,
            Err(_) => return future::ok(()),
        }
        reversi_data.cursor = cursor;
        let turn = reversi_data.turn;
        if check_putable(&reversi_data.field, reversi_data.cursor, reversi_data.turn) {
            reversi_data.field.set(cursor, Masu::Putted(turn));
            auto_reverse(&mut reversi_data.field, cursor, turn);
            reversi_data.turn = get_another_color(reversi_data.turn);
            if !reversi_data.field.puttable(reversi_data.turn) {
                reversi_data.turn = get_another_color(reversi_data.turn);
            }
        }
        senders
            .lock()
            .unwrap()
            .iter()
            .map(|(_, ws_sink)| ws_sink)
            .for_each(|x| send_reversi_data(x, &*reversi_data));
        future::ok(())
    });

    let receive_from_others = rx.map(Ok).forward(outgoing);

    pin_mut!(broadcast_incoming, receive_from_others);

    future::select(broadcast_incoming, receive_from_others).await;

    println!("{} disconnected", &addr);
    senders.lock().unwrap().remove(&addr);
}

#[tokio::main]
async fn main() -> Result<(), IoError> {
    let reversi_data = Arc::new(Mutex::new(ReversiData::new()));
    let senders = Arc::new(Mutex::new(HashMap::new()));
    while let Ok((stream, addr)) = TcpListener::bind(&"127.0.0.1:9001".to_string())
        .await
        .expect("Failed to bind")
        .accept()
        .await
    {
        tokio::spawn(handle_connection(
            senders.clone(),
            stream,
            addr,
            reversi_data.clone(),
        ));
    }
    Ok(())
}
