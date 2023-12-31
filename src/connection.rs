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
        ClientConnection { connected_state: Default::default(), url }
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
    pub async fn _disconnect(self) -> ClientConnection<Disconnected>{
        let message = CommandMessage::new(MessageContainer{
            message_body: String::from(""), 
            message_type: MessageTypes::Disconnection, 
            command: None,
            sender: String::from(""),
            room: String::new()
        });
        self.send(message).await;
        ClientConnection { connected_state: std::marker::PhantomData::<Disconnected>, url: self.url }
    }

    pub async fn send(&self, message: impl ClientServer) {
        match message.message_type(){
            MessageTypes::Normal => {
                let message = message.to_string();
                let client = Client::new();
                let x = client.post(format!("{}/v1/message", &self.url)).body(message).send().await.unwrap();
                if x.status() != StatusCode::OK || x.status() != StatusCode::CREATED{
                    println!("Can not send message: {}", x.status())
                }
            }

            MessageTypes::Command => {
                let message = message.to_string();
                let client = Client::new();
                client.post(format!("{}/v1/message/command", &self.url)).body(message).send().await.unwrap();

            }

            MessageTypes::Heartbeat => {
                let message = message.to_string();
                let client = Client::new();
                client.post(&self.url).body(message).send().await.unwrap();

            }

            _ => {
                
            }
        }
    }

    pub async fn send_heartbeat(&self){
        let message = MessageContainer { message_body: String::from(""), 
        message_type: MessageTypes::Heartbeat, 
        command: None, sender: String::from(""), room: String::new() };

        let heartbeat = CommandMessage::new(message);

        let client = Client::new();
        client.post(&self.url).body(heartbeat.to_string()).send().await.unwrap();
    }
}