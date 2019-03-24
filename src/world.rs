use specs::World;

use crate::{
    components::{
        camera::Camera, color::Color, delta_time::DeltaTime, shape::Shape, sprite::Sprite,
        text::Text, transform::Transform,
    },
    renderer::get_ortho,
};

pub fn setup_world(world: &mut World) {
    world.add_resource(DeltaTime::default());
    world.add_resource(Camera(get_ortho()));

    world.register::<Color>();
    world.register::<Shape>();
    world.register::<Sprite>();
    world.register::<Text>();
    world.register::<Transform>();
}
