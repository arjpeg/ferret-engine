use ferret_engine::prelude::*;
use glam::Vec3;
use legion::{system, systems::CommandBuffer};

fn main() {
    ferret_engine::init_logging();

    Application::builder()
        .add_startup_system(setup_system())
        .run();
}

#[system]
fn setup(cmd: &mut CommandBuffer) {
    cmd.push((
        Mesh2D(Shape2D::Rectangle),
        Material2D::FlatColor {
            r: 0.6,
            g: 0.3,
            b: 0.6,
        },
        Transform {
            translation: Vec3::new(0.5, 0.0, 0.0),
            scale: Vec3::splat(0.7),
            ..Default::default()
        },
    ));

    cmd.push((
        Mesh2D(Shape2D::Rectangle),
        Material2D::FlatColor {
            r: 0.4,
            g: 0.2,
            b: 0.6,
        },
        Transform::with_scale(Vec3::splat(0.5)),
    ));
}
