use bevy::{prelude::*, utils::HashMap, window::PrimaryWindow};

const PIECE_SIZE: usize = 60;
const BOARD_SIZE: usize = 8;

#[derive(PartialEq, Eq, Hash)]
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

#[derive(Resource)]
struct GameAssets {
    piece_atlas: Handle<TextureAtlas>,
    pieces: HashMap<Piece, usize>,
}

#[derive(Resource)]
struct BoardPopulationDone(bool);

fn main() {
    App::new()
        .insert_resource(BoardPopulationDone(false))
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
        commands.spawn(SpriteSheetBundle {
            sprite: TextureAtlasSprite {
                custom_size: Some(Vec2::splat(PIECE_SIZE as f32)),
                index: game_assets.pieces[&Piece::Pawn],
                ..default()
            },
            texture_atlas: game_assets.piece_atlas.clone(),
            transform: Transform::from_xyz((PIECE_SIZE / 2) as f32, (PIECE_SIZE / 2) as f32, 0.0),
            ..default()
        });

        population_done.0 = true;
    }
}
