use std::fmt::{Display, Formatter};
use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
use dashmap::DashMap;
use futures::stream::SplitStream;
use futures::{SinkExt, StreamExt};
use sqlx::{Pool, Postgres};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_util::codec::{Framed, LinesCodec, LinesCodecError};
use tracing::info;

const MAX_MESSAGES: usize = 128;
#[derive(Debug)]
struct AppChatState {
    chat_map: DashMap<SocketAddr, mpsc::Sender<Arc<Message>>>,
}
#[derive(Debug)]
enum Message {
    UserJoined(String),
    UserLeft(String),
    Chat { sender: String, content: String },
}

#[derive(Debug)]
struct Peer {
    username: String,
    stream: SplitStream<Framed<TcpStream, LinesCodec>>,
}

pub async fn chat_server() -> Result<()> {
    let app_state = Arc::new(AppChatState {
        chat_map: DashMap::new(),
    });

    let address = "127.0.0.1:8080";

    info!("listening on {}", address);
    let listener = tokio::net::TcpListener::bind(address).await?;

    loop {
        let (socket, address) = listener.accept().await.unwrap();
        info!("accepted connection from: {}", address);
        let app_state_clone = app_state.clone();

        tokio::spawn(async move {
            if let Err(e) = handler_client(socket, address, app_state_clone).await {
                println!("error: {}", e);
            }
        });
    }
}

async fn handler_client(
    socket: tokio::net::TcpStream,
    address: SocketAddr,
    app_state: Arc<AppChatState>,
) -> Result<()> {
    let mut stream = tokio_util::codec::Framed::new(socket, tokio_util::codec::LinesCodec::new());

    stream.send("send your name ").await?;
    let user_name = match stream.next().await {
        None => {
            return Ok(());
        }
        Some(Err(e)) => {
            return Err(e.into());
        }
        Some(Ok(user_name)) => user_name,
    };

    let mut peer = app_state.add(address, user_name.clone(), stream).await;

    let join_message = Arc::new(Message::UserJoined(user_name.clone()));

    info!("join {}", join_message.to_string());

    app_state.broadcast(address, join_message).await;

    while let Some(Ok(line)) = peer.stream.next().await {
        println!("{}", line);
        let message = Arc::new(Message::Chat {
            sender: user_name.clone(),
            content: line,
        });
        info!("{}", message.to_string());
        app_state.broadcast(address, message).await;
    }

    let left_message = Arc::new(Message::UserLeft(user_name.clone()));
    info!("{}", left_message.to_string());
    app_state.broadcast(address, left_message).await;

    Ok(())
}

impl AppChatState {
    async fn broadcast(&self, addr: SocketAddr, message: Arc<Message>) {
        for peer in self.chat_map.iter() {
            if peer.key() == &addr {
                continue;
            }

            if let Err(e) = peer.value().send(message.clone()).await {
                println!("error sending message: {}", e);
                self.chat_map.remove(&addr);
            }
        }
    }

    async fn add(
        &self,
        addr: SocketAddr,
        username: String,
        stream: Framed<TcpStream, LinesCodec>,
    ) -> Peer {
        let (tx, mut rx) = mpsc::channel(MAX_MESSAGES);

        self.chat_map.insert(addr, tx);

        let (mut sender, mut receiver) = stream.split();

        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                if let Err(e) = sender.send(message.to_string()).await {
                    println!("error sending message: {}", e);
                    break;
                }
            }
        });

        Peer {
            username,
            stream: receiver,
        }
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Message::UserJoined(content) => {
                write!(f, "join[{}]", content)
            }
            Message::UserLeft(content) => {
                write!(f, "left[{}]", content)
            }
            Message::Chat { sender, content } => {
                write!(f, "chat_server[{}]:{}", sender, content)
            }
        }
    }
}
