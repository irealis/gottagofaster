use bevy::{app::AppExit, prelude::*, window::CursorGrabMode};
use bevy_egui::{
    egui::{self, Color32, FontId, Frame, Margin, TextStyle, Visuals},
    EguiContexts,
};

use crate::{ghost::GhostOneshots, map::Map, Maps, State, StateOneshots};

pub fn setup_ui(mut contexts: EguiContexts) {
    let ctx = contexts.ctx_mut();
    let mut style = (*ctx.style()).clone();
    style.visuals = Visuals::light();
    style.text_styles = [
        (
            TextStyle::Heading,
            FontId::new(25.0, egui::FontFamily::Proportional),
        ),
        (
            TextStyle::Body,
            FontId::new(16.0, egui::FontFamily::Proportional),
        ),
        (
            TextStyle::Monospace,
            FontId::new(12.0, egui::FontFamily::Proportional),
        ),
        (
            TextStyle::Button,
            FontId::new(16.0, egui::FontFamily::Proportional),
        ),
        (
            TextStyle::Small,
            FontId::new(8.0, egui::FontFamily::Proportional),
        ),
    ]
    .into();
    ctx.set_style(style);
}

pub fn ui_mainscreen(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut exit: EventWriter<AppExit>,
    mut state: ResMut<NextState<State>>,
    mut windows: Query<&mut Window>,
    maps: Res<Maps>,
    oneshots: Res<StateOneshots>,
    ghost_oneshots: Res<GhostOneshots>,
) {
    let ctx = contexts.ctx_mut();
    egui::Area::new("forg").show(ctx, |ui| {
        Frame {
            outer_margin: Margin::symmetric(300., 0.),
            inner_margin: Margin::same(20.),
            fill: Color32::from_rgba_unmultiplied(255, 255, 255, 150),
            ..Default::default()
        }
        .show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.horizontal(|ui| {
                    ui.collapsing("Load map", |ui| {
                        for map in &maps.maps {
                            if ui.button(map).clicked() {
                                let mut window = windows.single_mut();
                                window.cursor.grab_mode = CursorGrabMode::Locked;
                                window.cursor.visible = false;
                                commands.insert_resource(Map::from(map.as_str()));
                                commands.run_system(oneshots.load_map);
                                commands.run_system(ghost_oneshots.load);
                                state.set(State::Playing);
                            }
                        }
                    });
                });
                if ui.button("Leaderboard").clicked() {}
                if ui.button("Quit").clicked() {
                    exit.send(AppExit);
                }
            })
        })
    });
}

pub fn ui_finish(
    mut contexts: EguiContexts,
    mut state: ResMut<NextState<State>>,
    mut commands: Commands,
    mut windows: Query<&mut Window>,
    oneshots: Res<StateOneshots>,
) {
    let ctx = contexts.ctx_mut();
    egui::Area::new("forg").show(ctx, |ui| {
        Frame {
            outer_margin: Margin::symmetric(300., 0.),
            inner_margin: Margin::same(20.),
            fill: Color32::from_rgba_unmultiplied(255, 255, 255, 150),
            ..Default::default()
        }
        .show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Finish!");
                ui.label(format!("Finished in {}", "TODO"));
                ui.horizontal(|ui| {
                    if ui.button("Reset").clicked() {
                        commands.run_system(oneshots.unload);
                        let mut window = windows.single_mut();
                        window.cursor.grab_mode = CursorGrabMode::Locked;
                        window.cursor.visible = false;
                        commands.run_system(oneshots.load_map);
                        state.set(State::Playing);
                    }
                    if ui.button("Back to menu").clicked() {
                        state.set(State::Mainscreen);
                        commands.run_system(oneshots.unload);
                    }
                });
                egui::warn_if_debug_build(ui);
            })
        })
    });
}
