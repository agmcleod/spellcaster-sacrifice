use std::ops::Deref;

use cgmath::{ortho, Matrix4, SquareMatrix, Transform};
use gfx::{self, texture, traits::FactoryExt};
use gfx_glyph::{GlyphBrush, Layout, Section};
use specs::World;

use crate::{
    assets::spritesheet::Frame,
    assets::spritesheet_map::SpritesheetMap,
    components::{
        camera::Camera, color::Color, shape::Shape, text::Text,
        transform::Transform as ComponentTransform,
    },
    loader::Texture,
    SCREEN_HEIGHT, SCREEN_WIDTH,
};

mod tiled;

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
        let offset: u32 = (self.batch.len() * 4) as u32;
        let index_data: Vec<u32> = vec![
            0 + offset,
            1 + offset,
            2 + offset,
            2 + offset,
            3 + offset,
            0 + offset,
        ];
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
    ) where
        C: gfx::CommandBuffer<R>,
        F: gfx::Factory<R>,
    {
        if self.batch.len() > 0 {
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

    pub fn reset_transform(&mut self) {
        self.model = Matrix4::identity().into();
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
    ) where
        C: gfx::CommandBuffer<R>,
        F: gfx::Factory<R>,
    {
        let camera_res = world.read_resource::<Camera>();
        let camera = camera_res.deref();
        let w = transform.size.x as f32;
        let h = transform.size.y as f32;

        let mut tx = 0.0;
        let mut ty = 0.0;
        let mut tx2 = 1.0;
        let mut ty2 = 1.0;

        if let Some(frame_name) = frame_name {
            let sheet_name = spritesheet_map.frame_to_sheet_name.get(frame_name).unwrap();
            if self.last_sheet != *sheet_name {
                if self.batch.len() > 0 {
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
                }
                self.last_sheet = sheet_name.clone();
            }
            let (spritesheet, texture) = spritesheet_map.sheet_name_map.get(sheet_name).unwrap();
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
        } else {
            if self.batch.len() > 0 && self.last_sheet != "white_texture" {
                let (_, texture) = spritesheet_map
                    .sheet_name_map
                    .get(&self.last_sheet)
                    .unwrap();

                let texture = self.create_drawable_texture(factory, texture);
                self.draw_verticies(encoder, factory, texture, &camera);
            }
            self.last_sheet = "white_texture".to_string();
        };

        let color = if let Some(color) = color {
            color.0
        } else {
            [1.0; 4]
        };

        add_quad_to_batch(&mut self.batch, color, w, h, tx, ty, tx2, ty2);
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
    ) where
        R: gfx::Resources,
        C: gfx::CommandBuffer<R>,
        F: gfx::Factory<R>,
    {
        let absolute_pos = transform.get_absolute_pos();
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
                absolute_pos.x * hidpi_factor * scale_from_base_res.0,
                absolute_pos.y * hidpi_factor * scale_from_base_res.1,
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

    pub fn transform(&mut self, transform: &ComponentTransform, undo: bool) {
        let mut transform_mat = Matrix4::from_translation(*transform.get_pos());
        if undo {
            transform_mat = transform_mat.inverse_transform().unwrap();
            self.model = self.model.concat(&transform_mat);
        } else {
            self.model = self.model.concat(&transform_mat);
        }
    }
}

fn add_quad_to_batch(
    batch: &mut Vec<Vertex>,
    color: [f32; 4],
    w: f32,
    h: f32,
    tx: f32,
    ty: f32,
    tx2: f32,
    ty2: f32,
) {
    batch.push(Vertex {
        pos: [0.0, 0.0, 0.0],
        uv: [tx, ty],
        color: color,
    });
    batch.push(Vertex {
        pos: [w, 0.0, 0.0],
        uv: [tx2, ty],
        color: color,
    });
    batch.push(Vertex {
        pos: [w, h, 0.0],
        uv: [tx2, ty2],
        color: color,
    });
    batch.push(Vertex {
        pos: [0.0, h, 0.0],
        uv: [tx, ty2],
        color: color,
    });
}
