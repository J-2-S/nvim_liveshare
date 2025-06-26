use anyhow::Result;
use std::sync::Arc;
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::{TcpStream, ToSocketAddrs},
    sync::{Mutex, OnceCell, mpsc},
    task, time,
};
type PlaceHolder = String;
static HOST_UPDATES: OnceCell<Arc<Mutex<mpsc::Receiver<PlaceHolder>>>> = OnceCell::const_new();
pub async fn connect(host_addrs: impl ToSocketAddrs) -> Result<()> {
    let mut server_stream = TcpStream::connect(host_addrs).await?;
    let (tx, mut rx) = mpsc::channel::<PlaceHolder>(10);
    Ok(())
}
pub async fn first_download<S>(socket: &mut S) -> Result<()>
where
    S: AsyncReadExt + AsyncWriteExt + AsyncRead + AsyncWrite + Send + Unpin,
{
    todo!() // I don't know how we want to do this yet so we need to talk about it or I will write
    // it myself
}
pub async fn server_handle<S>(socket: &mut S, tx: mpsc::Sender<PlaceHolder>) -> Result<()>
where
    S: AsyncReadExt + AsyncWriteExt + AsyncRead + AsyncWrite + Send + Unpin,
{
    loop {
        let mut buff = vec![];
        tokio::select! {
            size = socket.read(&mut buff)=>{
                tx.send(todo!("buff")).await?;
            },
            //write the handle for jacobs end of the program
            _ = time::sleep(time::Duration::from_secs(10))=>{

            }
        }
        todo!()
    }
}
pub fn get_incoming_stream() -> Option<&'static Arc<Mutex<mpsc::Receiver<PlaceHolder>>>> {
    HOST_UPDATES.get()
}
