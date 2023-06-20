mod connection;

#[tokio::main(flavor="current_thread")]
async fn main() {
    let connection = connection::ClientConnection::new("http://en.wikipedia.org".to_string());
    let connection = match connection.connect().await{
        Ok(connected) => connected,
        Err(e) => {
            eprintln!("{}", e);
            panic!()
        }
    };
}
