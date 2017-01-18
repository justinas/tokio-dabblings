extern crate futures;
extern crate tokio_core;

use futures::{AsyncSink, Future, Poll, Sink, StartSend, Stream};
use futures::stream::{MergedItem, SplitSink};
use futures::sync::mpsc;
use tokio_core::io::{Framed, Io};
use tokio_core::net::{TcpListener, TcpStream};
use tokio_core::reactor::Core;

struct ShutdownRequest;

/// Implements a stream sink which:
///     * Sends a ShutdownRequest to the given channel if a "shutdown" message is sent to this sink
///     * Forwards any other message to the underlying sink
struct EchoShutdownSink {
    channel: mpsc::Sender<ShutdownRequest>,
    client_sink: SplitSink<Framed<TcpStream, codec::LineCodec>>,
    handle: tokio_core::reactor::Handle,
}

impl Sink for EchoShutdownSink {
    type SinkItem = String;
    type SinkError = ();

    fn start_send(&mut self, item: String) -> StartSend<Self::SinkItem, Self::SinkError> {
        match item {
            ref i if i == "shutdown" => {
                self.handle.spawn(self.channel
                    .clone()
                    .send(ShutdownRequest)
                    .map(|_| ())
                    .map_err(|_| ()));
                Ok(AsyncSink::Ready)
            }
            i => self.client_sink.start_send(i).map_err(|_| ()),
        }
    }
    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        self.channel
            .poll_complete()
            .map_err(|_| ())
            .and_then(|_| self.client_sink.poll_complete().map_err(|_| ()))
    }
}

fn main() {
    let mut core = Core::new().expect("Could not crate the event loop");
    let handle = core.handle();

    let addr = "127.0.0.1:7000".parse().unwrap();
    let listener = TcpListener::bind(&addr, &handle).expect("Could not bind the socket");

    let (sender, receiver) = mpsc::channel(0);
    let server = listener.incoming()
        .map_err(|_| ()) // TODO: do not discard the I/O error
        .merge(receiver).for_each(|thing| {
        match thing {
            MergedItem::First((socket, _)) => {
                let (client_sink, client_stream) = socket.framed(codec::LineCodec).split();
                let future = EchoShutdownSink {
                    channel: sender.clone(),
                    client_sink: client_sink,
                    handle: handle.clone(),
                }.send_all(client_stream.map_err(|_| ()));
                handle.spawn(future.map(|_| ()));
            }

            // for_each breaks the "loop" when it receives an errored future from the closure
            // impl<T, E> IntoFuture for Result<T, E> helps us here as well.
            MergedItem::Second(_) => return Err(()),
            MergedItem::Both(_, _) => return Err(()),
        }

        Ok(())
    });
    println!("Listening on localhost:7000");
    if let Err(_) = core.run(server) {
        println!("Shutting down on the request of a user");
    }
}

mod codec;
