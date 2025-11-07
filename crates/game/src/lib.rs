use bevy::prelude::*;
use bevy::input::mouse::MouseWheel;
use bevy::sprite::{MaterialMesh2dBundle, ColorMaterial};
use hex_chess_core::{HexCoord, Piece, PieceType, Variants, Color as ChessColor, CellColor};
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
// Note: wee_alloc feature is not currently enabled in Cargo.toml
// #[cfg(feature = "wee_alloc")]
// #[global_allocator]
// static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
pub fn main() {
    // This provides better error messages in both debug and release modes
    console_error_panic_hook::set_once();

    // Spawn the Bevy app
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Hexagonal Chess".into(),
                resolution: (1200.0, 800.0).into(),
                resizable: true,
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
            .init_state::<GameState>()
            .insert_state(GameState::Playing) // Start in Playing state
            .init_resource::<CapturedPieces>()
            .add_systems(Startup, setup)
            .add_systems(Update, (
                handle_input,
                handle_camera_zoom,
                handle_camera_pan,
                update_board_visuals,
                update_ui,
                update_captured_pieces_display,
                update_selection_visuals, // Show selected piece and valid moves
            ).run_if(in_state(GameState::Playing)))
            .add_systems(Update, (
                handle_menu_input,
            ).run_if(in_state(GameState::Menu)))
            .add_systems(Update, handle_menu_toggle); // Menu toggle works in all states
    }
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    Menu,
    #[default]
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

impl GameData {
    pub fn variant(&self) -> &hex_chess_core::VariantConfig {
        &self.game.variant
    }
}

#[derive(Resource)]
pub struct ValidMoveColor {
    pub color: Color,
}

#[derive(Resource, Default)]
pub struct CapturedPieces {
    pub white: Vec<Piece>, // White pieces that were captured (lost by White)
    pub black: Vec<Piece>, // Black pieces that were captured (lost by Black)
}

impl CapturedPieces {
    pub fn add(&mut self, piece: Piece) {
        match piece.color {
            ChessColor::White => self.white.push(piece),
            ChessColor::Black => self.black.push(piece),
        }
    }
}

#[derive(Component)]
pub struct HexTile {
    pub coord: HexCoord,
    pub base_color: Color, // Store original color for 2D
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

#[derive(Component)]
pub struct CapturedPiecesUI {
    pub color: ChessColor,
}

#[derive(Component)]
pub struct CoordinateLabel;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Note: meshes and materials are kept for potential future use, but we're using SpriteBundle for 2D
    // Create the game with default variant
    let variant = Variants::glinski_chess();
    let game = hex_chess_core::Game::new(variant);
    
    // Store game data temporarily to access board
    let game_data = GameData {
        game,
        selected_piece: None,
        valid_moves: Vec::new(),
        camera_entity: Entity::PLACEHOLDER, // Will be set after spawning
    };
    
    // Spawn 2D camera - centered on the board
    // The board spans roughly -200 to 200 in both x and y with BOARD_SCALE=100.0
    // Adjust camera scale to fit the board properly
    // Default Camera2dBundle scale=1.0 means 1 pixel = 1 world unit
    // With BOARD_SCALE=100.0, board is about 400 units wide, so we need to scale camera to see it
    let camera_entity = commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 1000.0), // 2D camera uses z for depth
        projection: OrthographicProjection {
            scale: 1.2, // Default zoom level for comfortable viewing
            ..default()
        }.into(),
        ..default()
    }).id();
    
    // Debug: log camera setup
    let msg = wasm_bindgen::JsValue::from_str("2D Camera spawned");
    unsafe {
        web_sys::console::log_1(&msg);
    }
    
    // Update game data with camera entity
    let game_data = GameData {
        camera_entity,
        ..game_data
    };
    
    // Spawn the board first (needs game_data to know which tiles to spawn)
    spawn_board(&mut commands, &mut meshes, &mut materials, &game_data, &asset_server);
    
    // Spawn coordinate labels around the perimeter
    spawn_coordinate_labels(&mut commands, &game_data);
    
    // Store game data resource after spawning board
    commands.insert_resource(game_data);
    
    // Create and cache valid move color (green highlight)
    commands.insert_resource(ValidMoveColor {
        color: bevy::prelude::Color::srgb(0.2, 0.8, 0.3),
    });
    
    // Spawn UI
    spawn_ui(&mut commands, &asset_server);
    
    // Spawn captured pieces display areas
    spawn_captured_pieces_areas(&mut commands);
}

fn spawn_board(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    game_data: &GameData,
    _asset_server: &Res<AssetServer>,
) {
    // Create hex tile colors - simple, high-contrast colors for 2D
    // Light squares: beige (#F5F5DC)
    let light_color = bevy::prelude::Color::srgb(0.96, 0.96, 0.86);
    
    // Medium squares: brown (#8B7355)
    let medium_color = bevy::prelude::Color::srgb(0.55, 0.45, 0.33);
    
    // Dark squares: dark brown (#654321)
    let dark_color = bevy::prelude::Color::srgb(0.40, 0.26, 0.13);
    
    // Debug: log cell colors availability
    let cell_colors_count = game_data.game.board.cell_colors.len();
    let valid_coords_count = game_data.game.board.valid_coords.len();
    let msg = wasm_bindgen::JsValue::from_str(&format!("Board has {} valid coords, {} cell colors defined", valid_coords_count, cell_colors_count));
    unsafe {
        web_sys::console::log_1(&msg);
    }
    
    // Spawn hex tiles for all valid coordinates on the board
    let mut light_count = 0;
    let mut dark_count = 0;
    let mut medium_count = 0;
    
    for &coord in &game_data.game.board.valid_coords {
        // Get cell color from board, with fallback to checkerboard pattern
        let (base_color, color_name) = match game_data.game.board.cell_colors.get(&coord) {
            Some(CellColor::Light) => {
                light_count += 1;
                (light_color, "Light")
            }
            Some(CellColor::Medium) => {
                medium_count += 1;
                (medium_color, "Medium")
            }
            Some(CellColor::Dark) => {
                dark_count += 1;
                (dark_color, "Dark")
            }
            None => {
                // Fallback to checkerboard pattern based on hex coordinates
                let (q, r, _s) = coord.to_cube();
                if (q + r) % 2 == 0 {
                    light_count += 1;
                    (light_color, "Light (fallback)")
                } else {
                    dark_count += 1;
                    (dark_color, "Dark (fallback)")
                }
            }
        };
        
        let (x, y) = coord.to_pixel();
        const BOARD_SCALE: f32 = 100.0; // Scale for 2D visibility - increased for larger board
        
        // Debug: log a few tile positions and their colors
        if coord == HexCoord::new(0, 0) || coord == HexCoord::new(-2, 3) || coord == HexCoord::new(3, -4) {
            let msg = wasm_bindgen::JsValue::from_str(&format!("Tile at {:?} -> {} color -> pixel ({:.2}, {:.2}) -> world ({:.2}, {:.2})", coord, color_name, x, y, x * BOARD_SCALE, y * BOARD_SCALE));
            unsafe {
                web_sys::console::log_1(&msg);
            }
        }
        
        // Use MaterialMesh2dBundle for hexagonal tiles
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: meshes.add(RegularPolygon::new(BOARD_SCALE * 0.45, 6)).into(),
                material: materials.add(ColorMaterial::from(base_color)),
                transform: Transform::from_xyz(x * BOARD_SCALE, y * BOARD_SCALE, 0.0),
                ..default()
            },
            HexTile { coord, base_color },
        ));
    }
    
    // Debug: log tile color distribution
    let msg = wasm_bindgen::JsValue::from_str(&format!("Tile colors: {} light, {} dark, {} medium", light_count, dark_count, medium_count));
    unsafe {
        web_sys::console::log_1(&msg);
    }
    
    // Spawn chess pieces
    let piece_count = game_data.game.board.pieces.len();
    let msg = wasm_bindgen::JsValue::from_str(&format!("Spawning {} pieces", piece_count));
    unsafe {
        web_sys::console::log_1(&msg);
    }
    
    const BOARD_SCALE: f32 = 100.0; // Match tile scaling - increased for larger board
    for (&coord, &piece) in &game_data.game.board.pieces {
        let (x, y) = coord.to_pixel();
        let world_x = x * BOARD_SCALE;
        let world_y = y * BOARD_SCALE;
        
        let msg = wasm_bindgen::JsValue::from_str(&format!("Spawning piece {:?} at {:?} (pixel: {:.2}, {:.2}) -> world ({:.2}, {:.2})", piece, coord, x, y, world_x, world_y));
        unsafe {
            web_sys::console::log_1(&msg);
        }
        
        // Create distinct piece colors and shapes for 2D
        let (piece_color, piece_label, piece_size) = match (piece.color, piece.piece_type) {
            // White pieces - white background with black text
            (ChessColor::White, PieceType::Pawn) => (
                bevy::prelude::Color::srgb(1.0, 1.0, 1.0),
                "P",
                0.35
            ),
            (ChessColor::White, PieceType::Rook) => (
                bevy::prelude::Color::srgb(1.0, 1.0, 1.0),
                "R",
                0.40
            ),
            (ChessColor::White, PieceType::Knight) => (
                bevy::prelude::Color::srgb(1.0, 1.0, 1.0),
                "N",
                0.40
            ),
            (ChessColor::White, PieceType::Bishop) => (
                bevy::prelude::Color::srgb(1.0, 1.0, 1.0),
                "B",
                0.40
            ),
            (ChessColor::White, PieceType::Queen) => (
                bevy::prelude::Color::srgb(1.0, 1.0, 1.0),
                "Q",
                0.45
            ),
            (ChessColor::White, PieceType::King) => (
                bevy::prelude::Color::srgb(1.0, 1.0, 1.0),
                "K",
                0.45
            ),
            (ChessColor::White, _) => (
                bevy::prelude::Color::srgb(1.0, 1.0, 1.0),
                "?",
                0.40
            ),
            // Black pieces - dark gray/black background with white text
            (ChessColor::Black, PieceType::Pawn) => (
                bevy::prelude::Color::srgb(0.2, 0.2, 0.2),
                "P",
                0.35
            ),
            (ChessColor::Black, PieceType::Rook) => (
                bevy::prelude::Color::srgb(0.2, 0.2, 0.2),
                "R",
                0.40
            ),
            (ChessColor::Black, PieceType::Knight) => (
                bevy::prelude::Color::srgb(0.2, 0.2, 0.2),
                "N",
                0.40
            ),
            (ChessColor::Black, PieceType::Bishop) => (
                bevy::prelude::Color::srgb(0.2, 0.2, 0.2),
                "B",
                0.40
            ),
            (ChessColor::Black, PieceType::Queen) => (
                bevy::prelude::Color::srgb(0.2, 0.2, 0.2),
                "Q",
                0.45
            ),
            (ChessColor::Black, PieceType::King) => (
                bevy::prelude::Color::srgb(0.2, 0.2, 0.2),
                "K",
                0.45
            ),
            (ChessColor::Black, _) => (
                bevy::prelude::Color::srgb(0.2, 0.2, 0.2),
                "?",
                0.40
            ),
        };
        
        let text_color = if piece.color == ChessColor::White {
            bevy::prelude::Color::srgb(0.0, 0.0, 0.0) // Black text for white pieces
        } else {
            bevy::prelude::Color::srgb(1.0, 1.0, 1.0) // White text for black pieces
        };
        
        // Create piece as a hexagonal mesh
        let piece_size_pixels = piece_size * BOARD_SCALE * 0.35;
        
        // Spawn piece with hexagonal mesh and text label
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: meshes.add(RegularPolygon::new(piece_size_pixels, 6)).into(),
                material: materials.add(ColorMaterial::from(piece_color)),
                transform: Transform::from_xyz(world_x, world_y, 1.0), // z=1.0 to be above tiles
                ..default()
            },
            ChessPiece { coord, piece },
        )).with_children(|parent| {
            // Add text label for piece type (use default font if custom font not available)
            parent.spawn(Text2dBundle {
                text: Text::from_section(
                    piece_label,
                    TextStyle {
                        font_size: piece_size * BOARD_SCALE * 0.5,
                        color: text_color,
                        ..default()
                    },
                ),
                transform: Transform::from_xyz(0.0, 0.0, 0.1), // Slightly above the hexagon
                ..default()
            });
        });
    }
    
    let msg = wasm_bindgen::JsValue::from_str(&format!("Finished spawning pieces. Total pieces on board: {}", game_data.game.board.pieces.len()));
    unsafe {
        web_sys::console::log_1(&msg);
    }
}

// 3D mesh creation functions removed - using 2D shapes instead

#[derive(Component)]
pub struct RulesUI;

fn spawn_ui(commands: &mut Commands, _asset_server: &Res<AssetServer>) {
    // Main game UI (top left)
    commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(10.0),
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            background_color: bevy::prelude::Color::srgba(0.0, 0.0, 0.0, 0.7).into(),
            ..default()
        },
        GameUI,
    )).with_children(|parent| {
        parent.spawn(TextBundle::from_section(
            "Hexagonal Chess - Gliński's Chess",
            TextStyle {
                font_size: 24.0,
                color: bevy::prelude::Color::WHITE,
                ..default()
            },
        ));
    });
    
    // Rules UI (top right)
    commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                right: Val::Px(10.0),
                width: Val::Px(300.0),
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            background_color: bevy::prelude::Color::srgba(0.0, 0.0, 0.0, 0.8).into(),
            ..default()
        },
        RulesUI,
    )).with_children(|parent| {
        parent.spawn(TextBundle::from_section(
            "Rules",
            TextStyle {
                font_size: 20.0,
                color: bevy::prelude::Color::WHITE,
                ..default()
            },
        ));
    });
}

fn handle_input(
    mut game_data: ResMut<GameData>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    hex_tiles: Query<&HexTile>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    piece_query: Query<(Entity, &mut ChessPiece)>,
    captured_pieces: ResMut<CapturedPieces>,
) {
    if mouse_buttons.just_pressed(MouseButton::Left) {
        // Debug: log that click was detected
        let msg = wasm_bindgen::JsValue::from_str("Mouse click detected");
        unsafe {
            web_sys::console::log_1(&msg);
        }
        
        if let Some(clicked_coord) = get_clicked_hex(&windows, &camera_query, &hex_tiles) {
            let msg = wasm_bindgen::JsValue::from_str(&format!("Clicked hex: {:?}", clicked_coord));
            unsafe {
                web_sys::console::log_1(&msg);
            }
            handle_hex_click(&mut game_data, clicked_coord, &mut commands, &mut meshes, &mut materials, piece_query, captured_pieces);
        } else {
            let msg = wasm_bindgen::JsValue::from_str("No hex coordinate found for click");
            unsafe {
                web_sys::console::log_1(&msg);
            }
        }
    }
}

fn handle_menu_toggle(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
) {
    // Toggle menu with 'M' key
    if keyboard_input.just_pressed(KeyCode::KeyM) {
        let msg = wasm_bindgen::JsValue::from_str("M key pressed - toggling menu");
        unsafe {
            web_sys::console::log_1(&msg);
        }
        match current_state.get() {
            GameState::Menu => {
                next_state.set(GameState::Playing);
                let msg = wasm_bindgen::JsValue::from_str("Switching to Playing state");
                unsafe {
                    web_sys::console::log_1(&msg);
                }
            }
            GameState::Playing => {
                next_state.set(GameState::Menu);
                let msg = wasm_bindgen::JsValue::from_str("Switching to Menu state");
                unsafe {
                    web_sys::console::log_1(&msg);
                }
            }
            _ => {}
        }
    }
}

fn get_clicked_hex(
    windows: &Query<&Window>,
    camera_query: &Query<(&Camera, &GlobalTransform)>,
    hex_tiles: &Query<&HexTile>,
) -> Option<HexCoord> {
    // Get cursor position
    let window = windows.get_single().ok()?;
    let cursor_pos = window.cursor_position()?;
    
    // Get camera
    let (camera, camera_transform) = camera_query.get_single().ok()?;
    
    // Use Bevy's viewport_to_world_2d method for accurate screen-to-world conversion
    let world_pos = camera.viewport_to_world_2d(camera_transform, cursor_pos)?;
    
    // Debug: log the click position
    let msg = wasm_bindgen::JsValue::from_str(&format!("Click at screen ({:.2}, {:.2}) -> world ({:.2}, {:.2})", cursor_pos.x, cursor_pos.y, world_pos.x, world_pos.y));
    unsafe {
        web_sys::console::log_1(&msg);
    }
    
    // Find the nearest hex tile by calculating distance to each tile's center
    let mut closest_tile: Option<(HexCoord, f32)> = None;
    const BOARD_SCALE: f32 = 100.0;
    let click_threshold = BOARD_SCALE * 0.5; // Within 50% of a hex unit (larger threshold for easier clicking)
    
    for tile in hex_tiles.iter() {
        let (px, py) = tile.coord.to_pixel();
        let tile_pos = Vec2::new(px * BOARD_SCALE, py * BOARD_SCALE);
        
        // Calculate distance from click to tile center
        let dist = world_pos.distance(tile_pos);
        
        if dist < click_threshold {
            if closest_tile.is_none() || dist < closest_tile.unwrap().1 {
                closest_tile = Some((tile.coord, dist));
            }
        }
    }
    
    if let Some((coord, dist)) = closest_tile {
        let msg = wasm_bindgen::JsValue::from_str(&format!("Found nearest tile: {:?} at distance {:.2}", coord, dist));
        unsafe {
            web_sys::console::log_1(&msg);
        }
        Some(coord)
    } else {
        let msg = wasm_bindgen::JsValue::from_str(&format!("No tile found within {:.2} units of click at ({:.2}, {:.2})", click_threshold, world_pos.x, world_pos.y));
        unsafe {
            web_sys::console::log_1(&msg);
        }
        None
    }
}

fn handle_hex_click(
    game_data: &mut ResMut<GameData>,
    coord: HexCoord,
    commands: &mut Commands,
    _meshes: &mut ResMut<Assets<Mesh>>,
    _materials: &mut ResMut<Assets<ColorMaterial>>,
    mut piece_query: Query<(Entity, &mut ChessPiece)>,
    mut captured_pieces: ResMut<CapturedPieces>,
) {
    let msg = wasm_bindgen::JsValue::from_str(&format!("handle_hex_click called with coord: {:?}", coord));
    unsafe {
        web_sys::console::log_1(&msg);
    }
    
    if let Some(selected) = game_data.selected_piece {
        let msg = wasm_bindgen::JsValue::from_str(&format!("Piece already selected at: {:?}", selected));
        unsafe {
            web_sys::console::log_1(&msg);
        }
        
        // Try to move the selected piece
        if game_data.valid_moves.contains(&coord) {
            let msg = wasm_bindgen::JsValue::from_str(&format!("Valid move! Attempting to move from {:?} to {:?}", selected, coord));
            unsafe {
                web_sys::console::log_1(&msg);
            }
            
            // Check if there's a piece at the destination to capture
            let captured_piece = game_data.game.board.get_piece(coord).copied();
            
            if let Err(e) = game_data.game.make_move(selected, coord) {
                let error_msg = wasm_bindgen::JsValue::from_str(&format!("Move error: {:?}", e));
                unsafe {
                    web_sys::console::log_1(&error_msg);
                }
            } else {
                let msg = wasm_bindgen::JsValue::from_str("Move successful! Updating piece entity...");
                unsafe {
                    web_sys::console::log_1(&msg);
                }
                
                // Remove captured piece entity if any
                if let Some(captured) = captured_piece {
                    let msg = wasm_bindgen::JsValue::from_str(&format!("Capture detected! Removing piece: {:?} at {:?}", captured, coord));
                    unsafe {
                        web_sys::console::log_1(&msg);
                    }
                    
                    for (entity, chess_piece) in piece_query.iter() {
                        if chess_piece.coord == coord && chess_piece.piece.piece_type == captured.piece_type && chess_piece.piece.color == captured.color {
                            commands.entity(entity).despawn_recursive();
                            captured_pieces.add(captured);
                            let msg = wasm_bindgen::JsValue::from_str(&format!("Despawned captured piece entity at {:?}", coord));
                            unsafe {
                                web_sys::console::log_1(&msg);
                            }
                            break;
                        }
                    }
                }
                
                // Update the piece entity's coordinate
                let mut found = false;
                for (_entity, mut chess_piece) in piece_query.iter_mut() {
                    if chess_piece.coord == selected {
                        chess_piece.coord = coord;
                        found = true;
                        let msg = wasm_bindgen::JsValue::from_str(&format!("Updated piece entity from {:?} to {:?}", selected, coord));
                        unsafe {
                            web_sys::console::log_1(&msg);
                        }
                        break;
                    }
                }
                
                if !found {
                    let msg = wasm_bindgen::JsValue::from_str(&format!("WARNING: Could not find piece entity at {:?}", selected));
                    unsafe {
                        web_sys::console::log_1(&msg);
                    }
                }
                
                game_data.selected_piece = None;
                game_data.valid_moves.clear();
            }
        } else {
            // Clicked on invalid move, deselect
            let msg = wasm_bindgen::JsValue::from_str(&format!("Invalid move to {:?}, deselecting", coord));
            unsafe {
                web_sys::console::log_1(&msg);
            }
            game_data.selected_piece = None;
            game_data.valid_moves.clear();
        }
    } else {
        // Select a piece
        let msg = wasm_bindgen::JsValue::from_str(&format!("No piece selected. Checking for piece at {:?}", coord));
        unsafe {
            web_sys::console::log_1(&msg);
        }
        
        // Debug: log all pieces on the board
        let all_pieces: Vec<_> = game_data.game.board.pieces.iter().collect();
        let msg = wasm_bindgen::JsValue::from_str(&format!("Board has {} pieces total. Checking for piece at {:?}", all_pieces.len(), coord));
        unsafe {
            web_sys::console::log_1(&msg);
        }
        
        // Debug: list first few piece coordinates with their world positions
        let mut piece_info = Vec::new();
        for (coord, _piece) in game_data.game.board.pieces.iter().take(5) {
            const BOARD_SCALE: f32 = 100.0;
            let (px, py) = coord.to_pixel();
            let wx = px * BOARD_SCALE;
            let wy = py * BOARD_SCALE;
            piece_info.push(format!("{:?} -> world({:.2}, {:.2})", coord, wx, wy));
        }
        let msg = wasm_bindgen::JsValue::from_str(&format!("Sample pieces: {}", piece_info.join(", ")));
        unsafe {
            web_sys::console::log_1(&msg);
        }
        
        if let Some(piece) = game_data.game.board.get_piece(coord) {
            let msg = wasm_bindgen::JsValue::from_str(&format!("Found piece: {:?} at {:?}", piece, coord));
            unsafe {
                web_sys::console::log_1(&msg);
            }
            
            let current_player_str = match game_data.game.current_player {
                ChessColor::White => "White",
                ChessColor::Black => "Black",
            };
            let piece_color_str = match piece.color {
                ChessColor::White => "White",
                ChessColor::Black => "Black",
            };
            
            let msg = wasm_bindgen::JsValue::from_str(&format!("Current player: {}, Piece color: {}", current_player_str, piece_color_str));
            unsafe {
                web_sys::console::log_1(&msg);
            }
            
            if piece.color == game_data.game.current_player {
                game_data.selected_piece = Some(coord);
                game_data.valid_moves = game_data.game.board.get_valid_moves(coord);
                let msg = wasm_bindgen::JsValue::from_str(&format!("Piece selected! Valid moves: {:?}", game_data.valid_moves));
                unsafe {
                    web_sys::console::log_1(&msg);
                }
            } else {
                let msg = wasm_bindgen::JsValue::from_str("Piece belongs to other player, cannot select");
                unsafe {
                    web_sys::console::log_1(&msg);
                }
            }
        } else {
            let msg = wasm_bindgen::JsValue::from_str(&format!("No piece found at {:?}", coord));
            unsafe {
                web_sys::console::log_1(&msg);
            }
        }
    }
}

fn update_board_visuals(
    _game_data: Res<GameData>,
    _piece_query: Query<(&mut Transform, &ChessPiece)>,
) {
    // Update piece positions based on game state
    // Note: Selection visuals are handled in update_selection_visuals
    // This function is kept for future use when we need to sync piece positions
    // with the game board state after moves
}

fn update_ui(
    game_data: Res<GameData>,
    mut ui_query: Query<&mut Text, With<GameUI>>,
    mut rules_query: Query<&mut Text, (With<RulesUI>, Without<GameUI>)>,
) {
    if let Ok(mut text) = ui_query.get_single_mut() {
        let variant = game_data.variant();
        let current_player = match game_data.game.current_player {
            ChessColor::White => "White",
            ChessColor::Black => "Black",
        };
        
        let mut ui_text = format!("{} - {} to move", variant.name, current_player);
        
        // Add piece selection information
        if let Some(selected_coord) = game_data.selected_piece {
            if let Some(piece) = game_data.game.board.get_piece(selected_coord) {
                let piece_color = match piece.color {
                    ChessColor::White => "White",
                    ChessColor::Black => "Black",
                };
                let piece_type = match piece.piece_type {
                    PieceType::King => "King",
                    PieceType::Queen => "Queen",
                    PieceType::Rook => "Rook",
                    PieceType::Bishop => "Bishop",
                    PieceType::Knight => "Knight",
                    PieceType::Pawn => "Pawn",
                    PieceType::Chancellor => "Chancellor",
                    PieceType::Archbishop => "Archbishop",
                };
                let move_count = game_data.valid_moves.len();
                ui_text = format!("{} | Selected: {} {} at {:?} | {} valid moves", 
                    ui_text, piece_color, piece_type, selected_coord, move_count);
            }
        }
        
        text.sections[0].value = ui_text;
    }
    
    // Update rules UI
    if let Ok(mut rules_text) = rules_query.get_single_mut() {
        let variant = game_data.variant();
        let mut rules_text_content = format!("{}\n{}\n\n", variant.name, variant.description);
        
        rules_text_content.push_str("Gliński's Chess Rules:\n\n");
        rules_text_content.push_str("Piece Movement:\n");
        rules_text_content.push_str("• Pawn: Moves straight forward, captures diagonally forward (2 directions)\n");
        rules_text_content.push_str("• Rook: Moves in 6 directions (rank, file, diagonals)\n");
        rules_text_content.push_str("• Bishop: Moves in 6 diagonal directions\n");
        rules_text_content.push_str("• Knight: Leaps 3 steps (L-shaped, 12 possible moves)\n");
        rules_text_content.push_str("• Queen: Rook + Bishop (9 directions)\n");
        rules_text_content.push_str("• King: One step to 6 adjacent hexes (rook-like, one step)\n");
        
        rules_text_content.push_str("\nSpecial Rules:\n");
        let mut has_en_passant = false;
        for rule in &variant.special_rules {
            match rule {
                hex_chess_core::SpecialRule::EnPassant => {
                    has_en_passant = true;
                }
                _ => {}
            }
        }
        if has_en_passant {
            rules_text_content.push_str("• En Passant allowed\n");
        }
        rules_text_content.push_str("• No Castling\n");
        rules_text_content.push_str("• Pawns promote at opposite border\n");
        
        // Add controls info to rules UI
        rules_text_content.push_str("\n\nControls:\n");
        rules_text_content.push_str("• Click to select and move pieces\n");
        rules_text_content.push_str("• Mouse wheel or +/- to zoom\n");
        rules_text_content.push_str("• Arrow keys to pan camera\n");
        rules_text_content.push_str("• R to reset camera\n");
        rules_text_content.push_str("• M to toggle menu\n");
        
        rules_text.sections[0].value = rules_text_content;
    }
}

fn update_selection_visuals(
    game_data: Res<GameData>,
    mut piece_query: Query<(&mut Transform, &ChessPiece)>,
    mut tile_query: Query<(&mut Transform, &Handle<ColorMaterial>, &HexTile), Without<ChessPiece>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    valid_move_color: Res<ValidMoveColor>,
) {
    const BOARD_SCALE: f32 = 100.0;
    
    // Update piece positions - selected pieces are highlighted by z-index
    for (mut transform, chess_piece) in piece_query.iter_mut() {
        let (x, y) = chess_piece.coord.to_pixel();
        let is_selected = game_data.selected_piece == Some(chess_piece.coord);
        let z = if is_selected {
            2.0 // Raise selected piece above normal pieces
        } else {
            1.0 // Normal height above tiles
        };
        
        // Note: Scale changes would require access to the mesh handle
        // For now, we just use z-index to highlight selected pieces
        
        transform.translation = Vec3::new(x * BOARD_SCALE, y * BOARD_SCALE, z);
    }
    
    // Highlight valid move tiles by changing color
    for (mut transform, material_handle, tile) in tile_query.iter_mut() {
        let (x, y) = tile.coord.to_pixel();
        let is_valid_move = game_data.valid_moves.contains(&tile.coord);
        
        // Change material color for valid moves (green highlight)
        if let Some(material) = materials.get_mut(material_handle) {
            if is_valid_move {
                material.color = valid_move_color.color;
            } else {
                // Restore base color
                material.color = tile.base_color;
            }
        }
        
        transform.translation = Vec3::new(x * BOARD_SCALE, y * BOARD_SCALE, 0.0);
    }
}

fn handle_camera_zoom(
    mut camera_query: Query<&mut OrthographicProjection, With<Camera>>,
    mut scroll_events: EventReader<MouseWheel>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    let mut projection = match camera_query.get_single_mut() {
        Ok(proj) => proj,
        Err(_) => return,
    };
    
    // Mouse wheel zoom
    for event in scroll_events.read() {
        let zoom_delta = event.y * 0.1;
        projection.scale = (projection.scale - zoom_delta).clamp(0.2, 2.0);
        
        let msg = wasm_bindgen::JsValue::from_str(&format!("Zoom: {:.2}", projection.scale));
        unsafe {
            web_sys::console::log_1(&msg);
        }
    }
    
    // Keyboard zoom (+ and - keys)
    if keyboard.pressed(KeyCode::Equal) || keyboard.pressed(KeyCode::NumpadAdd) {
        projection.scale = (projection.scale - 0.02).max(0.2);
    }
    if keyboard.pressed(KeyCode::Minus) || keyboard.pressed(KeyCode::NumpadSubtract) {
        projection.scale = (projection.scale + 0.02).min(2.0);
    }
}

fn handle_camera_pan(
    mut camera_query: Query<&mut Transform, With<Camera>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut last_cursor_pos: Local<Option<Vec2>>,
    windows: Query<&Window>,
) {
    let mut camera_transform = match camera_query.get_single_mut() {
        Ok(trans) => trans,
        Err(_) => return,
    };
    
    // Arrow key panning
    let pan_speed = 5.0;
    if keyboard.pressed(KeyCode::ArrowLeft) {
        camera_transform.translation.x -= pan_speed;
    }
    if keyboard.pressed(KeyCode::ArrowRight) {
        camera_transform.translation.x += pan_speed;
    }
    if keyboard.pressed(KeyCode::ArrowUp) {
        camera_transform.translation.y += pan_speed;
    }
    if keyboard.pressed(KeyCode::ArrowDown) {
        camera_transform.translation.y -= pan_speed;
    }
    
    // Middle mouse button drag panning
    let window = windows.single();
    if mouse_buttons.pressed(MouseButton::Middle) {
        if let Some(cursor_pos) = window.cursor_position() {
            if let Some(last_pos) = *last_cursor_pos {
                let delta = cursor_pos - last_pos;
                camera_transform.translation.x -= delta.x;
                camera_transform.translation.y += delta.y; // Invert Y
            }
            *last_cursor_pos = Some(cursor_pos);
        }
    } else {
        *last_cursor_pos = None;
    }
    
    // Reset camera with 'R' key
    if keyboard.just_pressed(KeyCode::KeyR) {
        camera_transform.translation = Vec3::new(0.0, 0.0, 1000.0);
        let msg = wasm_bindgen::JsValue::from_str("Camera reset to center");
        unsafe {
            web_sys::console::log_1(&msg);
        }
    }
}

fn handle_menu_input(
    mut game_state: ResMut<NextState<GameState>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    // Press Space or M to start/return to game
    if keyboard_input.just_pressed(KeyCode::Space) || keyboard_input.just_pressed(KeyCode::KeyM) {
        game_state.set(GameState::Playing);
    }
}

fn spawn_coordinate_labels(
    commands: &mut Commands,
    game_data: &GameData,
) {
    const BOARD_SCALE: f32 = 100.0;
    const LABEL_DISTANCE: f32 = 1.3; // Position labels 30% beyond hex center
    
    let msg = wasm_bindgen::JsValue::from_str("Spawning coordinate labels...");
    unsafe {
        web_sys::console::log_1(&msg);
    }
    
    for &coord in &game_data.game.board.valid_coords {
        // Check if this is a perimeter hex (has at least one invalid neighbor)
        let neighbors = coord.neighbors();
        let is_perimeter = neighbors.iter()
            .any(|n| !game_data.game.board.valid_coords.contains(n));
        
        if is_perimeter {
            let (px, py) = coord.to_pixel();
            let label_x = px * BOARD_SCALE * LABEL_DISTANCE;
            let label_y = py * BOARD_SCALE * LABEL_DISTANCE;
            
            commands.spawn((
                Text2dBundle {
                    text: Text::from_section(
                        format!("({}, {})", coord.q, coord.r),
                        TextStyle {
                            font_size: 11.0,
                            color: Color::srgba(0.7, 0.7, 0.7, 0.6),
                            ..default()
                        },
                    ),
                    transform: Transform::from_xyz(label_x, label_y, 5.0),
                    ..default()
                },
                CoordinateLabel,
            ));
        }
    }
    
    let msg = wasm_bindgen::JsValue::from_str("Coordinate labels spawned");
    unsafe {
        web_sys::console::log_1(&msg);
    }
}

fn spawn_captured_pieces_areas(
    commands: &mut Commands,
) {
    let msg = wasm_bindgen::JsValue::from_str("Spawning captured pieces areas...");
    unsafe {
        web_sys::console::log_1(&msg);
    }
    
    // White's captured pieces (bottom-left) - pieces lost by White
    commands.spawn(NodeBundle {
        style: Style {
            position_type: PositionType::Absolute,
            left: Val::Px(10.0),
            bottom: Val::Px(10.0),
            width: Val::Px(140.0),
            padding: UiRect::all(Val::Px(8.0)),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        background_color: Color::srgba(0.15, 0.15, 0.15, 0.85).into(),
        ..default()
    }).with_children(|parent| {
        parent.spawn(TextBundle::from_section(
            "White Lost:",
            TextStyle {
                font_size: 16.0,
                color: Color::srgb(0.9, 0.9, 0.9),
                ..default()
            },
        ));
        parent.spawn((
            TextBundle::from_section(
                "",
                TextStyle {
                    font_size: 14.0,
                    color: Color::srgb(0.8, 0.8, 0.8),
                    ..default()
                },
            ),
            CapturedPiecesUI { color: ChessColor::White },
        ));
    });
    
    // Black's captured pieces (top-right) - pieces lost by Black
    commands.spawn(NodeBundle {
        style: Style {
            position_type: PositionType::Absolute,
            right: Val::Px(10.0),
            top: Val::Px(10.0),
            width: Val::Px(140.0),
            padding: UiRect::all(Val::Px(8.0)),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        background_color: Color::srgba(0.15, 0.15, 0.15, 0.85).into(),
        ..default()
    }).with_children(|parent| {
        parent.spawn(TextBundle::from_section(
            "Black Lost:",
            TextStyle {
                font_size: 16.0,
                color: Color::srgb(0.9, 0.9, 0.9),
                ..default()
            },
        ));
        parent.spawn((
            TextBundle::from_section(
                "",
                TextStyle {
                    font_size: 14.0,
                    color: Color::srgb(0.8, 0.8, 0.8),
                    ..default()
                },
            ),
            CapturedPiecesUI { color: ChessColor::Black },
        ));
    });
    
    let msg = wasm_bindgen::JsValue::from_str("Captured pieces areas spawned");
    unsafe {
        web_sys::console::log_1(&msg);
    }
}

fn update_captured_pieces_display(
    captured_pieces: Res<CapturedPieces>,
    mut query: Query<(&mut Text, &CapturedPiecesUI)>,
) {
    // Only update when captured pieces change
    if !captured_pieces.is_changed() {
        return;
    }
    
    for (mut text, ui) in query.iter_mut() {
        let pieces = match ui.color {
            ChessColor::White => &captured_pieces.white,
            ChessColor::Black => &captured_pieces.black,
        };
        
        if pieces.is_empty() {
            text.sections[0].value = String::new();
        } else {
            // Format pieces in a compact grid (3 pieces per row)
            let mut display = String::new();
            for (i, piece) in pieces.iter().enumerate() {
                let symbol = match piece.piece_type {
                    PieceType::Pawn => "P",
                    PieceType::Knight => "N",
                    PieceType::Bishop => "B",
                    PieceType::Rook => "R",
                    PieceType::Queen => "Q",
                    PieceType::King => "K",
                    PieceType::Chancellor => "C",
                    PieceType::Archbishop => "A",
                };
                display.push_str(symbol);
                
                // Add space between pieces in same row
                if (i + 1) % 3 != 0 && i < pieces.len() - 1 {
                    display.push(' ');
                }
                
                // New line every 3 pieces
                if (i + 1) % 3 == 0 && i < pieces.len() - 1 {
                    display.push('\n');
                }
            }
            text.sections[0].value = display;
        }
    }
}

