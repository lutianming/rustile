use x11::xlib::Window;

use std::boxed::Box;

use super::layout;
use super::WindowManager;

pub struct Workspace {
    root: Window,
    windows: Vec<Window>,
    layout: Box<layout::Layout>
}

impl Workspace {
    pub fn add(&mut self, window: Window) {
        self.windows.push(window);
    }
    pub fn remove(&mut self, window: Window) {
        let index = self.contain(window);
        match index {
            Some(i) => { self.windows.remove(i); }
            None => {}
        };
    }
    pub fn contain(&self, window: Window) -> Option<usize>{
        self.windows.iter().position(|x| *x == window)
    }
    pub fn new() -> Workspace {
        Workspace {
            root: 0,
            windows: Vec::new(),
            layout: Box::new(layout::TilingLayout::new(layout::Direction::Vertical))
        }
    }

    pub fn config(&self, wm: &WindowManager) {
        debug!("size {}", self.windows.len());
        self.layout.configure(&self.windows, wm);
    }
}
