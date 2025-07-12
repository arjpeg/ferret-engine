use crate::application::Application;

mod application;

#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn run() -> Result<(), wasm_bindgen::JsValue> {
    use wasm_bindgen::UnwrapThrowExt as _;

    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap_throw();

    Ok(Application::run().unwrap_throw())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn run() -> anyhow::Result<()> {
    env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .init();

    Application::run()
}
