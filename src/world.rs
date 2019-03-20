use specs::World;

use crate::components::{delta_time::DeltaTime, sprite::Sprite, transform::Transform};

pub fn setup_world(world: &mut World) {
    world.add_resource(DeltaTime::default());

    world.register::<Sprite>();
    world.register::<Transform>();
}
