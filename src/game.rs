use bevy::prelude::*;
use std::collections::{HashMap, HashSet};
use rand::prelude::*;
use crate::render::CellVisuals;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;

// --- CONFIGURATION ---
pub const SPHERE_RADIUS: f32 = 2.0;

pub const BASE_MINE_PERCENTAGE: f64 = 0.15;

// --- STATE ---
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    #[default]
    Loading,
    MainMenu,
    Playing,
    GameOver,
    Victory,
}

#[derive(Resource)]
pub struct GameSettings {
    pub invert_y: bool,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self { invert_y: false }
    }
}

#[derive(Resource, Serialize, Deserialize)]
pub struct GameSession {
    pub level: u32,
    pub max_level: u32,
    pub is_first_click: bool,
    pub total_mines: usize,
    pub flags_placed: usize,
    pub cells_revealed: usize,
    pub total_cells: usize,
    pub start_time: Option<f64>,
}

impl Default for GameSession {
    fn default() -> Self {
        Self {
            level: 1,
            max_level: 1,
            is_first_click: true,
            total_mines: 0,
            flags_placed: 0,
            cells_revealed: 0,
            total_cells: 0,
            start_time: None,
        }
    }
}

// --- EVENTS ---
#[derive(Event)]
pub struct RevealCell(pub Entity);

#[derive(Event)]
pub struct ChordCell(pub Entity);

// --- COMPONENTS ---

#[derive(Component)]
#[require(Mesh3d, MeshMaterial3d<StandardMaterial>, Transform)]
pub struct Cell {
    pub id: usize,
    pub neighbor_ids: Vec<usize>,
    pub is_mine: bool,
    pub state: CellState,
    pub adjacent_mines: u8,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
pub enum CellState {
    #[default]
    Hidden,
    Revealed,
    Flagged,
}

// --- SYSTEMS ---

pub fn process_reveal_queue(
    mut events: EventReader<RevealCell>,
    mut chord_events: EventReader<ChordCell>,
    mut commands: Commands,
    mut all_cells_q: Query<(Entity, &mut Cell)>,
    visuals: Res<CellVisuals>,
    mut session: ResMut<GameSession>,
    mut app_state: ResMut<NextState<AppState>>,
    _asset_server: Res<AssetServer>,
) {
    let mut queue: Vec<Entity> = events.read().map(|e| e.0).collect();
    
    // Process Chords
    let mut chord_targets = Vec::new();
    for chord in chord_events.read() {
        if let Ok((_, center_cell)) = all_cells_q.get(chord.0) {
            let mut flags = 0;
            let mut neighbors = Vec::new();
            
            for &nid in &center_cell.neighbor_ids {
                for (ne, nc) in all_cells_q.iter() {
                    if nc.id == nid {
                        if nc.state == CellState::Flagged { flags += 1; }
                        else if nc.state == CellState::Hidden { neighbors.push(ne); }
                    }
                }
            }
            
            if flags == center_cell.adjacent_mines {
                chord_targets.extend(neighbors);
            }
        }
    }
    queue.extend(chord_targets);

    let mut visited = HashSet::new();

    if !queue.is_empty() && session.total_mines == 0 {
        initialize_mines(&mut all_cells_q, queue[0], &mut session);
    }

    while let Some(entity) = queue.pop() {
        if visited.contains(&entity) { continue; }
        
        if let Ok((_, mut cell)) = all_cells_q.get_mut(entity) {
            if cell.state != CellState::Hidden { continue; }
            
            cell.state = CellState::Revealed;
            session.cells_revealed += 1;
            visited.insert(entity);

            let is_mine = cell.is_mine;
            let adj = cell.adjacent_mines;
            let neighbors = cell.neighbor_ids.clone();

            if is_mine {
                commands.entity(entity).insert(MeshMaterial3d(visuals.exploded.clone()));
                app_state.set(AppState::GameOver);
            } else {
                let mat = if adj > 0 && adj <= 8 {
                    visuals.adjacent[(adj - 1) as usize].clone()
                } else {
                    visuals.revealed.clone()
                };
                commands.entity(entity).insert(MeshMaterial3d(mat));
                
                if adj == 0 {
                    // Flood Fill
                    for nid in neighbors {
                        for (ne, nc) in all_cells_q.iter() {
                            if nc.id == nid && nc.state == CellState::Hidden {
                                queue.push(ne);
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn reveal_all_mines(
    mut commands: Commands,
    q_cells: Query<(Entity, &Cell)>, 
    visuals: Res<CellVisuals>,
) {
    for (e, cell) in &q_cells {
        if cell.is_mine && cell.state != CellState::Revealed {
             commands.entity(e).insert(MeshMaterial3d(visuals.mine.clone()));
        }
    }
}

pub fn initialize_mines(
    all_cells: &mut Query<(Entity, &mut Cell)>, 
    safe_entity: Entity,
    session: &mut GameSession,
) {
    let mut rng = thread_rng();
    let safe_id = all_cells.get(safe_entity).unwrap().1.id;
    let safe_neighbors = all_cells.get(safe_entity).unwrap().1.neighbor_ids.clone();
    
    let mut safe_zone = HashSet::new();
    safe_zone.insert(safe_id);
    for nid in safe_neighbors { safe_zone.insert(nid); }

    let mut targets: Vec<Entity> = all_cells.iter()
        .filter(|(_, c)| !safe_zone.contains(&c.id))
        .map(|(e, _)| e)
        .collect();
        
    targets.shuffle(&mut rng);
    
    // Scale difficulty
    let difficulty_mult = 1.0 + (session.level as f64 - 1.0) * 0.2; 
    let percentage = (BASE_MINE_PERCENTAGE * difficulty_mult).min(0.5);
    
    session.total_mines = (session.total_cells as f64 * percentage) as usize;
    
    let mines: HashSet<Entity> = targets.into_iter().take(session.total_mines).collect();
    let mut id_is_mine = HashMap::new();

    for (e, mut c) in all_cells.iter_mut() {
        c.is_mine = mines.contains(&e);
        id_is_mine.insert(c.id, c.is_mine);
    }

    for (_, mut c) in all_cells.iter_mut() {
        if !c.is_mine {
            c.adjacent_mines = c.neighbor_ids.iter()
                .filter(|&nid| *id_is_mine.get(nid).unwrap_or(&false))
                .count() as u8;
        }
    }
}

pub fn check_win_condition(session: Res<GameSession>, mut state: ResMut<NextState<AppState>>) {
    if session.total_mines > 0 && session.cells_revealed >= session.total_cells - session.total_mines {
        state.set(AppState::Victory);
    }
}

pub fn update_max_level(mut session: ResMut<GameSession>) {
    if session.level > session.max_level {
        session.max_level = session.level;
    }
}

pub fn save_game(session: Res<GameSession>) {
    if let Ok(json) = serde_json::to_string(&*session) {
        if let Ok(mut file) = fs::File::create("save.json") {
            let _ = file.write_all(json.as_bytes());
        }
    }
}

pub fn load_game() -> GameSession {
    if let Ok(contents) = fs::read_to_string("save.json") {
        if let Ok(session) = serde_json::from_str(&contents) {
            return session;
        }
    }
    GameSession::default()
}
