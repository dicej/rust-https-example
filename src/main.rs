#[macro_use]
extern crate derive_error;

extern crate hyper;
extern crate hyper_tls;
extern crate native_tls;
extern crate tokio_core;
extern crate futures;
extern crate websocket;

use std::io::{self};
use futures::{Future, Stream, Poll};
use futures::future::join_all;
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

struct ResultFuture<A> where A: Future {
    result: Option<Result<A,A::Error>>
}

impl<A> Future for ResultFuture<A> where A: Future {
    type Item = A::Item;
    type Error = A::Error;

    fn poll(&mut self) -> Poll<A::Item, A::Error> {
        match self.result {
            Some(Ok(ref mut future)) => future.poll(),
            _ => Err(self.result.take().expect("cannot poll twice").err().unwrap())
        }
    }
}

fn result_future<A>(result: Result<A,A::Error>) -> ResultFuture<A> where A: Future {
    ResultFuture { result: Some(result) }
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

    let client = Client::configure()
        .connector(HttpsConnector::new(4, &handle)?)
        .build(&handle);

    core.run
        (ws_client.map_err(MyError::from).join
         (join_all
          (["hyper.rs", "google.com"].iter()
           .map(|host| format!("https://{}", host).parse()
                .map_err(hyper::Error::from)
                .map(|uri| client.get(uri))
                .map(|future| future.map
                     (move |res| println!
                      ("{} response: {}", host, res.status()))))
           .map(result_future)).map_err(MyError::from)))?;

    Ok(())
}

fn main() {
    run().unwrap()
}
