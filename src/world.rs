use specs::World;

use crate::components::delta_time::DeltaTime;

pub fn setup_world(world: &mut World) {
    world.add_resource(DeltaTime::default());
}
