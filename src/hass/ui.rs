use bevy::prelude::*;
use bevy_egui::{
    egui::{self, pos2, vec2, Color32, Id},
    EguiContexts,
};
use itertools::Itertools;

use super::{HassDevice, HassEntity, HassEntityState};

pub fn render_names(
    mut contexts: EguiContexts,
    camera: Query<(&Camera, &GlobalTransform)>,
    devices: Query<(Entity, &HassDevice, &GlobalTransform, &Children)>,
    entities: Query<(&HassEntity, &HassEntityState)>,
) {
    let Ok((camera, camera_transform)) = camera.get_single() else {
        return;
    };

    for (device, _, transform, children) in devices.iter() {
        let Ok(screen_position) =
            camera.world_to_viewport(camera_transform, transform.translation())
        else {
            continue;
        };

        let entities = entities.iter_many(children).collect::<Vec<_>>();

        let viewport = camera.logical_viewport_size().unwrap();
        let x = screen_position.x - viewport.x;
        let y = screen_position.y;
        let anchor = pos2(x, y);

        let offset = vec2(100., 50.);

        let ctx = contexts.ctx_mut();

        egui::Area::new(Id::new(device))
            .anchor(egui::Align2::RIGHT_TOP, anchor.to_vec2() - offset)
            .show(ctx, |ui| {
                ui.painter().line_segment(
                    [ui.cursor().right_top(), ui.cursor().right_top() + vec2(50., 0.)],
                    (1., Color32::WHITE),
                );
                ui.painter().line_segment(
                    [ui.cursor().right_top() + vec2(50., 0.), ui.cursor().right_top() + offset],
                    (1., Color32::WHITE),
                );

                ui.style_mut().interaction.selectable_labels = false;

                ui.strong(
                    entities
                        .iter()
                        .map(|(e, state)| format!("{}: {}", e.entity_id, state.state))
                        .join(",\n"),
                );
            });
    }
}
