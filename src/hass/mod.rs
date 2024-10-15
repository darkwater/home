use bevy::prelude::*;
use serde::Deserialize;
use serde_with::{formats::CommaSeparator, serde_as, StringWithSeparator};

use self::{
    api::HassApi,
    protocol::{HassEvent, HassRequest, HassResponse},
};
use crate::ws::WebSocketClient;

mod api;
pub mod protocol;
mod ui;

#[derive(Default)]
pub struct HassPlugin;

impl Plugin for HassPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(WebSocketClient::<HassApi>::new())
            .add_systems(Startup, |mut ev: EventWriter<HassRequest>| {
                ev.send(HassRequest::SubscribeEvents {
                    event_type: Some("state_changed".into()),
                });
            })
            .add_systems(Update, (load_entities_from_gltf, ui::render_names, update_state));
    }
}

#[serde_as]
#[derive(Debug, Deserialize)]
struct HassExtras {
    #[serde_as(as = "StringWithSeparator::<CommaSeparator, String>")]
    entity_ids: Vec<String>,
}

#[derive(Debug, Clone, Component)]
struct HassDevice;

#[derive(Debug, Clone, Component)]
struct HassEntity {
    entity_id: String,
}

#[derive(Debug, Clone, Component)]
struct HassEntityState {
    state: String,
}

fn load_entities_from_gltf(
    query: Query<(Entity, &GltfExtras), Added<GltfExtras>>,
    mut commands: Commands,
    // hass: Res<HassApi>,
) {
    for (entity, extras) in query.iter() {
        let Ok(extras) = serde_json::from_str::<HassExtras>(&extras.value) else {
            continue;
        };

        println!("found extras: {extras:#?}");

        commands
            .entity(entity)
            .insert(HassDevice)
            .with_children(|parent| {
                for entity_id in extras.entity_ids {
                    parent.spawn((
                        HassEntity { entity_id },
                        HassEntityState { state: String::new() },
                        GlobalTransform::default(),
                    ));
                }
            });
    }
}

fn update_state(
    mut query: Query<(&HassEntity, &mut HassEntityState)>,
    mut ev: EventReader<HassResponse>,
) {
    for ev in ev.read() {
        if let HassResponse::Event {
            event: HassEvent::StateChanged { entity_id, old_state: _, new_state },
        } = ev
        {
            // TODO: cache these in a resource
            for (entity, mut state) in query.iter_mut() {
                if entity.entity_id == *entity_id {
                    state.state = new_state["state"].as_str().unwrap().to_string();
                }
            }
        }
    }
}
