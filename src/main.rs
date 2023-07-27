use std::collections::{BTreeMap, BTreeSet};

use macroquad::{prelude::*, rand::ChooseRandom, ui::root_ui};
use nanoserde::{DeJson, SerJson};

const TILE_SIZE: f32 = 50.0;
const MAP_WIDTH: usize = 15;
const MAP_HEIGHT: usize = 15;

#[derive(Clone, Debug, Default, DeJson, SerJson, PartialEq, Eq)]
struct TileInfo {
    id: String,
    color: (u8, u8, u8),
    friends: Vec<String>,
}

impl PartialOrd for TileInfo {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl Ord for TileInfo {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

#[macroquad::main("BasicShapes")]
async fn main() {
    // load all jsons in tiles folder
    let mut tiles = BTreeSet::new();
    let mut images = BTreeMap::new();
    for entry in std::fs::read_dir("tiles").unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().unwrap() == "json" {
            let json = load_string(path.to_str().unwrap()).await.unwrap();
            // load image in assets folder with the same name
            let tile: TileInfo = DeJson::deserialize_json(&json).unwrap();
            let id = tile.id.clone();
            tiles.insert(tile);

            // let image_path = path.with_extension("png");
            // path at assets folder and png extension
            let image_path = format!("assets/{}.png", id);

            let image = load_texture(&image_path).await.unwrap();

            images.insert(id, image);
        }
    }

    let all_tile_ids: Vec<String> = tiles.iter().map(|tile| tile.id.clone()).collect();

    // a MAP_WIDTH by MAP_HEIGHT array of tile Vecs of all tile ids
    let mut board = vec![vec![all_tile_ids.clone(); MAP_WIDTH]; MAP_HEIGHT];

    let mut play_mode = false;

    loop {
        clear_background(RED);

        // draw vertical lines
        for x in 0..MAP_WIDTH {
            draw_line(
                x as f32 * TILE_SIZE,
                0.0,
                x as f32 * TILE_SIZE,
                MAP_HEIGHT as f32 * TILE_SIZE,
                1.0,
                DARKGRAY,
            );
        }

        // draw horizontal lines
        for y in 0..MAP_HEIGHT {
            draw_line(
                0.0,
                y as f32 * TILE_SIZE,
                MAP_WIDTH as f32 * TILE_SIZE,
                y as f32 * TILE_SIZE,
                1.0,
                DARKGRAY,
            );
        }

        // draw tiles

        for y in 0..MAP_HEIGHT {
            for x in 0..MAP_WIDTH {
                let tile_ids = &board[y][x];
                let tile_count = tile_ids.len();
                let tile_x = x as f32 * TILE_SIZE;
                let tile_y = y as f32 * TILE_SIZE;
                let small_square_size = TILE_SIZE / tile_count as f32;
                // draw all possible tiles in one cell in a row
                for (i, tile_id) in tile_ids.iter().enumerate() {
                    let tile = tiles.iter().find(|tile| tile.id == *tile_id).unwrap();

                    if tile_count != 1 {
                        let tile_color = Color::new(
                            tile.color.0 as f32 / 255.0,
                            tile.color.1 as f32 / 255.0,
                            tile.color.2 as f32 / 255.0,
                            1.0,
                        );

                        draw_rectangle(
                            tile_x + i as f32 * small_square_size,
                            tile_y,
                            small_square_size,
                            small_square_size,
                            tile_color,
                        );
                    } else {
                        draw_texture_ex(
                            images.get(tile_id).unwrap(),
                            tile_x + i as f32 * small_square_size,
                            tile_y,
                            WHITE,
                            DrawTextureParams {
                                dest_size: Some(vec2(small_square_size, small_square_size)),
                                ..Default::default()
                            },
                        );
                    }
                }
            }
        }

        if root_ui().button(vec2(0.0, screen_height() - 20.0), "Step") {
            step(&mut board, &tiles);
        }

        if root_ui().button(vec2(100.0, screen_height() - 20.0), "Play") {
            play_mode = !play_mode;
            println!("Play {play_mode}");
        }

        if play_mode {
            step(&mut board, &tiles);
        }

        if root_ui().button(vec2(150.0, screen_height() - 20.0), "Solve") {
            while step(&mut board, &tiles) {}
        }

        if root_ui().button(vec2(50.0, screen_height() - 20.0), "Reset") {
            board = vec![vec![all_tile_ids.clone(); MAP_WIDTH]; MAP_HEIGHT];
        }

        next_frame().await
    }
}

fn step(board: &mut Vec<Vec<Vec<String>>>, tiles: &BTreeSet<TileInfo>) -> bool {
    // find cell with lowest number of possible tiles
    let mut min_tile_count = usize::MAX;
    let mut min_tile_count_cell = (0, 0);
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let tile_count = board[y][x].len();
            if tile_count < min_tile_count && tile_count > 1 {
                min_tile_count = tile_count;
                min_tile_count_cell = (x, y);
            }
        }
    }

    if min_tile_count == usize::MAX {
        return false;
    }

    let (x, y) = min_tile_count_cell;

    // find all possible tiles for this cell
    let possible_tiles = &board[y][x];
    // pick one of them and remove others
    // let picked_tile = &possible_tiles[0].clone();
    let picked_tile = &possible_tiles.choose().unwrap().clone();
    board[y][x] = vec![picked_tile.clone()];

    // find all friends of this cell
    let mut friends_coords = Vec::new();
    if x > 0 && board[y][x - 1].len() > 1 {
        friends_coords.push((x - 1, y));
    }
    if x < MAP_WIDTH - 1 && board[y][x + 1].len() > 1 {
        friends_coords.push((x + 1, y));
    }
    if y > 0 && board[y - 1][x].len() > 1 {
        friends_coords.push((x, y - 1));
    }
    if y < MAP_HEIGHT - 1 && board[y + 1][x].len() > 1 {
        friends_coords.push((x, y + 1));
    }

    // in friends remove tiles that are not friends with picked tile
    for (x, y) in friends_coords {
        let friends = &tiles
            .iter()
            .find(|tile| tile.id == *picked_tile)
            .unwrap()
            .friends;
        let mut new_friends = Vec::new();
        for friend in friends {
            if board[y][x].contains(friend) {
                new_friends.push(friend.clone());
            }
        }
        board[y][x] = new_friends;
    }

    true
}
