use bevy::{prelude::*, utils::HashMap, window::PrimaryWindow};

const PIECE_SIZE: usize = 60;
const BOARD_SIZE: usize = 8;

#[derive(Component, PartialEq, Eq, Hash)]
enum Piece {
    King,
    Queen,
    Knight,
    Pawn,
    Bishop,
    Rook,
}

#[derive(Component)]
struct Tile {
    x: usize,
    y: usize,
}

#[derive(Component, PartialEq, Eq)]
struct BoardPosition {
    x: usize,
    y: usize,
}

impl BoardPosition {
    fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

#[derive(Component, PartialEq, Eq)]
enum Player {
    White,
    Black,
}

#[derive(Resource)]
struct GameAssets {
    piece_atlas: Handle<TextureAtlas>,
    pieces: HashMap<Piece, usize>,
}

#[derive(Resource)]
struct BoardPopulationDone(bool);

#[derive(Resource)]
struct CurrentTurn(Player);

#[derive(Resource)]
struct SelectedPiece(Option<Entity>);

fn main() {
    App::new()
        .insert_resource(BoardPopulationDone(false))
        .insert_resource(CurrentTurn(Player::White))
        .insert_resource(SelectedPiece(None))
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: (
                        (PIECE_SIZE * BOARD_SIZE) as f32,
                        (PIECE_SIZE * BOARD_SIZE) as f32,
                    )
                        .into(),
                    title: "Chess".to_string(),
                    resizable: true,
                    ..default()
                }),
                ..default()
            }),
        )
        .add_startup_system(load_assets)
        .add_startup_system(spawn_camera)
        .add_startup_system(generate_board)
        .add_system(populate_board)
        .add_system(update_pieces_positions)
        .add_system(handle_piece_selection)
        .add_system(display_possible_piece_movements)
        .run();
}

fn load_assets(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let piece_atlas = TextureAtlas::from_grid(
        assets.load("pieces.png"),
        Vec2::splat(PIECE_SIZE as f32),
        6,
        2,
        None,
        None,
    );

    commands.insert_resource(GameAssets {
        piece_atlas: texture_atlases.add(piece_atlas),
        pieces: HashMap::from([
            (Piece::King, 0),
            (Piece::Queen, 1),
            (Piece::Knight, 2),
            (Piece::Pawn, 3),
            (Piece::Bishop, 4),
            (Piece::Rook, 5),
        ]),
    });
}

fn spawn_camera(mut commands: Commands, window: Query<&Window, With<PrimaryWindow>>) {
    let window = window.get_single().unwrap();

    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 0.0),
        ..default()
    });
}

fn generate_board(mut commands: Commands) {
    let board = commands
        .spawn((TransformBundle::default(), VisibilityBundle::default()))
        .id();

    for x in 0..BOARD_SIZE {
        for y in 0..BOARD_SIZE {
            let piece = commands
                .spawn((
                    SpriteBundle {
                        transform: Transform::from_xyz(
                            (x * PIECE_SIZE + PIECE_SIZE / 2) as f32,
                            (y * PIECE_SIZE + PIECE_SIZE / 2) as f32,
                            0.0,
                        ),
                        sprite: Sprite {
                            color: get_tile_color(x, y),
                            custom_size: Some(Vec2::splat(PIECE_SIZE as f32)),
                            ..default()
                        },
                        ..default()
                    },
                    Tile { x, y },
                ))
                .id();

            commands.entity(board).add_child(piece);
        }
    }
}

fn populate_board(
    mut commands: Commands,
    mut population_done: ResMut<BoardPopulationDone>,
    game_assets: Res<GameAssets>,
) {
    if !population_done.0 {
        spawn_white_pieces(&game_assets, &mut commands);
        spawn_black_pieces(&game_assets, &mut commands);
        population_done.0 = true;
    }
}

fn update_pieces_positions(mut pieces: Query<(&mut Transform, &BoardPosition), With<Piece>>) {
    for (mut transform, position) in pieces.iter_mut() {
        transform.translation.x = (position.x * PIECE_SIZE + (PIECE_SIZE / 2)) as f32;
        transform.translation.y = (position.y * PIECE_SIZE + (PIECE_SIZE / 2)) as f32;
    }
}

fn handle_piece_selection(
    buttons: Res<Input<MouseButton>>,
    window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    pieces: Query<(Entity, &BoardPosition, &Player), With<Piece>>,
    mut tiles: Query<(&Tile, &mut Sprite)>,
    current_player: Res<CurrentTurn>,
    mut selected_piece: ResMut<SelectedPiece>,
) {
    let window = window.get_single().unwrap();
    let (camera, camera_transform) = camera.get_single().unwrap();

    let mut selected_piece_board_position = None;

    if buttons.just_pressed(MouseButton::Left) {
        if let Some(world_position) = window
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
            .map(|ray| ray.origin.truncate())
        {
            for (entity, position, player) in pieces.iter() {
                if player == &current_player.0
                    && position.x as f32 == to_board_posistion(world_position.x)
                    && position.y as f32 == to_board_posistion(world_position.y)
                {
                    selected_piece.0 = Some(entity);
                    selected_piece_board_position = Some(position);
                    break;
                } else {
                    selected_piece.0 = None;
                }
            }
            for (tile_pos, mut tile_sprite) in tiles.iter_mut() {
                if let Some(selected_piece_board_position) = selected_piece_board_position {
                    if tile_pos.x == selected_piece_board_position.x
                        && tile_pos.y == selected_piece_board_position.y
                    {
                        tile_sprite.color = Color::YELLOW;
                    } else {
                        tile_sprite.color = get_tile_color(tile_pos.x, tile_pos.y);
                    }
                } else {
                    tile_sprite.color = get_tile_color(tile_pos.x, tile_pos.y);
                }
            }
        }
    }
}

fn display_possible_piece_movements(
    selected_piece: Res<SelectedPiece>,
    pieces: Query<(&BoardPosition, &Player, &Piece)>,
) {
    if let Some(selected_piece_ent) = selected_piece.0 {
        let mut white_pieces_positions = Vec::new();
        let mut black_pieces_positions = Vec::new();

        for (piece_board_position, piece_player, _) in pieces.iter() {
            match piece_player {
                &Player::White => {
                    white_pieces_positions.push(piece_board_position);
                }
                &Player::Black => {
                    black_pieces_positions.push(piece_board_position);
                }
            }
        }

        let selected_piece = pieces.get(selected_piece_ent).unwrap();

        dbg!(get_possible_moves(
            selected_piece.2,
            selected_piece.0,
            selected_piece.1,
            white_pieces_positions,
            black_pieces_positions
        ));
    }
}

fn get_tile_color(x: usize, y: usize) -> Color {
    if (x % 2 == 1 && y % 2 != 1) || (x % 2 != 1 && y % 2 == 1) {
        Color::LIME_GREEN
    } else {
        Color::GREEN
    }
}

fn to_board_posistion(pos: f32) -> f32 {
    (pos.round() / PIECE_SIZE as f32).floor()
}

fn get_possible_moves(
    piece_type: &Piece,
    piece_position: &BoardPosition,
    piece_player: &Player,
    white_pieces_positions: Vec<&BoardPosition>,
    black_pieces_positions: Vec<&BoardPosition>,
) -> Vec<(f32, f32)> {
    let mut possible_moves = Vec::new();

    match piece_type {
        Piece::King => {}
        Piece::Queen => {}
        Piece::Knight => {}
        Piece::Pawn => match piece_player {
            &Player::White => {
                if !white_pieces_positions
                    .contains(&&BoardPosition::new(piece_position.x, piece_position.y + 1))
                    && !black_pieces_positions
                        .contains(&&BoardPosition::new(piece_position.x, piece_position.y + 1))
                    && piece_position.y < 7
                {
                    possible_moves.push((piece_position.x as f32, (piece_position.y + 1) as f32));
                } else if !white_pieces_positions
                    .contains(&&BoardPosition::new(piece_position.x, piece_position.y + 2))
                    && !black_pieces_positions
                        .contains(&&BoardPosition::new(piece_position.x, piece_position.y + 2))
                    && piece_position.y == 1
                {
                    possible_moves.push((piece_position.x as f32, (piece_position.y + 2) as f32));
                } else if black_pieces_positions.contains(&&BoardPosition::new(
                    piece_position.x + 1,
                    piece_position.y + 1,
                )) {
                    possible_moves
                        .push(((piece_position.x + 1) as f32, (piece_position.y + 1) as f32));
                } else if black_pieces_positions.contains(&&BoardPosition::new(
                    piece_position.x - 1,
                    piece_position.y + 1,
                )) {
                    possible_moves
                        .push(((piece_position.x - 1) as f32, (piece_position.y + 1) as f32));
                }
            }
            &Player::Black => {
                if !white_pieces_positions
                    .contains(&&BoardPosition::new(piece_position.x, piece_position.y - 1))
                    && !black_pieces_positions
                        .contains(&&BoardPosition::new(piece_position.x, piece_position.y - 1))
                    && piece_position.y > 0
                {
                    possible_moves.push((piece_position.x as f32, (piece_position.y - 1) as f32));
                } else if !white_pieces_positions
                    .contains(&&BoardPosition::new(piece_position.x, piece_position.y - 2))
                    && !black_pieces_positions
                        .contains(&&BoardPosition::new(piece_position.x, piece_position.y - 2))
                    && piece_position.y == 6
                {
                    possible_moves.push((piece_position.x as f32, (piece_position.y - 2) as f32));
                } else if white_pieces_positions.contains(&&BoardPosition::new(
                    piece_position.x + 1,
                    piece_position.y - 1,
                )) {
                    possible_moves
                        .push(((piece_position.x + 1) as f32, (piece_position.y - 1) as f32));
                } else if white_pieces_positions.contains(&&BoardPosition::new(
                    piece_position.x - 1,
                    piece_position.y - 1,
                )) {
                    possible_moves
                        .push(((piece_position.x - 1) as f32, (piece_position.y - 1) as f32));
                }
            }
        },
        Piece::Bishop => {}
        Piece::Rook => {}
    }

    possible_moves
}

fn spawn_piece(
    piece_type: Piece,
    player: Player,
    x: usize,
    y: usize,
    texture_atlas: Handle<TextureAtlas>,
    index: usize,
    commands: &mut Commands,
) {
    commands.spawn((
        SpriteSheetBundle {
            sprite: TextureAtlasSprite {
                custom_size: Some(Vec2::splat(PIECE_SIZE as f32)),
                index,
                ..default()
            },
            texture_atlas,
            ..default()
        },
        piece_type,
        player,
        BoardPosition::new(x, y),
    ));
}

fn spawn_white_pieces(game_assets: &GameAssets, commands: &mut Commands) {
    spawn_piece(
        Piece::King,
        Player::White,
        4,
        0,
        game_assets.piece_atlas.clone(),
        game_assets.pieces[&Piece::King],
        commands,
    );
    spawn_piece(
        Piece::Queen,
        Player::White,
        3,
        0,
        game_assets.piece_atlas.clone(),
        game_assets.pieces[&Piece::Queen],
        commands,
    );
    spawn_piece(
        Piece::Knight,
        Player::White,
        1,
        0,
        game_assets.piece_atlas.clone(),
        game_assets.pieces[&Piece::Knight],
        commands,
    );
    spawn_piece(
        Piece::Knight,
        Player::White,
        6,
        0,
        game_assets.piece_atlas.clone(),
        game_assets.pieces[&Piece::Knight],
        commands,
    );
    spawn_piece(
        Piece::Bishop,
        Player::White,
        2,
        0,
        game_assets.piece_atlas.clone(),
        game_assets.pieces[&Piece::Bishop],
        commands,
    );
    spawn_piece(
        Piece::Bishop,
        Player::White,
        5,
        0,
        game_assets.piece_atlas.clone(),
        game_assets.pieces[&Piece::Bishop],
        commands,
    );
    spawn_piece(
        Piece::Rook,
        Player::White,
        0,
        0,
        game_assets.piece_atlas.clone(),
        game_assets.pieces[&Piece::Rook],
        commands,
    );
    spawn_piece(
        Piece::Rook,
        Player::White,
        7,
        0,
        game_assets.piece_atlas.clone(),
        game_assets.pieces[&Piece::Rook],
        commands,
    );

    for i in 0..BOARD_SIZE {
        spawn_piece(
            Piece::Pawn,
            Player::White,
            i,
            1,
            game_assets.piece_atlas.clone(),
            game_assets.pieces[&Piece::Pawn],
            commands,
        );
    }
}

fn spawn_black_pieces(game_assets: &GameAssets, commands: &mut Commands) {
    spawn_piece(
        Piece::King,
        Player::Black,
        3,
        7,
        game_assets.piece_atlas.clone(),
        game_assets.pieces[&Piece::King] + 6,
        commands,
    );
    spawn_piece(
        Piece::Queen,
        Player::Black,
        4,
        7,
        game_assets.piece_atlas.clone(),
        game_assets.pieces[&Piece::Queen] + 6,
        commands,
    );
    spawn_piece(
        Piece::Knight,
        Player::Black,
        1,
        7,
        game_assets.piece_atlas.clone(),
        game_assets.pieces[&Piece::Knight] + 6,
        commands,
    );
    spawn_piece(
        Piece::Knight,
        Player::Black,
        6,
        7,
        game_assets.piece_atlas.clone(),
        game_assets.pieces[&Piece::Knight] + 6,
        commands,
    );
    spawn_piece(
        Piece::Bishop,
        Player::Black,
        2,
        7,
        game_assets.piece_atlas.clone(),
        game_assets.pieces[&Piece::Bishop] + 6,
        commands,
    );
    spawn_piece(
        Piece::Bishop,
        Player::Black,
        5,
        7,
        game_assets.piece_atlas.clone(),
        game_assets.pieces[&Piece::Bishop] + 6,
        commands,
    );
    spawn_piece(
        Piece::Rook,
        Player::Black,
        0,
        7,
        game_assets.piece_atlas.clone(),
        game_assets.pieces[&Piece::Rook] + 6,
        commands,
    );
    spawn_piece(
        Piece::Rook,
        Player::Black,
        7,
        7,
        game_assets.piece_atlas.clone(),
        game_assets.pieces[&Piece::Rook] + 6,
        commands,
    );

    for i in 0..BOARD_SIZE {
        spawn_piece(
            Piece::Pawn,
            Player::Black,
            i,
            6,
            game_assets.piece_atlas.clone(),
            game_assets.pieces[&Piece::Pawn] + 6,
            commands,
        );
    }
}
