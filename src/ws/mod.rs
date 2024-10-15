// based on https://github.com/Xide/bevy-websocket-adapter

use std::{env, marker::PhantomData};

use bevy::prelude::*;
use client::Client;

mod client;

pub use client::WsApi;

#[derive(Default, Debug)]
pub struct WebSocketClient<A: WsApi> {
    phantom: PhantomData<fn() -> A>,
}

impl<A: WsApi> WebSocketClient<A> {
    pub fn new() -> Self {
        Self { phantom: PhantomData }
    }
}

impl<A: WsApi + 'static> Plugin for WebSocketClient<A> {
    fn build(&self, app: &mut App) {
        let client = Client::<A>::connect(env::var("HASS_URL").unwrap());

        app.insert_resource(client)
            .add_event::<A::Request>()
            .add_event::<A::Response>()
            .add_systems(PreUpdate, handle_messages::<A>);
    }
}

fn handle_messages<A: WsApi + 'static>(
    mut client: ResMut<Client<A>>,
    mut tx: EventReader<A::Request>,
    mut rx: EventWriter<A::Response>,
) {
    while let Some(ev) = client.try_recv() {
        rx.send(ev);
    }

    for ev in tx.read() {
        client.send(ev.clone());
    }
}
