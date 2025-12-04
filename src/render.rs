use bevy::{
    color::palettes::css::*,
    input::mouse::MouseWheel,
    prelude::*,
    core_pipeline::bloom::Bloom,
    render::{mesh::Indices, render_asset::RenderAssetUsages, render_resource::PrimitiveTopology},
};
use rand::prelude::*;
use crate::game::*;
use crate::utils::*;

// --- RESOURCES & COMPONENTS ---

#[derive(Resource, Clone, Default)]
pub struct CellVisuals {
    pub hidden: Handle<StandardMaterial>,
    pub flagged: Handle<StandardMaterial>,
    pub revealed: Handle<StandardMaterial>,
    pub mine: Handle<StandardMaterial>,
    pub exploded: Handle<StandardMaterial>,
    pub hovered: Handle<StandardMaterial>,
    pub adjacent: Vec<Handle<StandardMaterial>>,
}

#[derive(Component)]
pub struct HudText;

#[derive(Component)]
pub struct RestartMenu; // Marker for the menu root

#[derive(Component)]
pub struct RestartButton; // Marker for the button

#[derive(Component)]
pub struct InvertYButton;

#[derive(Component)]
pub struct InvertYText;

#[derive(Component)]
pub struct PrevLevelButton;

#[derive(Component)]
pub struct NextLevelButton;

#[derive(Component)]
pub struct LevelSelectText;

#[derive(Component)]
pub struct GameUi;

// --- SYSTEMS ---

pub fn setup_scene(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Camera {
            hdr: true, // HDR is required for bloom
            ..default()
        },
        Projection::Perspective(PerspectiveProjection {
            fov: 30.0_f32.to_radians(),
            ..default()
        }),
        Transform::from_xyz(0.0, 0.0, 12.0).looking_at(Vec3::ZERO, Vec3::Y),
        Bloom::NATURAL,
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

pub fn setup_ui(mut commands: Commands) {
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
        GameUi,
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
        GameUi,
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
        GameUi,
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

pub fn cleanup_ui(mut commands: Commands, q_ui: Query<Entity, With<GameUi>>) {
    for entity in &q_ui {
        commands.entity(entity).despawn();
    }
}


pub fn load_assets(
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut visuals: ResMut<CellVisuals>,
    mut state: ResMut<NextState<AppState>>,
) {
    info!("Loading assets...");
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
    state.set(AppState::MainMenu);
}

pub fn spawn_board(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    visuals: Res<CellVisuals>,
    mut session: ResMut<GameSession>,
    mut q_cam: Query<&mut Transform, With<Camera>>,
) {
    info!("Spawning board...");
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
    info!("Level: {}, Radius: {:.1}, Subdivisions: {}, Cells: {}", session.level, radius, subdivisions, session.total_cells);

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

    // Adjust Camera Distance to fit the sphere
    let fov_y = 30.0_f32.to_radians();
    let distance = (radius * 1.5) / (fov_y / 2.0).tan(); // 1.5 margin for better framing
    
    if let Ok(mut cam_transform) = q_cam.single_mut() {
        *cam_transform = Transform::from_xyz(0.0, 0.0, distance).looking_at(Vec3::ZERO, Vec3::Y);
    }
}

pub fn cleanup_board(mut commands: Commands, q_cells: Query<Entity, With<Cell>>) {
    for entity in &q_cells {
        commands.entity(entity).despawn();
    }
}

pub fn setup_menu(
    mut commands: Commands,
    state: Res<State<AppState>>,
    session: Res<GameSession>,
) {
    let (text, color) = match state.get() {
        AppState::Victory => ("Next Level", GREEN),
        AppState::MainMenu => ("Start Game", BLUE),
        _ => ("Restart", RED),
    };

    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            position_type: PositionType::Absolute,
            row_gap: Val::Px(20.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
        RestartMenu,
    ))
    .with_children(|parent| {
        // Level Selection Row
        parent.spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(20.0),
            ..default()
        }).with_children(|row| {
             // Prev Button
             row.spawn((
                Button,
                Node {
                    width: Val::Px(40.0),
                    height: Val::Px(40.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::Srgba(Srgba::gray(0.2))),
                PrevLevelButton,
             )).with_children(|btn| {
                 btn.spawn((Text::new("<"), TextColor(WHITE.into())));
             });

             // Level Text
             row.spawn((
                 Text::new(format!("Level {}", session.level)),
                 TextFont { font_size: 30.0, ..default() },
                 TextColor(WHITE.into()),
                 LevelSelectText,
             ));

             // Next Button
             row.spawn((
                Button,
                Node {
                    width: Val::Px(40.0),
                    height: Val::Px(40.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::Srgba(Srgba::gray(0.2))),
                NextLevelButton,
             )).with_children(|btn| {
                 btn.spawn((Text::new(">"), TextColor(WHITE.into())));
             });
        });

        // Restart/Next Action Button
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

pub fn menu_interaction(
    mut interaction_query: Query< 
        (&Interaction, &mut BackgroundColor, Option<&RestartButton>, Option<&PrevLevelButton>, Option<&NextLevelButton>),
        (Changed<Interaction>, With<Button>),
    >,
    mut app_state: ResMut<NextState<AppState>>,
    mut session: ResMut<GameSession>,
    state: Res<State<AppState>>,
    mut txt_q: Query<&mut Text, With<LevelSelectText>>,
) {
    for (interaction, mut color, restart, prev, next) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                if restart.is_some() {
                    if *state.get() == AppState::Victory {
                        session.level += 1;
                    } 
                    app_state.set(AppState::Playing);
                } else if prev.is_some() {
                    if session.level > 1 {
                        session.level -= 1;
                    }
                } else if next.is_some() {
                    if session.level < session.max_level {
                        session.level += 1;
                    }
                }
                
                // Update text
                if let Ok(mut txt) = txt_q.single_mut() {
                    **txt = format!("Level {}", session.level);
                }
            }
            Interaction::Hovered => *color = Color::Srgba(Srgba::gray(0.3)).into(),
            Interaction::None => *color = Color::Srgba(Srgba::gray(0.2)).into(),
        }
    }
}

pub fn cleanup_menu(mut commands: Commands, q_menu: Query<Entity, With<RestartMenu>>) {
    for entity in &q_menu {
        commands.entity(entity).despawn();
    }
}

pub fn on_cell_click(
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

pub fn on_cell_over(
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

pub fn on_cell_out(
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

pub fn update_hud(
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

pub fn camera_orbit_controls(
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
                let delta = ev.delta * 0.002;
                
                // Trackball / Free Orbit:
                // Rotate around Camera's Local Up and Right vectors to avoid Gimbal lock at poles.
                let right = *transform.right();
                let up = *transform.up();
                
                let y_mult = if settings.invert_y { -1.0 } else { 1.0 };
                
                // Yaw: Rotate around Camera Up
                let q_yaw = Quat::from_axis_angle(up, -delta.x);
                
                // Pitch: Rotate around Camera Right
                let q_pitch = Quat::from_axis_angle(right, -delta.y * y_mult);
                
                let rotation = q_yaw * q_pitch;
                
                // Apply rotation to position (orbit around center)
                transform.translation = rotation * transform.translation;
                
                // Apply rotation to camera orientation (look at center)
                transform.rotate(rotation);
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

pub fn toggle_invert_y(
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
                if let Ok(mut text) = text_query.single_mut() {
                    **text = format!("Invert Y: {}", if settings.invert_y { "On" } else { "Off" });
                }
            }
            Interaction::Hovered => *color = Color::Srgba(Srgba::gray(0.3)).into(),
            Interaction::None => *color = Color::Srgba(Srgba::gray(0.2)).into(),
        }
    }
}

pub fn setup_stars(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    let mut rng = rand::thread_rng();
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    
    let count = 2000;
    let radius = 80.0;
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(count);
    let mut indices = Vec::new();

    // Create simple quad particles for stars
    for i in 0..count {
        let theta = rng.gen_range(0.0..std::f32::consts::TAU);
        let phi = rng.gen_range(0.0..std::f32::consts::PI);
        let r = radius + rng.gen_range(-10.0..20.0);

        let x = r * phi.sin() * theta.cos();
        let y = r * phi.sin() * theta.sin();
        let z = r * phi.cos();
        let center = Vec3::new(x, y, z);

        // Make a tiny quad
        let size = rng.gen_range(0.05..0.15);
        let v0 = center + Vec3::new(-size, -size, 0.0);
        let v1 = center + Vec3::new(size, -size, 0.0);
        let v2 = center + Vec3::new(size, size, 0.0);
        let v3 = center + Vec3::new(-size, size, 0.0);

        let base = (i * 4) as u32;
        positions.push(v0.into());
        positions.push(v1.into());
        positions.push(v2.into());
        positions.push(v3.into());

        indices.extend_from_slice(&[base, base + 1, base + 2, base + 2, base + 3, base]);
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_indices(Indices::U32(indices));

    commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: WHITE.into(),
            emissive: LinearRgba::new(5.0, 5.0, 5.0, 1.0), // High intensity for bloom
            unlit: true,
            ..default()
        })),
        Transform::default(),
    ));
}

pub fn setup_planets(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    let mut _rng = rand::thread_rng();
    
    // Planet 1: Gas Giantish
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(8.0).mesh().uv(32, 18))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Srgba::rgb(0.8, 0.4, 0.1).into(),
            perceptual_roughness: 0.8,
            ..default()
        })),
        Transform::from_xyz(-40.0, 20.0, -50.0),
    ));

    // Planet 2: Icy
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(4.0).mesh().uv(32, 18))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Srgba::rgb(0.6, 0.8, 1.0).into(),
            perceptual_roughness: 0.2,
            ..default()
        })),
        Transform::from_xyz(50.0, -10.0, -40.0),
    ));
}
