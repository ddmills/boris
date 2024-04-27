use bevy::ecs::system::ResMut;
use bevy_inspector_egui::{bevy_egui::EguiContexts, egui};

use super::GameSpeed;

pub fn job_toolbar(mut contexts: EguiContexts, mut game_speed: ResMut<GameSpeed>) {
    egui::Window::new("Speed").show(contexts.ctx_mut(), |ui| {
        if ui.button("Pause").clicked() {
            game_speed.is_paused = !game_speed.is_paused;
        }

        if ui.button("0.5x").clicked() {
            game_speed.is_paused = false;
            game_speed.speed = 0.5;
        }

        if ui.button("1x").clicked() {
            game_speed.is_paused = false;
            game_speed.speed = 1.;
        }

        if ui.button("2x").clicked() {
            game_speed.is_paused = false;
            game_speed.speed = 2.;
        }

        if ui.button("4x").clicked() {
            game_speed.is_paused = false;
            game_speed.speed = 4.;
        }
    });
}
