use bevy::{prelude::*, utils::HashMap};

const PIECE_SIZE: usize = 64;

#[derive(PartialEq, Eq, Hash)]
enum Piece {
    King,
    Queen,
    Knight,
    Pawn,
    Bishop,
    Rook,
}

#[derive(Resource)]
struct GameAssets {
    piece_atlas: Handle<TextureAtlas>,
    pieces: HashMap<Piece, usize>,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(load_assets)
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
            (Piece::King, 1),
            (Piece::Queen, 2),
            (Piece::Knight, 3),
            (Piece::Pawn, 4),
            (Piece::Bishop, 5),
            (Piece::Rook, 6),
        ]),
    });
}
