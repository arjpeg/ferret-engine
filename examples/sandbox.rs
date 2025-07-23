use ferret_engine::prelude::*;
use legion::{system, systems::CommandBuffer};

struct Player {
    counter: u32,
    name: String,
}

fn main() {
    ferret_engine::init_logging();

    Application::builder()
        .add_startup_system(setup_system())
        .add_update_system(update_players_system())
        .run();
}

#[system]
fn setup(cmd: &mut CommandBuffer) {
    cmd.push((
        0u32,
        Player {
            counter: 0,
            name: "Player1".into(),
        },
    ));
}

#[system(for_each)]
fn update_players(player: &mut Player) {
    if player.counter % 100 == 0 {
        println!("{}: {}", player.name, player.counter);
    }
    player.counter += 1;
}
