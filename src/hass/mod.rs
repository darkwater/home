use std::env;

use bevy::{prelude::*, tasks::AsyncComputeTaskPool};
use serde::Deserialize;
use serde_with::{formats::CommaSeparator, serde_as, StringWithSeparator};

mod ui;

#[derive(Default)]
pub struct HassPlugin;

impl Plugin for HassPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HassApi>()
            .add_systems(Update, (setup_entities, ui::render_names));
    }
}

#[derive(Debug, Clone, Resource)]
struct HassApi {
    client: surf::Client,
}

impl Default for HassApi {
    fn default() -> Self {
        let client = surf::Config::new()
            .set_base_url(
                env::var("HASS_URL")
                    .expect("HASS_URL unset")
                    .parse()
                    .expect("HASS_URL invalid"),
            )
            .add_header(
                "Authorization",
                format!("Bearer {}", env::var("HASS_TOKEN").expect("HASS_TOKEN unset")),
            )
            .unwrap()
            .try_into()
            .unwrap();

        Self { client }
    }
}

#[serde_as]
#[derive(Debug, Component, Deserialize)]
struct HassDevice {
    #[serde_as(as = "StringWithSeparator::<CommaSeparator, String>")]
    entity_ids: Vec<String>,
}

fn setup_entities(
    query: Query<(Entity, &GltfExtras), Added<GltfExtras>>,
    mut commands: Commands,
    hass: Res<HassApi>,
) {
    for (entity, extras) in query.iter() {
        let Ok(extras) = serde_json::from_str::<HassDevice>(&extras.value) else {
            continue;
        };

        println!("found extras: {extras:#?}");

        let hass = hass.clone();
        let entity_ids = extras.entity_ids.clone();

        commands.entity(entity).insert(extras);

        AsyncComputeTaskPool::get()
            .spawn(async move {
                for id in entity_ids {
                    let mut res = hass
                        .client
                        .get(format!("/api/states/{}", id))
                        .await
                        .unwrap();

                    let body = res.body_json::<serde_json::Value>().await.unwrap();

                    println!("body: {body:#?}");
                }
            })
            .detach();
    }
}
