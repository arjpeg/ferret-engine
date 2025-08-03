use ferret_engine::prelude::{
    Application, Camera2D, FrameTimer, Material2D, Mesh2D, Shape2D, Transform, World,
};
use glam::Quat;

fn main() {
    ferret_engine::init_logging();

    Application::builder()
        .add_startup_system(setup)
        .add_update_system(rotate)
        .run();
}

struct Rotate {
    speed: f32,
}

fn setup(world: &mut World) {
    world.spawn((Camera2D { half_width: 10.0 }, Transform::default()));

    world.spawn((
        Material2D::FlatColor {
            r: 0.6,
            g: 0.34,
            b: 0.45,
        },
        Mesh2D(Shape2D::Rectangle),
        Transform::default(),
        Rotate { speed: 2.0 },
    ));
}

fn rotate(world: &mut World) {
    let dt = world.get_resource::<FrameTimer>().dt();

    for (_, (transform, rotation)) in world.query_mut::<(&mut Transform, &Rotate)>() {
        transform.rotation *= Quat::from_rotation_z(rotation.speed * dt);
    }
}
