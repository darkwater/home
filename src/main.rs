#![allow(clippy::type_complexity)]

use bevy::prelude::*;
use bevy_atmosphere::plugin::AtmospherePlugin;
use bevy_egui::EguiPlugin;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

mod hass;
mod ws;

fn main() {
    dotenvy::dotenv().unwrap();

    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "home".into(),
                    name: Some("red.dark.home".into()),
                    ..default()
                }),
                ..default()
            }),
            AtmospherePlugin,
            EguiPlugin,
            PanOrbitCameraPlugin,
            hass::HassPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, setup_camera)
        .add_systems(Update, close_on_esc)
        .run();
}

fn setup(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands
        .spawn((SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("home.glb"))),));
}

fn setup_camera(query: Query<Entity, Added<Camera>>, mut commands: Commands) {
    for camera in query.iter() {
        commands.entity(camera).insert((
            PanOrbitCamera {
                button_pan: MouseButton::Middle,
                button_orbit: MouseButton::Right,
                ..default()
            },
            // AtmosphereCamera::default(),
        ));
    }
}

pub fn close_on_esc(
    mut commands: Commands,
    focused_windows: Query<(Entity, &Window)>,
    input: Res<ButtonInput<KeyCode>>,
) {
    for (window, focus) in focused_windows.iter() {
        if !focus.focused {
            continue;
        }

        if input.just_pressed(KeyCode::Escape) {
            commands.entity(window).despawn();
        }
    }
}
