extern crate libc;

use x11::xlib;
use x11::xlib::Window;

use std::boxed::Box;
use std::collections::HashMap;
use super::layout;
use super::WindowManager;

pub struct Workspace {
    root: Window,
    windows: Vec<Window>,
    pub layout: Box<layout::Layout>
}

impl Workspace {
    pub fn new() -> Workspace {
        Workspace {
            root: 0,
            windows: Vec::new(),
            layout: Box::new(layout::TilingLayout::new(layout::Direction::Horizontal))
        }
    }

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

    pub fn change_layout(&mut self, layout_type: layout::Type) {
        let t = self.layout.get_type();
        if t == layout_type {
            debug!("layout toggle");
            self.layout.toggle();
        }
        else{
            match layout_type {
                layout::Type::Tiling => {
                    debug!("change layout to Tiling");
                    let tmp = layout::TilingLayout::new(layout::Direction::Horizontal);
                    self.layout = Box::new(tmp);
                }
            }
        }
    }

    pub fn config(&self, display: *mut xlib::Display, screen_num: libc::c_int) {
        debug!("size {}", self.windows.len());
        self.layout.configure(&self.windows, display, screen_num);
    }
}

pub struct Workspaces {
    current: char,
    pub spaces: HashMap<char, Workspace>,

}

impl Workspaces {
    pub fn new() -> Workspaces {
        Workspaces {
            current: '1',
            spaces: HashMap::new()
        }
    }

    pub fn create_workspace(&mut self, key: char) {
        if !self.spaces.contains_key(&key) {
            let space = Workspace::new();
            self.spaces.insert(key, space);
        }
    }

    pub fn current_workspace(&mut self) -> &mut Workspace{
        self.spaces.get_mut(&self.current).unwrap()
    }

}
