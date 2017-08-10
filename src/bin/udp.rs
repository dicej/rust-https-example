extern crate futures;
#[macro_use]
extern crate tokio_core;

use std::{env, io};
use std::net::SocketAddr;
use std::str::from_utf8;

use futures::{Future, Poll};
use tokio_core::net::UdpSocket;
use tokio_core::reactor::Core;

struct Server {
    socket: UdpSocket,
}

impl Future for Server {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<(), io::Error> {
        loop {
            let mut buf = vec![0; 2048];
            let (size, _) = try_nb!(self.socket.recv_from(&mut buf));
            println!("got message: {:?}", from_utf8(&buf[..size]));
        }
    }
}

fn main() {
    let addr = env::args().nth(1).unwrap_or("127.0.0.1:8080".to_string());
    let addr = addr.parse::<SocketAddr>().unwrap();

    // Create the event loop that will drive this server, and also bind the
    // socket we'll be listening to.
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let socket = UdpSocket::bind(&addr, &handle).unwrap();
    println!("Listening on: {}", addr);

    // Next we'll create a future to spawn (the one we defined above) and then
    // we'll run the event loop by running the future.
    core.run(Server { socket: socket }).unwrap();
}
