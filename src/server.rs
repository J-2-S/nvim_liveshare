use crate::config::Config;
use anyhow::{Result, anyhow};
use std::net::ToSocketAddrs;
use std::sync::Arc;
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::TcpListener,
    sync::{Mutex, OnceCell, broadcast, mpsc},
    task,
    time::{self, Duration},
};

type PlaceHolder = String;
static CLIENT_CHANGES: OnceCell<Arc<Mutex<mpsc::Receiver<PlaceHolder>>>> = OnceCell::const_new(); // String is a
// place holder for
// our data type
pub async fn start_server(config: &Config) -> Result<()> {
    let addrs = (config.hostname.clone(), config.port)
        .to_socket_addrs()?
        .next()
        .ok_or(anyhow!("Invaild socket addrs"))?;
    let server = TcpListener::bind(addrs).await?;
    let (tx, mut rx) = mpsc::channel::<PlaceHolder>(10);
    CLIENT_CHANGES.set(Arc::new(Mutex::new(rx)))?;
    loop {
        let (mut stream, client_addrs) = server.accept().await?;
        println!("Received connection from {}", client_addrs);
        let sender = tx.clone();
        task::spawn(async move {
            client_handles(&mut stream, sender).await;
        });
    }
    todo!()
}

async fn client_handles<S>(socket: &mut S, sender: mpsc::Sender<PlaceHolder>)
where
    S: Send + AsyncReadExt + AsyncWriteExt + Unpin + AsyncWrite + AsyncRead,
{
    let mut buff = vec![];
    loop {
        tokio::select! {
            Ok(data) = socket.read(&mut buff) => {
                sender.send(todo!("buff")).await;

            }, //Add jacobs Receiver to the listen here as well
            _ = tokio::time::sleep(Duration::from_secs(10))=>{}
        }
    }
}
pub fn get_incoming_stream() -> Option<&'static Arc<Mutex<mpsc::Receiver<PlaceHolder>>>> {
    return CLIENT_CHANGES.get();
}
