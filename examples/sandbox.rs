use ferret_engine::prelude::{
    Application, Camera2D, FrameTimer, InputState, Material2D, Mesh2D, Resources, Shape2D,
    Transform, World,
};
use glam::{Quat, Vec3, vec3};
use hecs::With;
use winit::keyboard::KeyCode;

fn main() {
    ferret_engine::init_logging();

    Application::builder()
        .add_startup_system(setup)
        .add_update_system(rotate)
        .add_update_system(handle_player_input)
        .run();
}

struct Rotate {
    speed: f32,
}

struct Player;

fn setup(world: &mut World, _: &mut Resources) {
    world.spawn((Camera2D { half_width: 10.0 }, Transform::default()));

    world.spawn((
        Material2D::FlatColor {
            r: 0.6,
            g: 0.34,
            b: 0.45,
        },
        Mesh2D(Shape2D::Square),
        Transform::default(),
        Rotate { speed: 2.0 },
    ));

    world.spawn((
        Material2D::FlatColor {
            r: 0.23,
            g: 0.56,
            b: 0.25,
        },
        Mesh2D(Shape2D::Square),
        Transform::with_translation(vec3(-4.0, 0.0, 0.0)),
        Player,
    ));
}

fn rotate(world: &mut World, resources: &mut Resources) {
    let dt = resources.get::<FrameTimer>().dt();

    for (_, (transform, rotation)) in world.query_mut::<(&mut Transform, &Rotate)>() {
        transform.rotation *= Quat::from_rotation_z(rotation.speed * dt);
    }
}

fn handle_player_input(world: &mut World, resources: &mut Resources) {
    let dt = resources.get::<FrameTimer>().dt();
    let input = resources.get::<InputState>();

    let translation_speed = 5.0;
    let rotation_speed = 90.0f32.to_radians();

    for (_, transform) in world.query_mut::<With<&mut Transform, &Player>>() {
        let mut translation = Vec3::ZERO;
        let mut rotation = Quat::IDENTITY;

        if input.key_held(KeyCode::KeyW) {
            translation += Vec3::Y;
        }
        if input.key_held(KeyCode::KeyS) {
            translation += Vec3::NEG_Y;
        }
        if input.key_held(KeyCode::KeyD) {
            translation += Vec3::X;
        }
        if input.key_held(KeyCode::KeyA) {
            translation += Vec3::NEG_X;
        }

        if input.key_pressed(KeyCode::ArrowLeft) {
            rotation = Quat::from_rotation_z(rotation_speed);
        }
        if input.key_pressed(KeyCode::ArrowRight) {
            rotation = Quat::from_rotation_z(rotation_speed * -1.0);
        }

        transform.translation += translation * translation_speed * dt;
        transform.rotation *= rotation;
    }
}
