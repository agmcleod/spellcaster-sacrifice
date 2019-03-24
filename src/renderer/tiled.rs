use tiled;

use crate::renderer::Vertex;

pub struct TiledMap {
    pub data: Vec<Vertex>,
}

impl TiledMap {
    pub fn new(tilemap: &tiled::Map) -> TiledMap {
        let mut vertex_data: Vec<Vertex> = Vec::new();

        let mut index = 0;
        for (layer_index, layer) in tilemap.layers.iter().enumerate() {
            let layer_z = layer_index as f32 + 1.0;
            for (row, cols) in layer.tiles.iter().enumerate() {
                for (col, cell) in cols.iter().enumerate() {
                    if *cell != 0 {
                        let x = col as f32 * tilemap.tile_width as f32;
                        let y = (tilemap.tile_height * tilemap.height) as f32
                            - (row as f32 * tilemap.tile_height as f32)
                            - tilemap.tile_height as f32;
                        let w = tilemap.tile_width as f32;
                        let h = tilemap.tile_height as f32;
                        vertex_data.push(Vertex {
                            pos: [x, y, layer_z],
                            uv: [0.0, 0.0],
                            color: [1.0, 1.0, 1.0, 1.0],
                        });
                        vertex_data.push(Vertex {
                            pos: [x + w, y, layer_z],
                            uv: [0.0, 0.0],
                            color: [1.0, 1.0, 1.0, 1.0],
                        });
                        vertex_data.push(Vertex {
                            pos: [x + w, y + h, layer_z],
                            uv: [0.0, 0.0],
                            color: [1.0, 1.0, 1.0, 1.0],
                        });
                        vertex_data.push(Vertex {
                            pos: [x, y + h, layer_z],
                            uv: [0.0, 0.0],
                            color: [1.0, 1.0, 1.0, 1.0],
                        });

                        // build out texture coord data
                        for tileset in tilemap.tilesets.iter() {
                            let image = &tileset.images[0];
                            // just handling a single image for now
                            if tileset.first_gid as usize + tileset.tiles.len() - 1
                                <= *cell as usize
                            {
                                let iw = image.width as u32;
                                let ih = image.height as u32;
                                let tiles_wide = iw / (tileset.tile_width + tileset.spacing);
                                let tiles_high = ih / (tileset.tile_height + tileset.spacing);
                                let tile_width_uv = tileset.tile_width as f32 / iw as f32;
                                let tile_height_uv = tileset.tile_height as f32 / ih as f32;
                                let x = ((*cell as u32 - 1u32) % tiles_wide) as f32
                                    + tileset.margin as f32 / iw as f32;
                                let y = ((*cell as u32 - 1u32) / tiles_wide) as f32
                                    + tileset.margin as f32 / ih as f32;
                                let tiles_wide = tiles_wide as f32;
                                let tiles_high = tiles_high as f32;
                                vertex_data[index].uv[0] = x / tiles_wide;
                                vertex_data[index].uv[1] = y / tiles_high + tile_height_uv;
                                vertex_data[index + 1].uv[0] = x / tiles_wide + tile_width_uv;
                                vertex_data[index + 1].uv[1] = y / tiles_high + tile_height_uv;
                                vertex_data[index + 2].uv[0] = x / tiles_wide + tile_width_uv;
                                vertex_data[index + 2].uv[1] = y / tiles_high;
                                vertex_data[index + 3].uv[0] = x / tiles_wide;
                                vertex_data[index + 3].uv[1] = y / tiles_high;
                                break;
                            }
                        }

                        index += 4;
                    }
                }
            }
        }

        TiledMap { data: vertex_data }
    }
}
