use std::collections::{BTreeMap, HashMap};

use macroquad::{
    hash,
    prelude::*,
    ui::{root_ui, widgets},
};

use crate::{TileInfo, MAP_HEIGHT, MAP_WIDTH, TILE_SIZE};

pub async fn generator_loop(
    board: &mut Vec<Vec<Vec<String>>>,
    tiles: &BTreeMap<String, TileInfo>,
    images: &BTreeMap<String, Texture2D>,
    all_tile_ids: &Vec<String>,
    play_mode: &mut bool,
) {
    let offset_x = if MAP_WIDTH as f32 * TILE_SIZE < screen_width() {
        (screen_width() - MAP_WIDTH as f32 * TILE_SIZE) / 2.0
    } else {
        0.0
    };

    let tile_size = if MAP_WIDTH as f32 * TILE_SIZE > screen_width() {
        screen_width() / MAP_WIDTH as f32
    } else {
        TILE_SIZE
    };

    // draw vertical lines
    for x in 0..MAP_WIDTH {
        draw_line(
            x as f32 * tile_size + offset_x,
            0.0,
            x as f32 * tile_size + offset_x,
            MAP_HEIGHT as f32 * tile_size,
            1.0,
            DARKGRAY,
        );
    }

    // draw horizontal lines
    for y in 0..MAP_HEIGHT {
        draw_line(
            0.0 + offset_x,
            y as f32 * tile_size,
            MAP_WIDTH as f32 * tile_size + offset_x,
            y as f32 * tile_size,
            1.0,
            DARKGRAY,
        );
    }

    // draw tiles

    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let tile_ids = &board[y][x];
            let tile_count = tile_ids.len();
            let tile_x = x as f32 * tile_size;
            let tile_y = y as f32 * tile_size;
            for (i, tile_id) in tile_ids.iter().enumerate() {
                if tile_count > 4 {
                    continue;
                } else if tile_count > 1 && tile_count <= 4 {
                    let small_square_size = tile_size / tile_count as f32;
                    // let tile = tiles.iter().find(|tile| tile.name == *tile_id).unwrap();
                    let tile = tiles.get(tile_id).unwrap();
                    let tile_color = Color::new(
                        tile.color.0 as f32 / 255.0,
                        tile.color.1 as f32 / 255.0,
                        tile.color.2 as f32 / 255.0,
                        1.0,
                    );

                    draw_rectangle(
                        tile_x + i as f32 * small_square_size + offset_x,
                        tile_y,
                        small_square_size,
                        small_square_size,
                        tile_color,
                    );
                } else {
                    let small_square_size = tile_size / tile_count as f32;
                    draw_texture_ex(
                        images.get(tile_id).unwrap(),
                        tile_x + i as f32 * small_square_size + offset_x,
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

    widgets::Group::new(hash!(), vec2(screen_width(), 100.0))
        .position(vec2(0.0, screen_height() - 100.0))
        .ui(&mut *root_ui(), |ui| {
            ui.label(None, "Controls:");
            if ui.button(None, "Step") {
                step(board, &tiles);
            }

            ui.same_line(0.);

            if ui.button(None, "Play") {
                // play_mode = !play_mode;
                *play_mode = !*play_mode;
                println!("Play {play_mode}");
            }
            ui.same_line(0.);

            if ui.button(None, "Solve") {
                while step(board, &tiles) {}
            }
            ui.same_line(0.);

            if ui.button(None, "Reset") {
                for y in 0..MAP_HEIGHT {
                    for x in 0..MAP_WIDTH {
                        board[y][x] = all_tile_ids.clone();
                    }
                }
            }
            ui.same_line(0.);

            if *play_mode {
                step(board, &tiles);
            }
        });
    // root_ui().group(hash!(), vec2(screen_width(), 50.0), |ui| {

    // });
}

enum Side {
    Top,
    Right,
    Bottom,
    Left,
}

fn add_maps(map1: &HashMap<String, i32>, map2: &HashMap<String, i32>) -> HashMap<String, i32> {
    let mut result = map1.clone();

    for (key, value) in map2 {
        let current_value = result.entry(key.clone()).or_insert(0);
        *current_value += value;
    }

    result
}

fn get_preference_map(
    board: &Vec<Vec<Vec<String>>>,
    tiles: &BTreeMap<String, TileInfo>,
    x: usize,
    y: usize,
) -> HashMap<String, i32> {
    let mut preference_map = HashMap::new();

    let mut friends_coords = Vec::new();
    if x > 0 && board[y][x - 1].len() == 1 {
        friends_coords.push((x - 1, y, Side::Left));
    }
    if x < MAP_WIDTH - 1 && board[y][x + 1].len() == 1 {
        friends_coords.push((x + 1, y, Side::Right));
    }
    if y > 0 && board[y - 1][x].len() == 1 {
        friends_coords.push((x, y - 1, Side::Top));
    }
    if y < MAP_HEIGHT - 1 && board[y + 1][x].len() == 1 {
        friends_coords.push((x, y + 1, Side::Bottom));
    }

    for (x, y, side) in friends_coords {
        let side_possible_values = &board[y][x];
        // sum all weights for top possibilities
        let top_preference_map: HashMap<String, i32> =
            side_possible_values
                .iter()
                .fold(HashMap::new(), |acc, side_possibility| {
                    let side_possibility = tiles.get(side_possibility).unwrap();

                    let opposite_weights = match side {
                        Side::Top => &side_possibility.weights_bottom,
                        Side::Right => &side_possibility.weights_left,
                        Side::Bottom => &side_possibility.weights_top,
                        Side::Left => &side_possibility.weights_right,
                    };

                    if opposite_weights.is_none() {
                        return acc;
                    }

                    let opposite_weights = opposite_weights.as_ref().unwrap();

                    add_maps(&acc, opposite_weights)
                });

        preference_map = add_maps(&preference_map, &top_preference_map);
    }

    preference_map
}

fn choose_with_preferences(possible_tiles_with_preferences: &Vec<(&String, i32)>) -> String {
    let mut total_weight = 0.0;
    for (_, weight) in possible_tiles_with_preferences {
        total_weight += *weight as f32;
    }

    let mut random_weight = rand::gen_range(0.0, total_weight);

    for (tile, weight) in possible_tiles_with_preferences {
        random_weight -= *weight as f32;
        if random_weight <= 0.0 {
            return tile.to_string();
        }
    }

    panic!("Should not happen");
}

fn step(board: &mut Vec<Vec<Vec<String>>>, tiles: &BTreeMap<String, TileInfo>) -> bool {
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

    let preferences = get_preference_map(board, tiles, x, y);

    // println!("preferences: {:?}", preferences);

    let possible_tiles_with_preferences = possible_tiles
        .iter()
        .map(|tile| {
            let mut preference = *preferences.get(tile).unwrap_or(&0) * 50;

            if preference == 0 {
                preference = 1;
            } else if preference < 0 {
                preference = 0;
            }

            (tile, preference)
        })
        .collect::<Vec<_>>();

    let picked_tile = choose_with_preferences(&possible_tiles_with_preferences);
    // let picked_tile = possible_tiles.choose().unwrap().clone();
    // println!(
    //     "x: {}\ny: {}\npossible_tiles_with_preferences: {:?}\npicked_tile: {}",
    //     x, y, possible_tiles_with_preferences, picked_tile
    // );

    board[y][x] = vec![picked_tile];

    collapse(board, tiles, x, y);

    true
}

fn collapse(
    board: &mut Vec<Vec<Vec<String>>>,
    tiles: &BTreeMap<String, TileInfo>,
    x: usize,
    y: usize,
) {
    let picked_tiles = &board[y][x];

    // find all friends of this cell
    let mut friends_coords = Vec::new();
    if x > 0 && board[y][x - 1].len() > 1 {
        friends_coords.push((x - 1, y, Side::Left));
    }
    if x < MAP_WIDTH - 1 && board[y][x + 1].len() > 1 {
        friends_coords.push((x + 1, y, Side::Right));
    }
    if y > 0 && board[y - 1][x].len() > 1 {
        friends_coords.push((x, y - 1, Side::Top));
    }
    if y < MAP_HEIGHT - 1 && board[y + 1][x].len() > 1 {
        friends_coords.push((x, y + 1, Side::Bottom));
    }

    // in friends remove tiles that are not friends with picked tile
    // let picked_tile_infos = tiles.iter().find(|tile| tile.id == *picked_tiles).unwrap();
    // find all infos for picked tiles
    let picked_tile_infos = picked_tiles
        .iter()
        .map(|picked_tile| tiles.get(picked_tile).unwrap())
        .collect::<Vec<_>>();

    for (f_x, f_y, side) in friends_coords {
        let friend_possibilities_count = board[f_y][f_x].len();

        let side_fiends = picked_tile_infos
            .iter()
            .map(|picked_tile_info| match side {
                Side::Top => &picked_tile_info.friends_top,
                Side::Right => &picked_tile_info.friends_right,
                Side::Bottom => &picked_tile_info.friends_bottom,
                Side::Left => &picked_tile_info.friends_left,
            })
            .flatten()
            .collect::<Vec<_>>();

        let mut new_friends = Vec::new();

        for friend in &board[f_y][f_x] {
            if side_fiends.contains(&friend) {
                new_friends.push(friend.clone());
            }
        }

        board[f_y][f_x] = new_friends;

        if board[f_y][f_x].len() != friend_possibilities_count {
            collapse(board, tiles, f_x, f_y);
        }
    }
}
