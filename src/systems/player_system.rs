use std::ops::Deref;

use specs::{Join, Read, ReadStorage, System, WriteStorage};

use crate::{
    components::{AnimationSheet, DeltaTime, Input, Player, Sprite, Transform},
    SCREEN_HEIGHT, SCREEN_WIDTH,
};

const VELOCITY: f32 = 50.0;

pub struct PlayerSystem;

impl PlayerSystem {
    pub fn new() -> Self {
        PlayerSystem {}
    }
}

impl<'a> System<'a> for PlayerSystem {
    type SystemData = (
        WriteStorage<'a, AnimationSheet>,
        Read<'a, DeltaTime>,
        Read<'a, Input>,
        ReadStorage<'a, Player>,
        WriteStorage<'a, Transform>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut animation_sheet_storage,
            delta_time_storage,
            input,
            player_storage,
            mut transform_storage,
        ) = data;

        let dt = delta_time_storage.deref().dt;

        for (animation_sheet, _, transform) in (
            &mut animation_sheet_storage,
            &player_storage,
            &mut transform_storage,
        )
            .join()
        {
            let mut moving = false;
            if input.is_pressed("left") {
                if animation_sheet.current_animation != "right" {
                    animation_sheet.set_current_animation("right");
                }
                if !transform.flip {
                    transform.flip = true;
                }
                transform.translate_x(-VELOCITY * dt);
                moving = true;
            }
            if input.is_pressed("right") {
                if animation_sheet.current_animation != "right" {
                    animation_sheet.set_current_animation("right");
                }
                if transform.flip {
                    transform.flip = false;
                }
                transform.translate_x(VELOCITY * dt);
                moving = true;
            }
            if input.is_pressed("up") {
                if animation_sheet.current_animation != "up" {
                    animation_sheet.set_current_animation("up");
                    transform.flip = false;
                }
                transform.translate_y(-VELOCITY * dt);
                moving = true;
            }
            if input.is_pressed("down") {
                if animation_sheet.current_animation != "down" {
                    animation_sheet.set_current_animation("down");
                    transform.flip = false;
                }
                transform.translate_y(VELOCITY * dt);
                moving = true;
            }

            animation_sheet.playing = moving;
            if moving {
                let pos = transform.get_pos();
                if pos.x < 0.0 {
                    transform.set_pos(0.0, pos.y, pos.z);
                } else if pos.x > SCREEN_WIDTH as f32 - transform.size.x as f32 {
                    transform.set_pos(SCREEN_WIDTH as f32 - transform.size.x as f32, pos.y, pos.z);
                } else if pos.y < 0.0 {
                    transform.set_pos(pos.x, 0.0, pos.z);
                } else if pos.y > SCREEN_HEIGHT as f32 - transform.size.y as f32 {
                    transform.set_pos(pos.x, SCREEN_HEIGHT as f32 - transform.size.y as f32, pos.z);
                }
            }
        }
    }
}
