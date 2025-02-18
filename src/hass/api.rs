use std::env;

use futures::{SinkExt as _, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite, MaybeTlsStream, WebSocketStream};

use super::protocol::{HassRequest, HassResponse};
use crate::ws::WsApi;

#[derive(Debug, Default)]
pub struct HassApi;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum AuthMessage {
    AuthRequired { ha_version: String },
    Auth { access_token: String },
    AuthOk { ha_version: String },
    AuthInvalid { message: String },
}

impl WsApi for HassApi {
    type Request = HassRequest;
    type Response = HassResponse;

    async fn on_connect(stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>) {
        let auth_req = stream
            .next()
            .await
            .expect("unexpected end of stream")
            .expect("failed to read message")
            .into_text()
            .expect("failed to parse message");

        let auth_req =
            serde_json::from_str::<AuthMessage>(&auth_req).expect("failed to parse message");

        if let AuthMessage::AuthRequired { ha_version } = auth_req {
            log::info!("Authenticating with Home Assistant version {}", ha_version);
        } else {
            log::warn!("Unexpected message: {:?}", auth_req);
        }

        let auth = AuthMessage::Auth {
            access_token: env::var("HASS_TOKEN").expect("HASS_TOKEN not set"),
        };

        let auth = serde_json::to_string(&auth).expect("failed to serialize message");

        stream
            .send(tungstenite::Message::Text(auth.into()))
            .await
            .expect("failed to send message");

        let auth_ok = stream
            .next()
            .await
            .expect("unexpected end of stream")
            .expect("failed to read message")
            .into_text()
            .expect("failed to parse message");

        let auth_ok =
            serde_json::from_str::<AuthMessage>(&auth_ok).expect("failed to parse message");

        match auth_ok {
            AuthMessage::AuthOk { ha_version } => {
                log::info!("Authenticated with Home Assistant version {}", ha_version);
            }
            AuthMessage::AuthInvalid { message } => {
                log::error!("Failed to authenticate: {}", message);
            }
            _ => {
                log::warn!("Unexpected message: {:?}", auth_ok);
            }
        }
    }
}
