#![recursion_limit="1536"]

#[macro_use]
extern crate cfg_if;

extern crate wasm_bindgen;
extern crate web_sys;
extern crate yew;
use wasm_bindgen::prelude::*;

mod app;
mod connect_form;
mod ide_home;
mod new_project_dialog;
mod socket_agent;
mod terminal;
mod terminal_src;
mod file_manager;
mod editor;
mod editor_src;
mod new_file;

use app::App;


cfg_if! {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function to get better error messages if we ever panic.
    if #[cfg(feature = "console_error_panic_hook")] {
        extern crate console_error_panic_hook;
        use console_error_panic_hook::set_once as set_panic_hook;
    } else {
        #[inline]
        fn set_panic_hook() {}
    }
}

cfg_if! {
    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
    // allocator.
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

// Called by our JS entry point to run the example
#[wasm_bindgen]
pub fn run() {
    // If the `console_error_panic_hook` feature is enabled this will set a panic hook, otherwise
    // it will do nothing.


    set_panic_hook();
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<App>();
}
