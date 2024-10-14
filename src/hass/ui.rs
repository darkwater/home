use bevy::prelude::*;
use bevy_egui::{
    egui::{self, pos2, vec2, Color32, Id},
    EguiContexts,
};

use super::HassDevice;

pub fn render_names(
    mut contexts: EguiContexts,
    camera: Query<(&Camera, &GlobalTransform)>,
    entities: Query<(&HassDevice, &GlobalTransform)>,
) {
    let Ok((camera, camera_transform)) = camera.get_single() else {
        return;
    };

    for (entity, transform) in entities.iter() {
        let Some(screen_position) =
            camera.world_to_viewport(camera_transform, transform.translation())
        else {
            continue;
        };

        let viewport = camera.logical_viewport_size().unwrap();
        let x = screen_position.x - viewport.x;
        let y = screen_position.y;
        let anchor = pos2(x, y);

        let offset = vec2(100., 50.);

        let ctx = contexts.ctx_mut();

        egui::Area::new(Id::new(entity.entity_ids.clone()))
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

                ui.strong(entity.entity_ids.join(", "));
            });
    }
}
