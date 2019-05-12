use std::ops::Deref;

use cgmath::{ortho, Matrix4, SquareMatrix, Vector3};
use gfx::{self, texture, traits::FactoryExt};
use gfx_glyph::{GlyphBrush, Layout, Section};
use specs::World;

use crate::{
    assets::spritesheet::Frame,
    assets::spritesheet_map::SpritesheetMap,
    components::{Camera, Color, Shape, Sprite, Text, Transform as ComponentTransform},
    loader::Texture,
    SCREEN_HEIGHT, SCREEN_WIDTH,
};

pub type ColorFormat = gfx::format::Srgba8;
pub type DepthFormat = gfx::format::Depth;

gfx_defines! {
    vertex Vertex {
        pos: [f32; 3] = "a_Pos",
        uv: [f32; 2] = "a_Uv",
        color: [f32; 4] = "a_Color",
    }

    constant Projection {
        model: [[f32; 4]; 4] = "u_Model",
        proj: [[f32; 4]; 4] = "u_Proj",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        projection_cb: gfx::ConstantBuffer<Projection> = "b_Projection",
        tex: gfx::TextureSampler<[f32; 4]> = "t_Texture",
        out: gfx::BlendTarget<ColorFormat> = ("Target0", gfx::state::ColorMask::all(), gfx::preset::blend::ALPHA),
        depth: gfx::DepthTarget<DepthFormat> = gfx::preset::depth::LESS_EQUAL_WRITE,
    }
}

pub fn get_ortho() -> Matrix4<f32> {
    let mut m = ortho(
        0.0,
        SCREEN_WIDTH as f32,
        SCREEN_HEIGHT as f32,
        0.0,
        100.0,
        0.0,
    );

    m.z.z *= -1.0;
    m
}

#[derive(Clone)]
pub struct WindowTargets<R: gfx::Resources> {
    pub color: gfx::handle::RenderTargetView<R, ColorFormat>,
    pub depth: gfx::handle::DepthStencilView<R, DepthFormat>,
}

pub struct Renderer<R: gfx::Resources> {
    pso: gfx::PipelineState<R, pipe::Meta>,
    projection: Projection,
    model: Matrix4<f32>,
    pub target: WindowTargets<R>,
    color_texture: (
        gfx::handle::ShaderResourceView<R, [f32; 4]>,
        gfx::handle::Sampler<R>,
    ),
    last_sheet: String,
    batch: Vec<Vertex>,
}

impl<R> Renderer<R>
where
    R: gfx::Resources,
{
    pub fn new<F>(factory: &mut F, target: WindowTargets<R>) -> Renderer<R>
    where
        F: gfx::Factory<R>,
    {
        use gfx::traits::FactoryExt;

        let pso = factory
            .create_pipeline_simple(
                include_bytes!("shaders/basic.glslv"),
                include_bytes!("shaders/basic.glslf"),
                pipe::new(),
            )
            .unwrap();

        let texels = [[0xff, 0xff, 0xff, 0xff]];
        let (_, texture_view) = factory
            .create_texture_immutable::<ColorFormat>(
                texture::Kind::D2(1, 1, texture::AaMode::Single),
                texture::Mipmap::Allocated,
                &[&texels],
            )
            .unwrap();

        let sinfo =
            texture::SamplerInfo::new(texture::FilterMethod::Mipmap, texture::WrapMode::Clamp);

        Renderer {
            pso: pso,
            projection: Projection {
                model: Matrix4::identity().into(),
                proj: get_ortho().into(),
            },
            model: Matrix4::identity(),
            target,
            color_texture: (texture_view, factory.create_sampler(sinfo)),
            last_sheet: String::new(),
            batch: Vec::new(),
        }
    }

    fn create_drawable_texture<F>(
        &self,
        factory: &mut F,
        texture: &Texture<R>,
    ) -> (
        gfx::handle::ShaderResourceView<R, [f32; 4]>,
        gfx::handle::Sampler<R>,
    )
    where
        F: gfx::Factory<R>,
    {
        (
            texture.clone(),
            factory.create_sampler(texture::SamplerInfo::new(
                texture::FilterMethod::Bilinear,
                texture::WrapMode::Clamp,
            )),
        )
    }

    /**
     * Draw an arbitrary batch of data, with a texture
     */
    pub fn draw_batch<F, C>(
        &mut self,
        batch: &Vec<Vertex>,
        encoder: &mut gfx::Encoder<R, C>,
        world: &World,
        factory: &mut F,
        spritesheet_map: &SpritesheetMap<R>,
        texture_name: &String,
        texture: &Texture<R>,
    ) where
        R: gfx::Resources,
        C: gfx::CommandBuffer<R>,
        F: gfx::Factory<R>,
    {
        let camera_res = world.read_resource::<Camera>();
        let camera = camera_res.deref();
        // flush renderer batch if it has stuff
        self.flush(
            encoder,
            factory,
            spritesheet_map,
            &camera,
            texture_name,
            false,
        );

        self.last_sheet = texture_name.to_owned();

        // setting capacity to 1.5x, as we have 6 indicies per 4 vertices
        let mut index_data: Vec<u32> = Vec::with_capacity((batch.len() as f32 * 1.5) as usize);
        let mut offset = 0;
        batch.chunks(4).fold(&mut index_data, |data, _| {
            data.append(&mut vec![
                0 + offset,
                1 + offset,
                2 + offset,
                2 + offset,
                3 + offset,
                0 + offset,
            ]);
            offset += 4;
            data
        });
        let (vbuf, slice) = factory.create_vertex_buffer_with_slice(&batch, &index_data[..]);

        let tex = self.create_drawable_texture(factory, texture);
        let params = pipe::Data {
            vbuf: vbuf,
            projection_cb: factory.create_constant_buffer(1),
            tex,
            out: self.target.color.clone(),
            depth: self.target.depth.clone(),
        };

        self.projection.proj = (*camera).0.into();

        self.projection.model = self.model.into();

        encoder.update_constant_buffer(&params.projection_cb, &self.projection);
        encoder.draw(&slice, &self.pso, &params);
    }

    fn draw_verticies<F, C>(
        &mut self,
        encoder: &mut gfx::Encoder<R, C>,
        factory: &mut F,
        tex: (
            gfx::handle::ShaderResourceView<R, [f32; 4]>,
            gfx::handle::Sampler<R>,
        ),
        camera: &Camera,
    ) where
        R: gfx::Resources,
        C: gfx::CommandBuffer<R>,
        F: gfx::Factory<R>,
    {
        // initialize as 1.5x, since we use 6 indices for 4 vertices
        let mut index_data: Vec<u32> = Vec::with_capacity((self.batch.len() as f32 * 1.5) as usize);

        let mut offset = 0;
        for _ in self.batch.chunks(4) {
            index_data.push(0 + offset);
            index_data.push(1 + offset);
            index_data.push(2 + offset);
            index_data.push(2 + offset);
            index_data.push(3 + offset);
            index_data.push(0 + offset);
            offset += 4;
        }

        let (vbuf, slice) = factory.create_vertex_buffer_with_slice(&self.batch, &index_data[..]);

        let params = pipe::Data {
            vbuf: vbuf,
            projection_cb: factory.create_constant_buffer(1),
            tex: tex,
            out: self.target.color.clone(),
            depth: self.target.depth.clone(),
        };

        self.projection.proj = (*camera).0.into();

        self.projection.model = self.model.into();

        encoder.update_constant_buffer(&params.projection_cb, &self.projection);
        encoder.draw(&slice, &self.pso, &params);
    }

    pub fn flush<C, F>(
        &mut self,
        encoder: &mut gfx::Encoder<R, C>,
        factory: &mut F,
        spritesheet_map: &SpritesheetMap<R>,
        camera: &Camera,
        current_sheet: &str,
        force: bool,
    ) where
        C: gfx::CommandBuffer<R>,
        F: gfx::Factory<R>,
    {
        if self.batch.len() > 0 && (self.last_sheet != current_sheet || force) {
            let texture = if self.last_sheet == "white_texture" {
                self.color_texture.clone()
            } else {
                let (_, texture) = spritesheet_map
                    .sheet_name_map
                    .get(&self.last_sheet)
                    .unwrap();

                self.create_drawable_texture(factory, texture)
            };

            self.draw_verticies(encoder, factory, texture, &camera);
            self.batch.clear();
        }
    }

    pub fn render<C, F>(
        &mut self,
        encoder: &mut gfx::Encoder<R, C>,
        world: &World,
        factory: &mut F,
        transform: &ComponentTransform,
        frame_name: Option<&String>,
        spritesheet_map: &SpritesheetMap<R>,
        color: Option<&Color>,
        offset_position: &Vector3<f32>,
    ) where
        C: gfx::CommandBuffer<R>,
        F: gfx::Factory<R>,
    {
        let camera_res = world.read_resource::<Camera>();
        let camera = camera_res.deref();

        let mut tx = 0.0;
        let mut ty = 0.0;
        let mut tx2 = 1.0;
        let mut ty2 = 1.0;

        let (w, h) = if let Some(frame_name) = frame_name {
            let sheet_name = spritesheet_map.frame_to_sheet_name.get(frame_name).unwrap();
            self.flush(
                encoder,
                factory,
                spritesheet_map,
                camera,
                &sheet_name,
                false,
            );
            self.last_sheet = sheet_name.clone();
            let (spritesheet, _) = spritesheet_map.sheet_name_map.get(sheet_name).unwrap();
            let region = spritesheet
                .frames
                .iter()
                .filter(|frame| frame.filename == *frame_name)
                .collect::<Vec<&Frame>>()[0];
            let sw = spritesheet.meta.size.w as f32;
            let sh = spritesheet.meta.size.h as f32;
            tx = region.frame.x as f32 / sw;
            ty = region.frame.y as f32 / sh;
            tx2 = (region.frame.x as f32 + region.frame.w as f32) / sw;
            ty2 = (region.frame.y as f32 + region.frame.h as f32) / sh;

            if transform.flip {
                let temp = tx2;
                tx2 = tx;
                tx = temp;
            }

            (
                region.sprite_source_size.w as f32,
                region.sprite_source_size.h as f32,
            )
        } else {
            self.flush(
                encoder,
                factory,
                spritesheet_map,
                camera,
                "white_texture",
                false,
            );
            self.last_sheet = "white_texture".to_string();
            (transform.size.x as f32, transform.size.y as f32)
        };

        let color = if let Some(color) = color {
            color.0
        } else {
            [1.0; 4]
        };

        add_quad_to_batch(
            &mut self.batch,
            color,
            offset_position.x,
            offset_position.y,
            offset_position.z,
            w,
            h,
            tx,
            ty,
            tx2,
            ty2,
        );
    }

    pub fn render_shape<C, F>(
        &mut self,
        encoder: &mut gfx::Encoder<R, C>,
        world: &World,
        factory: &mut F,
        shape: &Shape,
    ) where
        R: gfx::Resources,
        C: gfx::CommandBuffer<R>,
        F: gfx::Factory<R>,
    {
        use std::ops::Deref;

        let camera_res = world.read_resource::<Camera>();
        let camera = camera_res.deref();

        let buffers = &shape.buffers;
        let (vbuf, slice) =
            factory.create_vertex_buffer_with_slice(&buffers.vertices[..], &buffers.indices[..]);

        let params = pipe::Data {
            vbuf: vbuf,
            projection_cb: factory.create_constant_buffer(1),
            tex: self.color_texture.clone(),
            out: self.target.color.clone(),
            depth: self.target.depth.clone(),
        };

        self.projection.proj = (*camera).0.into();
        self.projection.model = self.model.into();

        encoder.update_constant_buffer(&params.projection_cb, &self.projection);
        encoder.draw(&slice, &self.pso, &params);
    }

    pub fn render_text<C, F>(
        &mut self,
        encoder: &mut gfx::Encoder<R, C>,
        text: &Text,
        transform: &ComponentTransform,
        color: &Color,
        glyph_brush: &mut GlyphBrush<R, F>,
        hidpi_factor: f32,
        scale_from_base_res: &(f32, f32),
        offset_position: &Vector3<f32>,
    ) where
        R: gfx::Resources,
        C: gfx::CommandBuffer<R>,
        F: gfx::Factory<R>,
    {
        let mut scale = text.scale.clone();
        scale.x *= hidpi_factor * scale_from_base_res.0;
        scale.y *= hidpi_factor * scale_from_base_res.1;
        let section = Section {
            text: text.text.as_ref(),
            scale,
            bounds: (
                text.size.x as f32 * hidpi_factor,
                text.size.y as f32 * hidpi_factor,
            ),
            screen_position: (
                offset_position.x * hidpi_factor * scale_from_base_res.0,
                offset_position.y * hidpi_factor * scale_from_base_res.1,
            ),
            color: color.0,
            z: 0.0,
            layout: Layout::default().h_align(text.align),
            ..Section::default()
        };

        glyph_brush.queue(section);
        glyph_brush
            .draw_queued(encoder, &self.target.color, &self.target.depth)
            .unwrap();
    }
}

fn add_quad_to_batch(
    batch: &mut Vec<Vertex>,
    color: [f32; 4],
    x: f32,
    y: f32,
    z: f32,
    w: f32,
    h: f32,
    tx: f32,
    ty: f32,
    tx2: f32,
    ty2: f32,
) {
    batch.push(Vertex {
        pos: [x, y, z],
        uv: [tx, ty],
        color: color,
    });
    batch.push(Vertex {
        pos: [x + w, y, z],
        uv: [tx2, ty],
        color: color,
    });
    batch.push(Vertex {
        pos: [x + w, y + h, z],
        uv: [tx2, ty2],
        color: color,
    });
    batch.push(Vertex {
        pos: [x, y + h, z],
        uv: [tx, ty2],
        color: color,
    });
}
