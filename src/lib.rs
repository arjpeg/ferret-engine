pub mod application;
pub mod prelude;
mod renderer;
mod timer;

#[cfg(target_family = "wasm")]
pub fn init_logging() {
    use wasm_bindgen::UnwrapThrowExt as _;

    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap_throw();
}

#[cfg(not(target_arch = "wasm32"))]
pub fn init_logging() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .filter_module("ferret-engine", log::LevelFilter::Debug)
        .init();
}
