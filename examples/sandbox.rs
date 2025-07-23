use ferret_engine::application::Application;
use legion::system;

fn main() {
    ferret_engine::init_logging();

    Application::builder()
        .add_system(hello_world_system())
        .run();
}

#[system]
fn hello_world() {
    println!("hello, world!");
}
