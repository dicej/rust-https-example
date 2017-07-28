#[macro_use]
extern crate derive_error;

extern crate ws;
extern crate hyper;
extern crate hyper_tls;
extern crate native_tls;
extern crate tokio_core;
extern crate futures;

use std::io::{self, Write};
use futures::{Future, Stream};
use hyper::Client;
use hyper_tls::HttpsConnector;
use tokio_core::reactor::Core;

use ws::{connect, Handler, Sender, Handshake, Message, CloseCode};

#[derive(Debug, Error)]
enum MyError {
    IoError(io::Error),
    TlsError(native_tls::Error),
    UriError(hyper::error::UriError),
    HyperError(hyper::Error),
    WsError(ws::Error)
}

struct WSClient {
    out: Sender,
}

impl Handler for WSClient {
    fn on_open(&mut self, _: Handshake) -> ws::Result<()> {
        println!("on_open");
        self.out.send("Hello WebSocket")
    }

    fn on_message(&mut self, msg: Message) -> ws::Result<()> {
        println!("Got message: {}", msg);
        self.out.close(CloseCode::Normal)
    }

    fn on_error(&mut self, err: ws::Error) {
        println!("Got error: {}", err);
    }
}

fn run() -> Result<(), MyError> {
    println!("Hello, world!");
    connect("wss://echo.websocket.org", |out| WSClient { out: out })?;

    let mut core = Core::new()?;
    let handle = core.handle();
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

    Result::Ok(core.run(work)?)
}

fn main() {
    run().unwrap()
}
