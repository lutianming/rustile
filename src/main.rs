extern crate x11;
#[macro_use]
extern crate log;
extern crate env_logger;

extern crate rustile;
use rustile::window_manager::WindowManager;

fn main() {
    env_logger::init().unwrap();

    let mut wm = WindowManager::new();
    wm.run();
}
