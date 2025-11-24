use bevy::{
    color::palettes::css::*,
    input::mouse::MouseWheel,
    picking::mesh_picking::MeshPickingPlugin,
    prelude::*,
    render::{mesh::Indices, render_asset::RenderAssetUsages, render_resource::PrimitiveTopology},
};
use std::collections::{HashMap, HashSet};
use rand::prelude::*;

// --- CONFIGURATION ---
const SPHERE_RADIUS: f32 = 2.0;
const SUBDIVISIONS: usize = 2;
const BASE_MINE_PERCENTAGE: f64 = 0.15;

// --- STATE ---
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum AppState {
    #[default]
    Playing,
    GameOver,
    Victory,
}

#[derive(Resource)]
struct GameSettings {
    invert_y: bool,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self { invert_y: false }
    }
}

#[derive(Resource)]
struct GameSession {
    level: u32,
    is_first_click: bool,
    total_mines: usize,
    flags_placed: usize,
    cells_revealed: usize,
    total_cells: usize,
    start_time: Option<f64>,
}

impl Default for GameSession {
    fn default() -> Self {
        Self {
            level: 1,
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
struct RevealCell(Entity);

#[derive(Event)]
struct ChordCell(Entity);

// --- RESOURCES & COMPONENTS ---

#[derive(Resource, Clone, Default)]
struct CellVisuals {
    hidden: Handle<StandardMaterial>,
    flagged: Handle<StandardMaterial>,
    revealed: Handle<StandardMaterial>,
    mine: Handle<StandardMaterial>,
    exploded: Handle<StandardMaterial>,
    hovered: Handle<StandardMaterial>,
    adjacent: Vec<Handle<StandardMaterial>>,
}

#[derive(Component)]
#[require(Mesh3d, MeshMaterial3d<StandardMaterial>, Transform)]
struct Cell {
    id: usize,
    neighbor_ids: Vec<usize>,
    is_mine: bool,
    state: CellState,
    adjacent_mines: u8,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
enum CellState {
    #[default]
    Hidden,
    Revealed,
    Flagged,
}

#[derive(Component)]
struct HudText;

#[derive(Component)]
struct RestartMenu; // Marker for the menu root

#[derive(Component)]
struct RestartButton; // Marker for the button

#[derive(Component)]
struct InvertYButton;

#[derive(Component)]
struct InvertYText;

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
        .init_resource::<GameSession>()
        .init_resource::<GameSettings>()
        .init_resource::<CellVisuals>() // Initialized in load_assets
        .add_event::<RevealCell>() 
        .add_event::<ChordCell>()
        .add_systems(PreStartup, load_assets)
        .add_systems(Startup, (setup_scene, setup_ui))
        .add_systems(OnEnter(AppState::Playing), spawn_board)
        .add_systems(Update, (
            update_hud,
            camera_orbit_controls,
            check_win_condition,
            toggle_invert_y,
        ).run_if(in_state(AppState::Playing)))
        .add_systems(Update, process_reveal_queue.run_if(in_state(AppState::Playing)))
        
        // Game Over / Victory Logic
        .add_systems(OnEnter(AppState::GameOver), (reveal_all_mines, setup_menu))
        .add_systems(OnEnter(AppState::Victory), setup_menu)
        .add_systems(Update, menu_interaction.run_if(in_state(AppState::GameOver).or(in_state(AppState::Victory))))
        .add_systems(OnExit(AppState::GameOver), (cleanup_board, cleanup_menu))
        .add_systems(OnExit(AppState::Victory), (cleanup_board, cleanup_menu))
        
        .run();
}

// --- SETUP & ASSETS ---

fn setup_scene(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 6.5).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        PointLight {
            shadows_enabled: true,
            intensity: 2_000_000.0,
            range: 100.0,
            ..default()
        },
        Transform::from_xyz(10.0, 10.0, 10.0),
    ));
    
    commands.spawn((
        PointLight {
            intensity: 500_000.0,
            ..default()
        },
        Transform::from_xyz(-10.0, -10.0, -5.0),
    ));
}

fn setup_ui(mut commands: Commands) {
    let font = TextFont {
        font_size: 20.0,
        ..default()
    };

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(20.0),
            left: Val::Px(20.0),
            ..default()
        },
        Text::new("Mines: 0 | Time: 0"),
        font.clone(),
        TextColor(WHITE.into()),
        HudText,
    ));

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(20.0),
            left: Val::Px(20.0),
            ..default()
        },
        Text::new("L-Click: Reveal | R-Click: Flag | Double-Click: Chord\nScroll: Zoom | Drag: Rotate"),
        font.clone(),
        TextColor(SILVER.into()),
    ));

    // Invert Y Toggle
    commands.spawn((
        Button,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(20.0),
            right: Val::Px(20.0),
            padding: UiRect::all(Val::Px(10.0)),
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BorderColor(WHITE.into()),
        BackgroundColor(Color::Srgba(Srgba::gray(0.2))),
        InvertYButton,
    ))
    .with_children(|parent| {
        parent.spawn((
            Text::new("Invert Y: Off"),
            font,
            TextColor(WHITE.into()),
            InvertYText,
        ));
    });
}

fn load_assets(
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut visuals: ResMut<CellVisuals>,
) {
    let adj_colors = [
        AQUA, LIME, RED, BLUE, MAGENTA, YELLOW, WHITE, BLACK,
    ];

    *visuals = CellVisuals {
        hidden: materials.add(StandardMaterial {
            base_color: Srgba::rgb(0.1, 0.1, 0.15).into(), // Dark Blue-Grey
            perceptual_roughness: 0.8,
            ..default()
        }),
        flagged: materials.add(StandardMaterial::from(Color::from(ORANGE))),
        revealed: materials.add(StandardMaterial {
            base_color: Srgba::rgb(0.9, 0.9, 0.9).into(), // Off-white
            perceptual_roughness: 0.8,
            ..default()
        }),
        mine: materials.add(StandardMaterial {
            base_color: BLACK.into(),
            perceptual_roughness: 0.1,
            ..default()
        }),
        exploded: materials.add(StandardMaterial {
            base_color: RED.into(),
            emissive: LinearRgba::new(2.0, 0.0, 0.0, 1.0),
            ..default()
        }),
        hovered: materials.add(StandardMaterial {
            base_color: Srgba::rgb(0.15, 0.15, 0.25).into(), // Slightly lighter
            perceptual_roughness: 0.8,
            ..default()
        }),
        adjacent: adj_colors
            .iter()
            .map(|c| {
                materials.add(StandardMaterial {
                    base_color: Color::from(*c).into(),
                    perceptual_roughness: 0.8,
                    ..default()
                })
            })
            .collect(),
    };
}

// --- BOARD MANAGEMENT ---

fn spawn_board(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    visuals: Res<CellVisuals>,
    mut session: ResMut<GameSession>,
) {
    // Reset session "per game" stats
    session.is_first_click = true;
    session.flags_placed = 0;
    session.cells_revealed = 0;
    session.start_time = None;
    session.total_mines = 0; // Will be set in initialize_mines

    let radius = SPHERE_RADIUS + (session.level as f32 - 1.0) * 0.5;
    let subdivisions = if session.level < 3 { 2 } else if session.level < 6 { 3 } else { 4 };
    let (polygons, adjacency) = generate_goldberg_polyhedron(radius, subdivisions);
    session.total_cells = polygons.len();

    for (idx, poly) in polygons.iter().enumerate() {
        let mesh = create_polygon_mesh(poly);
        
        commands.spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(visuals.hidden.clone()),
            Transform::default(),
            Cell {
                id: idx,
                neighbor_ids: adjacency[idx].clone(),
                is_mine: false,
                state: CellState::Hidden,
                adjacent_mines: 0,
            },
        ))
        .observe(on_cell_click)
        .observe(on_cell_over)
        .observe(on_cell_out);
    }
}

fn cleanup_board(mut commands: Commands, q_cells: Query<Entity, With<Cell>>) {
    for entity in &q_cells {
        commands.entity(entity).despawn_recursive();
    }
}

// --- UI MENUS ---

fn setup_menu(
    mut commands: Commands,
    state: Res<State<AppState>>,
) {
    let (text, color) = match state.get() {
        AppState::Victory => ("Next Level", GREEN),
        _ => ("Restart", RED),
    };

    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            position_type: PositionType::Absolute,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
        RestartMenu,
    ))
    .with_children(|parent| {
        parent.spawn(( 
            Button,
            Node {
                width: Val::Px(200.0),
                height: Val::Px(80.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::Srgba(Srgba::gray(0.2))),
            RestartButton,
        ))
        .with_children(|parent| {
            parent.spawn(( 
                Text::new(text),
                TextFont {
                    font_size: 40.0,
                    ..default()
                },
                TextColor(color.into()),
            ));
        });
    });
}

fn menu_interaction(
    mut interaction_query: Query< 
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<RestartButton>),
    >,
    mut app_state: ResMut<NextState<AppState>>,
    mut session: ResMut<GameSession>,
    state: Res<State<AppState>>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                if *state.get() == AppState::Victory {
                    session.level += 1;
                } else {
                    // Don't reset level on death
                }
                app_state.set(AppState::Playing);
            }
            Interaction::Hovered => *color = Color::Srgba(Srgba::gray(0.3)).into(),
            Interaction::None => *color = Color::Srgba(Srgba::gray(0.2)).into(),
        }
    }
}

fn cleanup_menu(mut commands: Commands, q_menu: Query<Entity, With<RestartMenu>>) {
    for entity in &q_menu {
        commands.entity(entity).despawn_recursive();
    }
}

// --- OBSERVERS ---

fn on_cell_click(
    trigger: Trigger<Pointer<Click>>,
    mut q_cell: Query<(&mut Cell, &mut MeshMaterial3d<StandardMaterial>)>, 
    visuals: Res<CellVisuals>,
    mut session: ResMut<GameSession>,
    mut reveal_writer: EventWriter<RevealCell>,
    mut chord_writer: EventWriter<ChordCell>,
    time: Res<Time>,
) {
    let entity = trigger.target;
    let event = trigger.event();
    
    if let Ok((mut cell, mut mat)) = q_cell.get_mut(entity) {
        match event.button {
            PointerButton::Primary => {
                if cell.state == CellState::Hidden {
                    if session.is_first_click {
                        session.is_first_click = false;
                        session.start_time = Some(time.elapsed_secs_f64());
                    }
                    reveal_writer.write(RevealCell(entity));
                } else if cell.state == CellState::Revealed {
                    chord_writer.write(ChordCell(entity));
                }
            }
            PointerButton::Secondary => {
                if cell.state == CellState::Hidden {
                    cell.state = CellState::Flagged;
                    mat.0 = visuals.flagged.clone();
                    session.flags_placed += 1;
                } else if cell.state == CellState::Flagged {
                    cell.state = CellState::Hidden;
                    mat.0 = visuals.hovered.clone();
                    session.flags_placed -= 1;
                }
            }
            _ => {}
        }
    }
}

fn on_cell_over(
    trigger: Trigger<Pointer<Over>>,
    mut q_cell: Query<(&Cell, &mut MeshMaterial3d<StandardMaterial>)>, 
    visuals: Res<CellVisuals>,
) {
    let entity = trigger.target;
    if let Ok((cell, mut mat)) = q_cell.get_mut(entity) {
        if cell.state == CellState::Hidden {
            mat.0 = visuals.hovered.clone();
        }
    }
}

fn on_cell_out(
    trigger: Trigger<Pointer<Out>>,
    mut q_cell: Query<(&Cell, &mut MeshMaterial3d<StandardMaterial>)>, 
    visuals: Res<CellVisuals>,
) {
    let entity = trigger.target;
    if let Ok((cell, mut mat)) = q_cell.get_mut(entity) {
        if cell.state == CellState::Hidden {
            mat.0 = visuals.hidden.clone();
        }
    }
}

// --- GAMEPLAY SYSTEMS ---

fn process_reveal_queue(
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

fn reveal_all_mines(
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

fn initialize_mines(
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



fn update_hud(
    mut text_q: Query<(&mut Text, &mut TextColor), With<HudText>>,
    session: Res<GameSession>,
    state: Res<State<AppState>>,
    time: Res<Time>,
) {
    if let Ok((mut text, mut color)) = text_q.single_mut() {
        let elapsed = session.start_time.map_or(0.0, |t| time.elapsed_secs_f64() - t);
        let msg = match state.get() {
            AppState::GameOver => "GAME OVER",
            AppState::Victory => "VICTORY!",
            _ => "",
        };
        let mines_left = (session.total_mines as i32) - (session.flags_placed as i32);
        
        **text = format!("Lvl: {} | Mines: {} | Time: {:.0}  {}", session.level, mines_left, elapsed, msg);
        
        match state.get() {
            AppState::GameOver => color.0 = RED.into(),
            AppState::Victory => color.0 = GREEN.into(),
            _ => color.0 = WHITE.into(),
        }
    }
}

fn check_win_condition(session: Res<GameSession>, mut state: ResMut<NextState<AppState>>) {
    if session.total_mines > 0 && session.cells_revealed >= session.total_cells - session.total_mines {
        state.set(AppState::Victory);
    }
}

fn camera_orbit_controls(
    mut q_cam: Query<&mut Transform, With<Camera>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut motion: EventReader<bevy::input::mouse::MouseMotion>,
    mut scroll: EventReader<MouseWheel>,
    settings: Res<GameSettings>,
    session: Res<GameSession>,
) {
    if let Ok(mut transform) = q_cam.single_mut() {
        if mouse.pressed(MouseButton::Right) {
            for ev in motion.read() {
                let delta = ev.delta * 0.002; // Slower rotation
                
                // Yaw: Rotate around Global Y
                let q_yaw = Quat::from_rotation_y(-delta.x);
                transform.rotate_around(Vec3::ZERO, q_yaw);

                // Pitch: Rotate around Local X (Right vector)
                let right = *transform.right(); 
                let y_mult = if settings.invert_y { -1.0 } else { 1.0 };
                let q_pitch = Quat::from_axis_angle(right, -delta.y * y_mult);
                transform.rotate_around(Vec3::ZERO, q_pitch);
            }
        }

        let radius = SPHERE_RADIUS + (session.level as f32 - 1.0) * 0.5;
        let min_dist = radius * 1.2;
        let max_dist = radius * 6.0;

        for ev in scroll.read() {
            let dist = transform.translation.length();
            let new_dist = (dist - ev.y * 0.5).clamp(min_dist, max_dist);
            transform.translation = transform.translation.normalize() * new_dist;
        }
    }
}

fn toggle_invert_y(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<InvertYButton>),
    >,
    mut text_query: Query<&mut Text, With<InvertYText>>,
    mut settings: ResMut<GameSettings>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                settings.invert_y = !settings.invert_y;
                if let Ok(mut text) = text_query.get_single_mut() {
                    **text = format!("Invert Y: {}", if settings.invert_y { "On" } else { "Off" });
                }
            }
            Interaction::Hovered => *color = Color::Srgba(Srgba::gray(0.3)).into(),
            Interaction::None => *color = Color::Srgba(Srgba::gray(0.2)).into(),
        }
    }
}

// --- GEOMETRY UTILS ---

fn generate_goldberg_polyhedron(radius: f32, subdivisions: usize) -> (Vec<Vec<Vec3>>, Vec<Vec<usize>>) {
    let t = (1.0 + 5.0f32.sqrt()) / 2.0;
    let mut verts = vec![
        Vec3::new(-1.0, t, 0.0), Vec3::new(1.0, t, 0.0), Vec3::new(-1.0, -t, 0.0), Vec3::new(1.0, -t, 0.0),
        Vec3::new(0.0, -1.0, t), Vec3::new(0.0, 1.0, t), Vec3::new(0.0, -1.0, -t), Vec3::new(0.0, 1.0, -t),
        Vec3::new(t, 0.0, -1.0), Vec3::new(t, 0.0, 1.0), Vec3::new(-t, 0.0, -1.0), Vec3::new(-t, 0.0, 1.0),
    ];
    for v in &mut verts { *v = v.normalize(); }

    let mut faces = vec![
        vec![0, 11, 5], vec![0, 5, 1], vec![0, 1, 7], vec![0, 7, 10], vec![0, 10, 11],
        vec![1, 5, 9], vec![5, 11, 4], vec![11, 10, 2], vec![10, 7, 6], vec![7, 1, 8],
        vec![3, 9, 4], vec![3, 4, 2], vec![3, 2, 6], vec![3, 6, 8], vec![3, 8, 9],
        vec![4, 9, 5], vec![2, 4, 11], vec![6, 2, 10], vec![8, 6, 7], vec![9, 8, 1],
    ];

    for _ in 0..subdivisions {
        let mut next_faces = Vec::new();
        let mut mid_cache = HashMap::new();
        for f in faces {
            let (v1, v2, v3) = (f[0], f[1], f[2]);
            let a = get_midpoint(v1, v2, &mut verts, &mut mid_cache);
            let b = get_midpoint(v2, v3, &mut verts, &mut mid_cache);
            let c = get_midpoint(v3, v1, &mut verts, &mut mid_cache);
            next_faces.extend_from_slice(&[vec![v1, a, c], vec![v2, b, a], vec![v3, c, b], vec![a, b, c]]);
        }
        faces = next_faces;
    }
    
    let mut poly_map: HashMap<usize, Vec<usize>> = HashMap::new();
    let mut centers = Vec::new();
    for (i, f) in faces.iter().enumerate() {
        centers.push(((verts[f[0]] + verts[f[1]] + verts[f[2]]) / 3.0).normalize() * radius);
        for &v in f { poly_map.entry(v).or_default().push(i); }
    }

    let mut polygons = Vec::new();
    let mut adjacency = Vec::new();

    for i in 0..verts.len() {
        if let Some(indices) = poly_map.get(&i) {
            let center = verts[i];
            let up = center.normalize();
            let mut sorted = indices.clone();
            sorted.sort_by(|&a, &b| {
                let pa = centers[a] - center * radius;
                let pb = centers[b] - center * radius;
                let tan = if up.y.abs() > 0.9 { Vec3::X } else { Vec3::Y }.cross(up).normalize();
                let bitan = up.cross(tan);
                pa.dot(tan).atan2(pa.dot(bitan)).partial_cmp(&pb.dot(tan).atan2(pb.dot(bitan))).unwrap()
            });
            polygons.push(sorted.iter().map(|&idx| centers[idx]).collect());
            
            let mut neighbors = HashSet::new();
            for &fi in &sorted {
                for &v in &faces[fi] {
                    if v != i { neighbors.insert(v); }
                }
            }
            adjacency.push(neighbors.into_iter().collect());
        }
    }
    (polygons, adjacency)
}

fn get_midpoint(p1: usize, p2: usize, verts: &mut Vec<Vec3>, cache: &mut HashMap<(usize, usize), usize>) -> usize {
    let key = if p1 < p2 { (p1, p2) } else { (p2, p1) };
    if let Some(&idx) = cache.get(&key) { return idx; }
    verts.push(((verts[p1] + verts[p2]) * 0.5).normalize());
    cache.insert(key, verts.len() - 1);
    verts.len() - 1
}

fn create_polygon_mesh(verts: &Vec<Vec3>) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    let center = verts.iter().sum::<Vec3>() / verts.len() as f32;
    let mut pos: Vec<[f32; 3]> = vec![center.into()];
    let mut norm: Vec<[f32; 3]> = vec![center.normalize().into()];
    let mut idxs = Vec::new();

    for (i, v) in verts.iter().enumerate() {
        let v_gap = center + (*v - center) * 0.92;
        pos.push(v_gap.into());
        norm.push(v_gap.normalize().into());
        let next = (i + 1) % verts.len();
        idxs.extend_from_slice(&[0, (next + 1) as u32, (i + 1) as u32]);
    }
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, pos);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, norm);
    mesh.insert_indices(Indices::U32(idxs));
    mesh
}
