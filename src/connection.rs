use message::client_server::CommandMessage;
use reqwest::{Client};
use reqwest::StatusCode;

use message::message::{MessageContainer, MessageTypes};
use message::client_server_trait::ClientServer;
use std::{error::Error, fmt};

#[derive(Debug)]
pub struct ConnectionError{
    message: String
}

impl Error for ConnectionError{}

impl fmt::Display for ConnectionError{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}


pub struct Connected;
pub struct Disconnected;

pub struct ClientConnection<State = Disconnected>{
    connected_state: std::marker::PhantomData<State>,
    url: String
}

impl ClientConnection<Disconnected>{
    pub fn new(url: String) -> ClientConnection<Disconnected>{
        ClientConnection { connected_state: Default::default(), url: url }
    }
    pub async fn connect(&self) -> Result<ClientConnection<Connected>, ConnectionError>{
        let client = Client::new();
        let response = client.get(&self.url).send().await.unwrap();
        match response.status(){
            StatusCode::OK =>{
                Ok(ClientConnection { connected_state: std::marker::PhantomData::<Connected>, url: self.url.clone() })
            },
            _ => {
                Err(ConnectionError{message: format!("Could not connect: {}\nBecause: {}", response.status(), response.text().await.unwrap())})
            }
        }
    }
}

impl ClientConnection<Connected>{
    pub async fn disconnect(self) -> ClientConnection<Disconnected>{
        let message = CommandMessage::new(MessageContainer{
            message_body: String::from(""), 
            message_type: MessageTypes::Disconnection, 
            command: None});
        self.send(message).await;
        ClientConnection { connected_state: std::marker::PhantomData::<Disconnected>, url: self.url }
    }

    pub async fn send(&self, message: impl ClientServer) {
        let message = message.to_string();
        let client = Client::new();
        client.post(&self.url).body(message).send().await.unwrap();
    }

    pub async fn send_heartbeat(&self){
        let message = MessageContainer { message_body: String::from(""), 
        message_type: MessageTypes::Heartbeat, 
        command: None };

        let heartbeat = CommandMessage::new(message);

        let client = Client::new();
        client.post(&self.url).body(heartbeat.to_string()).send().await.unwrap();
    }
}