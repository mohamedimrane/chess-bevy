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

#[derive(Component)]
struct BoardPosition {
    x: usize,
    y: usize,
}

impl BoardPosition {
    fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

#[derive(Resource)]
struct GameAssets {
    piece_atlas: Handle<TextureAtlas>,
    pieces: HashMap<Piece, usize>,
}

#[derive(Resource)]
struct BoardPopulationDone(bool);

#[derive(Resource)]
struct CurrentTurn(Turn);

enum Turn {
    White,
    Black,
}

#[derive(Resource)]
struct SelectedPiece(Option<Entity>);

fn main() {
    App::new()
        .insert_resource(BoardPopulationDone(false))
        .insert_resource(CurrentTurn(Turn::White))
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
            let color = if (x % 2 == 1 && y % 2 != 1) || (x % 2 != 1 && y % 2 == 1) {
                Color::LIME_GREEN
            } else {
                Color::GREEN
            };

            let piece = commands
                .spawn((
                    SpriteBundle {
                        transform: Transform::from_xyz(
                            (x * PIECE_SIZE + PIECE_SIZE / 2) as f32,
                            (y * PIECE_SIZE + PIECE_SIZE / 2) as f32,
                            0.0,
                        ),
                        sprite: Sprite {
                            color,
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
        transform.translation.x = ((position.x - 1) * PIECE_SIZE + (PIECE_SIZE / 2)) as f32;
        transform.translation.y = ((position.y - 1) * PIECE_SIZE + (PIECE_SIZE / 2)) as f32;
    }
}

fn handle_piece_selection(
    buttons: Res<Input<MouseButton>>,
    window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    pieces: Query<(Entity, &BoardPosition), With<Piece>>,
    mut selected_piece: ResMut<SelectedPiece>,
) {
    let window = window.get_single().unwrap();
    let (camera, camera_transform) = camera.get_single().unwrap();

    if buttons.just_pressed(MouseButton::Left) {
        if let Some(world_position) = window
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
            .map(|ray| ray.origin.truncate())
        {
            for (entity, position) in pieces.iter() {
                if (position.x - 1) as f32 == (world_position.x.round() / PIECE_SIZE as f32).floor()
                    && (position.y - 1) as f32
                        == (world_position.y.round() / PIECE_SIZE as f32).floor()
                {
                    selected_piece.0 = Some(entity);

                    // dbg!(selected_piece.0);
                    break;
                } else {
                    selected_piece.0 = None;
                    // dbg!(selected_piece.0);
                }
            }
        }
    }
}

fn spawn_piece(
    piece_type: Piece,
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
        BoardPosition::new(x, y),
        piece_type,
    ));
}

fn spawn_white_pieces(game_assets: &GameAssets, commands: &mut Commands) {
    spawn_piece(
        Piece::King,
        5,
        1,
        game_assets.piece_atlas.clone(),
        game_assets.pieces[&Piece::King],
        commands,
    );
    spawn_piece(
        Piece::Queen,
        4,
        1,
        game_assets.piece_atlas.clone(),
        game_assets.pieces[&Piece::Queen],
        commands,
    );
    spawn_piece(
        Piece::Knight,
        2,
        1,
        game_assets.piece_atlas.clone(),
        game_assets.pieces[&Piece::Knight],
        commands,
    );
    spawn_piece(
        Piece::Knight,
        7,
        1,
        game_assets.piece_atlas.clone(),
        game_assets.pieces[&Piece::Knight],
        commands,
    );
    spawn_piece(
        Piece::Bishop,
        3,
        1,
        game_assets.piece_atlas.clone(),
        game_assets.pieces[&Piece::Bishop],
        commands,
    );
    spawn_piece(
        Piece::Bishop,
        6,
        1,
        game_assets.piece_atlas.clone(),
        game_assets.pieces[&Piece::Bishop],
        commands,
    );
    spawn_piece(
        Piece::Rook,
        1,
        1,
        game_assets.piece_atlas.clone(),
        game_assets.pieces[&Piece::Rook],
        commands,
    );
    spawn_piece(
        Piece::Rook,
        8,
        1,
        game_assets.piece_atlas.clone(),
        game_assets.pieces[&Piece::Rook],
        commands,
    );

    for i in 1..=BOARD_SIZE {
        spawn_piece(
            Piece::Pawn,
            i,
            2,
            game_assets.piece_atlas.clone(),
            game_assets.pieces[&Piece::Pawn],
            commands,
        );
    }
}

fn spawn_black_pieces(game_assets: &GameAssets, commands: &mut Commands) {
    spawn_piece(
        Piece::King,
        4,
        8,
        game_assets.piece_atlas.clone(),
        game_assets.pieces[&Piece::King] + 6,
        commands,
    );
    spawn_piece(
        Piece::Queen,
        5,
        8,
        game_assets.piece_atlas.clone(),
        game_assets.pieces[&Piece::Queen] + 6,
        commands,
    );
    spawn_piece(
        Piece::Knight,
        2,
        8,
        game_assets.piece_atlas.clone(),
        game_assets.pieces[&Piece::Knight] + 6,
        commands,
    );
    spawn_piece(
        Piece::Knight,
        7,
        8,
        game_assets.piece_atlas.clone(),
        game_assets.pieces[&Piece::Knight] + 6,
        commands,
    );
    spawn_piece(
        Piece::Bishop,
        3,
        8,
        game_assets.piece_atlas.clone(),
        game_assets.pieces[&Piece::Bishop] + 6,
        commands,
    );
    spawn_piece(
        Piece::Bishop,
        6,
        8,
        game_assets.piece_atlas.clone(),
        game_assets.pieces[&Piece::Bishop] + 6,
        commands,
    );
    spawn_piece(
        Piece::Rook,
        1,
        8,
        game_assets.piece_atlas.clone(),
        game_assets.pieces[&Piece::Rook] + 6,
        commands,
    );
    spawn_piece(
        Piece::Rook,
        8,
        8,
        game_assets.piece_atlas.clone(),
        game_assets.pieces[&Piece::Rook] + 6,
        commands,
    );

    for i in 1..=BOARD_SIZE {
        spawn_piece(
            Piece::Pawn,
            i,
            7,
            game_assets.piece_atlas.clone(),
            game_assets.pieces[&Piece::Pawn] + 6,
            commands,
        );
    }
}
