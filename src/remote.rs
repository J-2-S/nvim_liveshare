use crate::{CWD, types::*};
use crate::{local::get_subscriber, types::Config};
use anyhow::{Result, anyhow};
use std::net::ToSocketAddrs;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::fs::{self, File};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::TcpListener,
    sync::{Mutex, OnceCell, mpsc, watch},
    task,
};
type PlaceHolder = Message;
static CHANGES: OnceCell<Arc<Mutex<mpsc::Receiver<PlaceHolder>>>> = OnceCell::const_new(); // String is a
// place holder for
// our data type
async fn send_dir<S>(socket: &mut S, dir: &Path) -> Result<()>
where
    S: Send + AsyncReadExt + AsyncWriteExt + Unpin + AsyncWrite + AsyncRead,
{
    let cwd: PathBuf = CWD
        .lock()
        .await
        .clone()
        .ok_or(anyhow!("No cwd found"))?
        .into();
    let mut fs_stream = fs::read_dir(dir).await?;
    while let Some(item) = fs_stream.next_entry().await? {
        let path = item.path();
        let rel_path = path.strip_prefix(&cwd)?;
        if path.is_file() {
            let mut file = File::open(&path).await?;
            let mut buff = String::new();
            file.read_to_string(&mut buff).await?;
            socket
                .write(&serde_json::to_vec(&Message {
                    method: Method::CreateFile,
                    file: rel_path.to_string_lossy().into(),
                    changes: vec![Change {
                        start: SOF,
                        end: EOF,
                        content: buff,
                    }],
                })?)
                .await?;
        } else if path.is_dir() {
            socket
                .write(&serde_json::to_vec(&Message {
                    method: Method::CreateDir,
                    file: rel_path.to_string_lossy().into(),
                    changes: vec![],
                })?)
                .await?;
            Box::pin(send_dir(socket, &path)).await?;
        } else {
            return Err(anyhow!("Invaild file type {}", path.display()));
        }
    }
    Ok(())
}

async fn copy_cwd<S>(socket: &mut S) -> Result<()>
where
    S: Send + AsyncReadExt + AsyncWriteExt + Unpin + AsyncWrite + AsyncRead,
{
    socket.write(br"\/INIT\/");
    let cwd: PathBuf = CWD
        .lock()
        .await
        .clone()
        .ok_or(anyhow!("No cwd found"))?
        .into(); //So this SHOULD never happen
    send_dir(socket, &cwd).await?;
    Ok(())
}
pub async fn start_server(config: Config, mut shutoff: watch::Receiver<bool>) -> Result<()> {
    let addrs = (config.hostname.clone(), config.port)
        .to_socket_addrs()?
        .next()
        .ok_or(anyhow!("Invaild socket addrs"))?;
    let server = TcpListener::bind(addrs).await?;

    let (tx, rx) = mpsc::channel::<PlaceHolder>(10);
    CHANGES.set(Arc::new(Mutex::new(rx)))?;
    loop {
        tokio::select! {
        result = server.accept() =>{
        let (mut stream, client_addrs) = result?;
        println!("Received connection from {}", client_addrs);
        let sender = tx.clone();
        let shutoff = shutoff.clone();
        task::spawn(async move {
            if let Err(error) = client_handler(&mut stream, sender, shutoff).await {
                eprintln!("{}", error);
            };
        });
        },
        _ = shutoff.changed() =>{
            if *shutoff.borrow() {
                break
            }
        }
        }
    }
    Ok(())
}
async fn client_handler<S>(
    socket: &mut S,
    sender: mpsc::Sender<PlaceHolder>,
    mut shutoff: watch::Receiver<bool>,
) -> Result<()>
where
    S: Send + AsyncReadExt + AsyncWriteExt + Unpin + AsyncWrite + AsyncRead,
{
    let mut buff = vec![];
    let mut receiver = get_subscriber().await;
    loop {
        tokio::select! {
            Ok(_) = socket.read(&mut buff) => {
                sender.send(serde_json::from_slice(&buff)?).await?; //Send any changes to the server
            },
            Ok(message) = receiver.recv() =>{
                socket.write(&serde_json::to_vec(&message)?).await?;
            }
            _ = shutoff.changed() =>{
                if *shutoff.borrow(){
                    socket.write(&serde_json::to_vec(&Message{
                        changes:vec![],
                        file:String::default(),
                        method: Method::Exit,
                    })?).await?;
                }
            }
        }
    }
}
async fn first_download<S>(socket: &mut S) -> Result<()>
where
    S: Send + AsyncReadExt + AsyncWriteExt + Unpin + AsyncWrite + AsyncRead,
{
    let mut buff = vec![];
    socket.read(&mut buff).await?;
    if buff != br"\/INIT\/" {
        return Err(anyhow!("Invaild start to session"));
    }
    buff.clear();
    while buff != br"\/DONE\/" {
        socket.read(&mut buff).await?;
        let message: Message = serde_json::from_slice(&buff)?;
        match message.method {
            Method::CreateFile => {
                File::create(message.file)
                    .await?
                    .write(message.changes[0].content.as_bytes())
                    .await?;
            }
            Method::CreateDir => todo!(),
            _ => todo!(),
        };
    }
    todo!()
}
async fn start_client(
    target: String,
    dir: &Path,
    mut shutoff: watch::Receiver<bool>,
) -> Result<()> {
    todo!()
}
pub fn get_incoming_stream() -> Option<&'static Arc<Mutex<mpsc::Receiver<PlaceHolder>>>> {
    CHANGES.get()
}
