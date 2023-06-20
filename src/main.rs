mod connection;
use tokio;
use std::io;
use std::sync::Arc;
use message::{client_server::NormalMessage, message::{MessageContainer, MessageTypes}, client_server_trait::ClientServer};


#[tokio::main(flavor="current_thread")]
async fn main() -> io::Result<()>{
    let connection = connection::ClientConnection::new("http://en.wikipedia.org".to_string());
    let connection = match connection.connect().await{
        Ok(connected) => Arc::new(connected),
        Err(e) => {
            eprintln!("{}", e);
            panic!()
        }
    }; 
    println!("Enter nick:");
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer)?;
    let nick = Arc::new(buffer);

    let heartbeat_connection = connection.clone();
    tokio::spawn(async move {
        loop{
            heartbeat_connection.send_heartbeat().await;
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        }
    });

    loop{
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer)?;
        let sender = nick.as_ref();
        if buffer.starts_with(":"){}
        let container = MessageContainer::new(buffer, 
            MessageTypes::Normal, None, sender.to_owned());
        let message = NormalMessage::new(container);
        println!("{:?}", message);
        connection.send(message).await;

    }
}
