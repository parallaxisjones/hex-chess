use bevy::prelude::*;
use hex_chess_core::*;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
pub fn main() {
    // This provides better error messages in debug mode.
    // It's disabled in release mode so it doesn't bloat up the file size.
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    // Spawn the Bevy app
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Hexagonal Chess".into(),
                resolution: (1200.0, 800.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(HexChessPlugin)
        .run();
}

pub struct HexChessPlugin;

impl Plugin for HexChessPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_state::<GameState>()
            .add_systems(Startup, setup)
            .add_systems(Update, (
                handle_input,
                update_board_visuals,
                update_ui,
            ).run_if(in_state(GameState::Playing)))
            .add_systems(Update, (
                handle_menu_input,
            ).run_if(in_state(GameState::Menu)));
    }
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Menu,
    Playing,
    GameOver,
}

#[derive(Resource)]
pub struct GameData {
    pub game: hex_chess_core::Game,
    pub selected_piece: Option<HexCoord>,
    pub valid_moves: Vec<HexCoord>,
    pub camera_entity: Entity,
}

#[derive(Component)]
pub struct HexTile {
    pub coord: HexCoord,
}

#[derive(Component)]
pub struct ChessPiece {
    pub coord: HexCoord,
    pub piece: Piece,
}

#[derive(Component)]
pub struct MoveIndicator;

#[derive(Component)]
pub struct GameUI;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Create the game with default variant
    let variant = Variants::glinski_chess();
    let game = hex_chess_core::Game::new(variant);
    
    // Spawn camera
    let camera_entity = commands.spawn(Camera2dBundle::default()).id();
    
    // Store game data
    commands.insert_resource(GameData {
        game,
        selected_piece: None,
        valid_moves: Vec::new(),
        camera_entity,
    });
    
    // Spawn the board
    spawn_board(&mut commands, &mut meshes, &mut materials);
    
    // Spawn UI
    spawn_ui(&mut commands, &asset_server);
}

fn spawn_board(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    // Create hex tile materials
    let light_material = materials.add(Color::srgb(0.9, 0.9, 0.8));
    let medium_material = materials.add(Color::srgb(0.7, 0.7, 0.6));
    let dark_material = materials.add(Color::srgb(0.5, 0.5, 0.4));
    
    // Create hex mesh
    let hex_mesh = meshes.add(create_hex_mesh());
    
    // Spawn hex tiles
    for coord in HexCoord::new(0, 0).neighbors() {
        let material = match (coord.q + coord.r) % 3 {
            0 => light_material.clone(),
            1 => medium_material.clone(),
            _ => dark_material.clone(),
        };
        
        let (x, y) = coord.to_pixel();
        
        commands.spawn((
            PbrBundle {
                mesh: hex_mesh.clone(),
                material,
                transform: Transform::from_xyz(x * 0.8, y * 0.8, 0.0),
                ..default()
            },
            HexTile { coord },
        ));
    }
}

fn create_hex_mesh() -> Mesh {
    // Create a simple hexagon mesh
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    
    // Center vertex
    vertices.push([0.0, 0.0, 0.0]);
    
    // Hexagon vertices
    for i in 0..6 {
        let angle = i as f32 * std::f32::consts::PI / 3.0;
        let x = angle.cos() * 0.5;
        let y = angle.sin() * 0.5;
        vertices.push([x, y, 0.0]);
    }
    
    // Create triangles
    for i in 0..6 {
        indices.push(0);
        indices.push(i + 1);
        indices.push((i + 1) % 6 + 1);
    }
    
    Mesh::new(
        bevy::render::render_resource::PrimitiveTopology::TriangleList,
        bevy::render::render_asset::RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
    .with_inserted_indices(bevy::render::mesh::Indices::U32(indices))
}

fn spawn_ui(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(10.0),
                ..default()
            },
            background_color: Color::rgba(0.0, 0.0, 0.0, 0.7).into(),
            ..default()
        },
        GameUI,
    )).with_children(|parent| {
        parent.spawn(TextBundle::from_section(
            "Hexagonal Chess",
            TextStyle {
                font_size: 24.0,
                color: Color::WHITE,
                ..default()
            },
        ));
    });
}

fn handle_input(
    mut game_data: ResMut<GameData>,
    mut mouse_button_events: EventReader<MouseButtonInput>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    hex_tiles: Query<&HexTile>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for event in mouse_button_events.iter() {
        if event.button == MouseButton::Left && event.state.is_pressed() {
            if let Some(clicked_coord) = get_clicked_hex(&windows, &camera_query, &hex_tiles) {
                handle_hex_click(&mut game_data, clicked_coord, &mut commands, &mut meshes, &mut materials);
            }
        }
    }
}

fn get_clicked_hex(
    windows: &Query<&Window>,
    camera_query: &Query<(&Camera, &GlobalTransform)>,
    hex_tiles: &Query<&HexTile>,
) -> Option<HexCoord> {
    // TODO: Implement proper mouse-to-hex coordinate conversion
    // For now, return None
    None
}

fn handle_hex_click(
    game_data: &mut ResMut<GameData>,
    coord: HexCoord,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    if let Some(selected) = game_data.selected_piece {
        // Try to move the selected piece
        if game_data.valid_moves.contains(&coord) {
            if let Err(e) = game_data.game.make_move(selected, coord) {
                web_sys::console::log_1(&format!("Move error: {:?}", e).into());
            } else {
                game_data.selected_piece = None;
                game_data.valid_moves.clear();
            }
        }
    } else {
        // Select a piece
        if let Some(piece) = game_data.game.board.get_piece(coord) {
            if piece.color == game_data.game.current_player {
                game_data.selected_piece = Some(coord);
                game_data.valid_moves = game_data.game.board.get_valid_moves(coord);
            }
        }
    }
}

fn update_board_visuals(
    game_data: Res<GameData>,
    mut piece_query: Query<(&mut Transform, &ChessPiece)>,
) {
    // Update piece positions based on game state
    for (mut transform, chess_piece) in piece_query.iter_mut() {
        let (x, y) = chess_piece.coord.to_pixel();
        transform.translation = Vec3::new(x * 0.8, y * 0.8, 0.1);
    }
}

fn update_ui(
    game_data: Res<GameData>,
    mut ui_query: Query<&mut Text, With<GameUI>>,
) {
    if let Ok(mut text) = ui_query.get_single_mut() {
        let current_player = match game_data.game.current_player {
            Color::White => "White",
            Color::Black => "Black",
        };
        text.sections[0].value = format!("Hexagonal Chess - {} to move", current_player);
    }
}

fn handle_menu_input(
    mut game_state: ResMut<NextState<GameState>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        game_state.set(GameState::Playing);
    }
}
