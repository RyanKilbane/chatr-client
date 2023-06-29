mod connection;
mod client_attributes;

use std::io;
use std::sync::Arc;
use client_attributes::ClientAttributes;
use command_reader::lexer;
use message::{client_server::{NormalMessage, CommandMessage}, message::{MessageContainer, MessageTypes}, client_server_trait::ClientServer};


#[tokio::main(flavor="current_thread")]
async fn main() -> io::Result<()>{
    let connection = connection::ClientConnection::new("http://127.0.0.1:8081".to_string());
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
    let nick = buffer.clone();
    buffer.clear();
    println!("Enter room:");
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer)?;
    let room = buffer.clone();

    let mut attributes = client_attributes::ClientAttributes::new(nick, Some(room));

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
        if buffer.is_empty(){
            continue;
        }
        if buffer.starts_with(':'){
            let mut lexer = lexer::Lexer::new(&buffer);
            let lexed = lexer.scan();
            if lexed.is_local_command(){
                attributes = handle_local_commands(lexed.clone(), attributes);
            } else{
                let message = build_command_message(String::new(), Some(buffer.to_owned()), &attributes);
                connection.send(message).await;
            }
        } else{
            let message = build_normal_message(buffer.to_owned(), &attributes);
            connection.send(message).await;
        }
    }
}


// Can put these two functions into a single function
fn build_command_message(message_body: String, command: Option<String>, client_attributes: &ClientAttributes) -> CommandMessage{
    let container = MessageContainer::new(message_body, 
        MessageTypes::Command, command, 
        client_attributes.nick.to_string(), 
        client_attributes.room.as_ref().unwrap().to_string());
    
    CommandMessage::new(container)
}

fn build_normal_message(message_body: String, client_attributes: &ClientAttributes) -> NormalMessage{
    let container = MessageContainer::new(message_body, MessageTypes::Normal, 
        None, 
        client_attributes.nick.to_string(), 
        client_attributes.room.as_ref().unwrap().to_string());

    NormalMessage::new(container)
}


fn handle_local_commands(lexer: lexer::Lexer<lexer::Parsed>, client_attributes: ClientAttributes) -> ClientAttributes{
    let command = lexer.tokens.first().unwrap();
    match *command {
        lexer::Tokens::Nick => {
            match lexer.tokens.get(1).unwrap(){
                lexer::Tokens::Arg(new_nick) => {
                    ClientAttributes::new(new_nick.to_string(), client_attributes.room.to_owned())
                }

                _ => {
                    eprintln!("Once again I've no idea how you got here");
                    panic!();
                }
            }
        }

        lexer::Tokens::Join => {
            match lexer.tokens.get(1).unwrap(){
                lexer::Tokens::Arg(new_room) => {
                    ClientAttributes::new(client_attributes.nick.to_string(), Some(new_room.to_string()))
                }

                _ => {
                    eprintln!("Once again I've no idea how you got here");
                    panic!();
                }
            }
        }
        _ => {
            eprintln!("I don't understand how you could ever have gotten here");
            panic!()
        }
    }
}

