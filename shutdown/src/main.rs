extern crate futures;
extern crate tokio_core;

use futures::{Future, Stream};
use tokio_core::io::{copy, Io};
use tokio_core::net::TcpListener;
use tokio_core::reactor::Core;

fn main() {
    let mut core = Core::new().expect("Could not crate the event loop");
    let handle = core.handle();

    let addr = "127.0.0.1:7000".parse().unwrap();
    let listener = TcpListener::bind(&addr, &handle).expect("Could not bind the socket");

    let server = listener.incoming().for_each(|(client, addr)| {
        let (reader, writer) = client.split();

        let future = copy(reader, writer)
            .map(move |amt| println!("wrote {} bytes to {}", amt, addr))
            .map_err(|err| println!("IO error: {:?}", err));
        handle.spawn(future);
        Ok(())
    });
    println!("Listening on localhost:7000");
    core.run(server).unwrap();
}
