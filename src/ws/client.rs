use std::{fmt::Debug, future::Future, marker::PhantomData};

use bevy::prelude::*;
use futures::{
    channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender},
    join, SinkExt, StreamExt,
};
use log::{error, warn};
use tokio::{net::TcpStream, runtime::Runtime, task::JoinHandle};
use tokio_tungstenite::{connect_async, tungstenite, MaybeTlsStream, WebSocketStream};

pub trait WsApi: Send {
    type Request: Into<tungstenite::Message> + Clone + Event + Send + 'static;
    type Response: TryFrom<tungstenite::Message, Error: Debug> + Event + Send + 'static;

    fn on_connect(
        #[allow(unused_variables)] stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
    ) -> impl Future<Output = ()> + std::marker::Send {
        async move {}
    }
}

#[derive(Debug, Resource)]
pub struct Client<A: WsApi> {
    rt: Runtime,
    handle: JoinHandle<()>,
    rx: UnboundedReceiver<tungstenite::Message>,
    tx: UnboundedSender<tungstenite::Message>,
    phantom: PhantomData<fn() -> A>,
}

impl<A: WsApi> Client<A> {
    pub fn connect(endpoint: String) -> Self {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("Could not build tokio runtime");

        let (mut ev_tx, ev_rx) = unbounded();
        let (from_handler_tx, mut from_handler_rx) = unbounded();

        let event_loop = async move {
            let (mut ws_stream, _) = connect_async(endpoint).await.expect("Failed to connect");

            A::on_connect(&mut ws_stream).await;

            let (mut write, mut read) = ws_stream.split();

            let read_handle = async move {
                while let Some(msg) = read.next().await {
                    match msg {
                        Err(e) => {
                            error!("failed to receive message: {:?}", e);
                        }
                        Ok(msg) => {
                            trace!("received message: {:?}", msg);

                            if let Err(e) = ev_tx.send(msg).await {
                                error!("failed to forward message, sink: {:?}", e);
                            }
                        }
                    }
                }
            };

            let write_handle = async move {
                loop {
                    let req = from_handler_rx.next().await;
                    match req {
                        None => {
                            warn!("handler dropped");
                            break;
                        }
                        Some(msg) => {
                            trace!("sending message: {:?}", msg);

                            if let Err(e) = write.send(msg).await {
                                warn!("failed to send message to server: {}", e);
                            }
                        }
                    }
                }
            };

            join!(read_handle, write_handle);

            warn!("event loop exited");
        };

        Self {
            handle: rt.spawn(event_loop),
            rt,
            rx: ev_rx,
            tx: from_handler_tx,
            phantom: PhantomData,
        }
    }

    pub fn try_recv(&mut self) -> Option<A::Response> {
        match self.rx.try_next() {
            Ok(Some(msg)) => match msg.try_into() {
                Ok(msg) => Some(msg),
                Err(e) => {
                    error!("failed to parse message: {e:?}");
                    None
                }
            },
            Ok(None) => {
                warn!("receiver dropped");
                None
            }
            Err(_) => None,
        }
    }

    pub fn send(&self, msg: A::Request) {
        if let Err(e) = self.tx.unbounded_send(msg.into()) {
            warn!("failed to forward message, sink: {e:?}");
        }
    }
}
