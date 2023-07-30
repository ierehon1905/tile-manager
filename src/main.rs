mod generator_loop;

use std::collections::{BTreeMap, BTreeSet};

use generator_loop::generator_loop;
use macroquad::{
    hash,
    prelude::*,
    rand::ChooseRandom,
    ui::{
        canvas, root_ui,
        widgets::{self, Button, Group},
        Skin,
    },
};
use nanoserde::{DeJson, SerJson};

const TILE_SIZE: f32 = 50.0;
const MAP_WIDTH: usize = 9;
const MAP_HEIGHT: usize = 9;

#[derive(Clone, Debug, Default, DeJson, SerJson, PartialEq, Eq)]
pub struct TileInfo {
    id: String,
    color: (u8, u8, u8),
    friends_top: Vec<String>,
    friends_right: Vec<String>,
    friends_bottom: Vec<String>,
    friends_left: Vec<String>,
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

enum AppPage {
    Generator,
    Manager,
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

    let mut app_page = AppPage::Generator;

    loop {
        clear_background(WHITE);

        if root_ui().button(vec2(screen_width() - 50.0, 0.0), "Tab") {
            app_page = match app_page {
                AppPage::Generator => AppPage::Manager,
                AppPage::Manager => AppPage::Generator,
            }
        }

        match app_page {
            AppPage::Generator => {
                generator_loop(&mut board, &tiles, &images, &all_tile_ids, &mut play_mode).await;
            }
            AppPage::Manager => {
                Group::new(hash!(), vec2(screen_width(), screen_height()))
                    .position(vec2(0.0, 0.0))
                    .ui(&mut *root_ui(), |ui| {
                        ui.label(None, "Manager");
                        ui.label(None, "Manager 2");
                        // let mut canvas = ui.canvas();
                        // canvas.image(Rect::new(0.0, 0.0, 100.0, 100.0), &images["grass"]);
                        ui.label(None, "Manager 3");
                        widgets::Texture::new(images["stone"].weak_clone())
                            .position(None)
                            .size(100.0, 100.0)
                            .ui(ui);
                        // ui.texture(images["stone"].clone(), 1000.0, 1000.0);
                    });
            }
        }

        next_frame().await
    }
}
