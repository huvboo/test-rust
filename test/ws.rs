
extern crate websocket;

use websocket::sync::Server;
use websocket::Message;


fn test_ws() {
    let server = Server::bind("127.0.0.1:1234").unwrap();

    for connection in server.filter_map(Result::ok) {
        // Spawn a new thread for each connection.
        thread::spawn(move || {
            let mut client = connection.accept().unwrap();

            // let message = Message::text("Hello, client!");
            let message = Message::binary::<&[u8]>(&[1, 2, 3]);
            let _ = client.send_message(&message);

            // ...
        });
    }
}

fn main() {
    test_ws();
}