use std::sync::atomic::{AtomicI32, Ordering};

use bevy::prelude::Event;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tokio_tungstenite::tungstenite;

static ID: AtomicI32 = AtomicI32::new(1);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HassWrapper<T> {
    id: i32,
    #[serde(flatten)]
    data: T,
}

#[derive(Debug, Clone, Event, Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum HassRequest {
    SubscribeEvents {
        #[serde(skip_serializing_if = "Option::is_none")]
        event_type: Option<String>,
    },
    GetStates,
}

#[derive(Debug, Clone, Event, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum HassResponse {
    Result { success: bool, result: serde_json::Value },
    Event { event: HassEvent },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "event_type", content = "data")]
pub enum HassEvent {
    StateChanged {
        entity_id: String,
        old_state: serde_json::Value,
        new_state: serde_json::Value,
    },
}

impl From<HassRequest> for HassWrapper<HassRequest> {
    fn from(data: HassRequest) -> Self {
        Self {
            id: ID.fetch_add(1, Ordering::Relaxed),
            data,
        }
    }
}

impl<T: Serialize> From<HassWrapper<T>> for tungstenite::Message {
    fn from(wrapper: HassWrapper<T>) -> Self {
        tungstenite::Message::Text(serde_json::to_string(&wrapper).unwrap())
    }
}

impl<T: DeserializeOwned> TryFrom<tungstenite::Message> for HassWrapper<T> {
    type Error = serde_json::Error;

    fn try_from(msg: tungstenite::Message) -> Result<Self, Self::Error> {
        serde_json::from_str(msg.to_text().unwrap())
    }
}

impl From<HassRequest> for tungstenite::Message {
    fn from(req: HassRequest) -> Self {
        HassWrapper::from(req).into()
    }
}

impl TryFrom<tungstenite::Message> for HassResponse {
    type Error = serde_json::Error;

    fn try_from(msg: tungstenite::Message) -> Result<Self, Self::Error> {
        HassWrapper::<HassResponse>::try_from(msg).map(|wrapper| wrapper.data)
    }
}
