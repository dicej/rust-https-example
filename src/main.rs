#[macro_use]
extern crate derive_error;

extern crate hyper;
extern crate hyper_tls;
extern crate native_tls;
extern crate tokio_core;
extern crate futures;
extern crate websocket;

use std::io::{self, Write};
use futures::{Future, Stream};
use hyper::Client;
use hyper_tls::HttpsConnector;
use tokio_core::reactor::Core;
use websocket::{ClientBuilder, Message};
use websocket::message::OwnedMessage;
use websocket::futures::Sink;

#[derive(Debug, Error)]
enum MyError {
    IoError(io::Error),
    TlsError(native_tls::Error),
    UriError(hyper::error::UriError),
    HyperError(hyper::Error),
    ParseError(websocket::client::ParseError),
    WebsocketError(websocket::WebSocketError)
}

fn run() -> Result<(), MyError> {
    println!("Hello, world!");

    let mut core = Core::new()?;
    let handle = core.handle();

    let ws_client = ClientBuilder::new("wss://echo.websocket.org")?
        .async_connect(None, &handle)
        .and_then(|(s, _)| s.send(Message::text("hallo").into()))
        .and_then(|s| s.into_future().map_err(|e| e.0))
        .map(|(m, _)| match m {
            Some(OwnedMessage::Text(text)) => println!("Got message: {}", text),
            _=> (),
        });

    core.run(ws_client)?;
    
    let client = Client::configure()
        .connector(HttpsConnector::new(4, &handle)?)
        .build(&handle);

    let work = client.get("https://hyper.rs".parse()?).and_then(|res| {
        println!("Response: {}", res.status());

        res.body().for_each(|chunk| {
            io::stdout()
                .write_all(&chunk)
                .map_err(From::from)
        })
    });

    core.run(work).map_err(MyError::from)
}

fn main() {
    run().unwrap()
}
