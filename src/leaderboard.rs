use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Write},
    path::Path,
};

use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Color32, Frame, Margin},
    EguiContexts,
};
use serde::{Deserialize, Serialize};

use crate::{map::Map, State};
pub struct LeaderboardPlugin;

#[derive(Event)]
pub enum LeaderboardEvent {
    SaveLeaderboardData(String, f32),
}

impl Plugin for LeaderboardPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<LeaderboardEvent>()
            .add_systems(Startup, load_highscores)
            .add_systems(
                Update,
                display_leaderboard.run_if(
                    in_state(crate::State::Finished).or_else(in_state(crate::State::Leaderboard)),
                ),
            )
            .add_systems(
                PostUpdate,
                add_highscore.run_if(on_event::<LeaderboardEvent>()),
            );
    }
}

fn load_highscores(mut commands: Commands) {
    #[cfg(target_arch = "wasm32")]
    {
        commands.init_resource::<MapHighscores>();
        return;
    }

    let path = Path::new("maps").join("highscores.json");
    if !path.exists() {
        File::create(path).unwrap();
        commands.init_resource::<MapHighscores>();
        return;
    } else {
        let mut file = File::open(path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let highscores = serde_json::from_str::<MapHighscores>(&contents);
        match highscores {
            Ok(highscores) => {
                commands.insert_resource(highscores);
            }
            Err(_) => commands.init_resource::<MapHighscores>(),
        }
    }
}

fn display_leaderboard(
    mut contexts: EguiContexts,
    highscores: Res<MapHighscores>,
    map: Option<Res<Map>>,
    mut state: ResMut<NextState<State>>,
) {
    let ctx = contexts.ctx_mut();
    // if there is a map, only display the current maps highscores.
    if let Some(map) = map {
        let mut map_highscores = highscores.maps.get(&map.name).unwrap().clone();
        // Floats have no ord, so use partial
        map_highscores.sort_by(|a, b| a.partial_cmp(b).unwrap());

        egui::Area::new("highscores").show(ctx, |ui| {
            Frame {
                outer_margin: Margin::symmetric(300., 0.),
                inner_margin: Margin::same(20.),
                fill: Color32::from_rgba_unmultiplied(255, 255, 255, 150),
                ..Default::default()
            }
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label("Highscores:");
                    for h in map_highscores {
                        ui.label(format!("{}", h));
                    }
                });
            });
        });
    } else {
        // otherwise, display all highscores.
        egui::Area::new("highscores").show(ctx, |ui| {
            Frame {
                outer_margin: Margin::symmetric(300., 0.),
                inner_margin: Margin::same(20.),
                fill: Color32::from_rgba_unmultiplied(255, 255, 255, 150),
                ..Default::default()
            }
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    if highscores.maps.is_empty() {
                        ui.label("No highscores yet.");
                        return;
                    } else {
                        ui.label("Highscores:");
                    }

                    for (map, highscores) in &highscores.maps {
                        let mut map_highscores = highscores.clone();
                        // Floats have no ord, so use partial
                        map_highscores.sort_by(|a, b| a.partial_cmp(b).unwrap());
                        ui.collapsing(map.replace(".glb", ""), |ui| {
                            for h in map_highscores {
                                ui.label(format!("{}", h));
                            }
                        });
                    }
                });
                ui.vertical_centered(|ui| {
                    let btn = ui.button("Back");
                    if btn.clicked() {
                        state.set(crate::State::Mainscreen);
                    }
                });
            });
        });
    }
}

fn add_highscore(mut er: EventReader<LeaderboardEvent>, mut leaderboard: ResMut<MapHighscores>) {
    for e in er.read() {
        match e {
            LeaderboardEvent::SaveLeaderboardData(map, time) => {
                let map_scores = leaderboard.maps.get_mut(map);
                if let Some(map_scores) = map_scores {
                    map_scores.push(*time);
                } else {
                    leaderboard.maps.insert(map.clone(), vec![*time]);
                }

                let serialized = serde_json::to_string(leaderboard.into_inner()).unwrap();

                // Overwrite the file
                let mut file = File::create("maps/highscores.json").unwrap();
                _ = file.write_all(serialized.as_bytes());
                break; // This should only be triggered once either way
            }
        }
    }
}

#[derive(Default, Resource, Serialize, Deserialize)]
pub struct MapHighscores {
    maps: HashMap<String, Vec<f32>>,
}
