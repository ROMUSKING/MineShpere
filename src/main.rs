use bevy::{
    picking::mesh_picking::MeshPickingPlugin,
    prelude::*,
};

mod game;
mod render;
mod utils;

use game::*;
use render::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bevy 0.16 Sphere Sweeper".into(),
                resolution: (1280.0, 800.0).into(),
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(MeshPickingPlugin)
        .insert_resource(MeshPickingSettings {
            require_markers: false, 
            ..default()
        })
        .init_state::<AppState>()
        .insert_resource(load_game())
        .init_resource::<GameSettings>()
        .init_resource::<CellVisuals>() // Initialized in load_assets
        .add_event::<RevealCell>() 
        .add_event::<ChordCell>()
        .add_event::<ChordCell>()
        .add_systems(Startup, (setup_scene, setup_stars, setup_planets))
        .add_systems(OnEnter(AppState::Loading), load_assets)
        .add_systems(OnEnter(AppState::MainMenu), setup_menu)
        .add_systems(OnExit(AppState::MainMenu), cleanup_menu)
        .add_systems(OnEnter(AppState::Playing), (spawn_board, setup_ui))
        .add_systems(OnExit(AppState::Playing), (cleanup_board, cleanup_ui))
        .add_systems(Update, save_game.run_if(resource_changed::<GameSession>))
        .add_systems(Update, (
            update_hud,
            camera_orbit_controls,
            check_win_condition,
            toggle_invert_y,
        ).run_if(in_state(AppState::Playing)))
        .add_systems(Update, process_reveal_queue.run_if(in_state(AppState::Playing)))
        
        // Game Over / Victory Logic
        .add_systems(OnEnter(AppState::GameOver), (reveal_all_mines, setup_menu))
        .add_systems(OnEnter(AppState::Victory), (setup_menu, update_max_level))
        .add_systems(Update, menu_interaction.run_if(in_state(AppState::MainMenu).or(in_state(AppState::GameOver)).or(in_state(AppState::Victory))))
        .add_systems(OnExit(AppState::GameOver), (cleanup_board, cleanup_menu))
        .add_systems(OnExit(AppState::Victory), (cleanup_board, cleanup_menu))
        
        .run();
}
