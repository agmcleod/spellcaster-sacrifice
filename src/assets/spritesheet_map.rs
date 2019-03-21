use std::collections::HashMap;

use gfx;

use super::spritesheet::Spritesheet;
use crate::loader;

pub struct SpritesheetMap<R: gfx::Resources> {
    pub frame_to_sheet_name: HashMap<String, String>,
    pub sheet_name_map: HashMap<String, (Spritesheet, loader::Texture<R>)>,
}

impl<R> SpritesheetMap<R>
where
    R: gfx::Resources,
{
    fn new<F>(factory: &mut F, sheet_names: &[&str]) -> Self
    where
        F: gfx::Factory<R>,
    {
        let mut frame_to_sheet_name = HashMap::new();
        let mut sheet_name_map = HashMap::new();
        for name in sheet_names {
            let asset_data =
                loader::read_text_from_file(&format!("resources/{}.json", name)).unwrap();
            let spritesheet: Spritesheet = serde_json::from_str(asset_data.as_ref()).unwrap();
            let (asset_texture, _, _) =
                loader::gfx_load_texture(&format!("resources/{}.png", name), factory);

            for frame in &spritesheet.frames {
                frame_to_sheet_name.insert(frame.filename.clone(), name.to_string());
            }

            sheet_name_map.insert(name.to_string(), (spritesheet, asset_texture));
        }

        SpritesheetMap {
            frame_to_sheet_name,
            sheet_name_map,
        }
    }
}
