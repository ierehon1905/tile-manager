mod generator_loop;
use base64::Engine as _;

use std::collections::{BTreeMap, HashMap};

use generator_loop::generator_loop;
use macroquad::{
    prelude::*,
    ui::{root_ui, Skin},
};
use nanoserde::{DeJson, SerJson, SerJsonState};

const TILE_SIZE: f32 = 16.0;
const MAP_WIDTH: usize = 36;
const MAP_HEIGHT: usize = 36;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct HexColor(u8, u8, u8);

impl DeJson for HexColor {
    fn de_json(
        state: &mut nanoserde::DeJsonState,
        input: &mut std::str::Chars,
    ) -> Result<Self, nanoserde::DeJsonErr> {
        let raw_hex = state.as_string().unwrap();
        // make cusstom err
        let r = u8::from_str_radix(&raw_hex[1..3], 16).map_err(|_| nanoserde::DeJsonErr {
            col: state.col,
            line: state.line,
            msg: "invalid hex color".to_string(),
        })?;
        let g = u8::from_str_radix(&raw_hex[3..5], 16).map_err(|_| nanoserde::DeJsonErr {
            col: state.col,
            line: state.line,
            msg: "invalid hex color".to_string(),
        })?;
        let b = u8::from_str_radix(&raw_hex[5..7], 16).map_err(|_| nanoserde::DeJsonErr {
            col: state.col,
            line: state.line,
            msg: "invalid hex color".to_string(),
        })?;

        let color = HexColor(r, g, b);

        state.next_tok(input)?;

        Ok(color)
    }
}

impl SerJson for HexColor {
    fn ser_json(&self, _d: usize, s: &mut SerJsonState) {
        let hex = format!(
            "#{:02x}{:02x}{:02x}",
            self.0 as u8, self.1 as u8, self.2 as u8
        );

        s.label(&hex);
    }
}

#[derive(Clone, Debug, Default, DeJson, SerJson, PartialEq, Eq)]
pub struct TileInfo {
    name: String,
    #[nserde(default)]
    color: HexColor,
    friends_top: Vec<String>,
    friends_right: Vec<String>,
    friends_bottom: Vec<String>,
    friends_left: Vec<String>,
    slots_top: Vec<String>,
    slots_right: Vec<String>,
    slots_bottom: Vec<String>,
    slots_left: Vec<String>,
    image: String,
    weights_top: Option<HashMap<String, i32>>,
    weights_right: Option<HashMap<String, i32>>,
    weights_bottom: Option<HashMap<String, i32>>,
    weights_left: Option<HashMap<String, i32>>,
}

impl PartialOrd for TileInfo {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.name.partial_cmp(&other.name)
    }
}

impl Ord for TileInfo {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

#[derive(Clone, Debug, Default, DeJson, SerJson)]
#[nserde(transparent)]
pub struct TilesConfig(Vec<TileInfo>);

#[macroquad::main("BasicShapes")]
async fn main() {
    let raw_config = load_string("config.json").await.unwrap();
    let config = TilesConfig::deserialize_json(&raw_config).unwrap();

    // load all jsons in tiles folder
    let mut tiles = BTreeMap::new();
    let mut images = BTreeMap::new();

    for tile in config.0 {
        let base64_image = &tile.image["data:image/png;base64,".len()..];

        let bytes = base64::engine::general_purpose::STANDARD
            .decode(base64_image)
            .unwrap();

        let image = image::load_from_memory(&bytes).unwrap();

        let width = image.width() as u16;
        let height = image.height() as u16;
        println!("{} {} {}", &tile.name, width, height);
        let rgba_pixels = image.to_rgba8().into_raw();

        println!("{:?}", bytes);

        let texture = Texture2D::from_rgba8(width, height, &rgba_pixels);
        images.insert(tile.name.clone(), texture);

        // tiles.insert(tile);
        tiles.insert(tile.name.clone(), tile);
    }

    // let all_tile_ids: Vec<String> = tiles.iter().map(|tile| tile.name.clone()).collect();
    let all_tile_ids: Vec<String> = tiles.keys().cloned().collect();

    let mut board = vec![vec![all_tile_ids.clone(); MAP_WIDTH]; MAP_HEIGHT];

    let mut play_mode = false;

    let label_style = root_ui().style_builder().font_size(40).build();
    // root_ui().default_skin().
    // root_ui().default_skin().button_style
    let button_style = root_ui()
        .style_builder()
        .font_size(40)
        .color(color_u8!(150, 150, 150, 255))
        .color_hovered(color_u8!(180, 180, 180, 255))
        .color_clicked(color_u8!(200, 200, 200, 255))
        .build();

    let custom_skin = Skin {
        label_style,
        button_style,
        ..root_ui().default_skin().clone()
    };

    loop {
        clear_background(WHITE);

        root_ui().push_skin(&custom_skin);

        generator_loop(&mut board, &tiles, &images, &all_tile_ids, &mut play_mode).await;

        root_ui().pop_skin();

        next_frame().await
    }
}
