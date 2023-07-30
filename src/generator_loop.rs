use std::collections::{BTreeMap, BTreeSet};

use macroquad::{prelude::*, rand::ChooseRandom, ui::root_ui};

use crate::{TileInfo, MAP_HEIGHT, MAP_WIDTH, TILE_SIZE};

pub async fn generator_loop(
    board: &mut Vec<Vec<Vec<String>>>,
    tiles: &BTreeSet<TileInfo>,
    images: &BTreeMap<String, Texture2D>,
    all_tile_ids: &Vec<String>,
    play_mode: &mut bool,
) {
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
        step(board, &tiles);
    }

    if root_ui().button(vec2(100.0, screen_height() - 20.0), "Play") {
        // play_mode = !play_mode;
        *play_mode = !*play_mode;
        println!("Play {play_mode}");
    }

    if *play_mode {
        step(board, &tiles);
    }

    if root_ui().button(vec2(150.0, screen_height() - 20.0), "Solve") {
        while step(board, &tiles) {}
    }

    if root_ui().button(vec2(50.0, screen_height() - 20.0), "Reset") {
        for y in 0..MAP_HEIGHT {
            for x in 0..MAP_WIDTH {
                board[y][x] = all_tile_ids.clone();
            }
        }
    }
}

enum Side {
    Top,
    Right,
    Bottom,
    Left,
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

    collapse(board, tiles, x, y);

    true
}

fn collapse(board: &mut Vec<Vec<Vec<String>>>, tiles: &BTreeSet<TileInfo>, x: usize, y: usize) {
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
        .map(|picked_tile| tiles.iter().find(|tile| tile.id == *picked_tile).unwrap())
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
