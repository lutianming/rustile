extern crate libc;

use x11::xlib;
use x11::xlib::{ Window, Display };

use std::boxed::Box;
use std::collections::HashMap;
use super::layout;
use super::WindowManager;

pub struct Workspace {
    root: Window,
    windows: Vec<Window>,
    layout: Box<layout::Layout>
}

impl Workspace {
    pub fn new() -> Workspace {
        Workspace {
            root: 0,
            windows: Vec::new(),
            layout: Box::new(layout::TilingLayout::new(layout::Direction::Horizontal))
        }
    }

    pub fn p(&self) {
        for w in self.windows.iter() {
            println!("{}", w);
        }
    }
    pub fn add(&mut self, window: Window) {
        let index = self.contain(window);
        match index {
            Some(i) => {}
            None => { self.windows.push(window) ;}
        }
    }

    pub fn remove(&mut self, window: Window) {
        let index = self.contain(window);
        match index {
            Some(i) => { self.windows.remove(i); }
            None => {}
        };
    }

    pub fn get(&self, index: usize) -> Window{
        self.windows[index]
    }

    pub fn size(&self) -> usize {
        self.windows.len()
    }

    pub fn hide(&mut self, display: *mut Display) {
        for w in self.windows.iter() {
            unsafe{
                xlib::XUnmapWindow(display, *w);
            }
        }
    }

    pub fn show(&mut self, display: *mut Display) {
        for w in self.windows.iter() {
            unsafe{
                xlib::XMapWindow(display, *w);
            }
        }
    }

    pub fn contain(&self, window: Window) -> Option<usize>{
        self.windows.iter().position(|x| *x == window)
    }

    pub fn change_layout(&mut self, layout_type: layout::Type) {
        let t = self.layout.get_type();
        if t == layout_type {
            self.layout.toggle();
        }
        else{
            match layout_type {
                layout::Type::Tiling => {
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

    pub fn contain(&mut self, key: char) -> bool{
        self.spaces.contains_key(&key)
    }

    pub fn create(&mut self, key: char) {
        let space = Workspace::new();
        self.spaces.insert(key, space);
    }

    pub fn delete(&mut self, key: char) {
        self.spaces.remove(&key);
    }

    pub fn get(&mut self, key: char) -> Option<&mut Workspace>{
        self.spaces.get_mut(&key)
    }

    pub fn current(&mut self) -> &mut Workspace {
        self.spaces.get_mut(&self.current).unwrap()
    }

    pub fn current_name(&self) -> char {
        self.current
    }

    pub fn switch_current(&mut self, new: char, display: *mut Display){
        if new != self.current {
            if !self.contain(new) {
                self.create(new);
            }
            let old = self.current;
            self.current = new;

            match self.get(old) {
                Some(v) => {
                    v.hide(display);
                }
                None => {}
            }

            match self.get(new) {
                Some(v) => {
                    v.show(display);
                }
                None => {}
            }
        }
    }

    pub fn move_window(&mut self, window: Window, from: char, to: char){
        if self.get(to).is_none(){
            self.create(to);
        }

        if from != to {
            match self.get(from) {
                Some(w) => { w.remove(window); }
                None => {}
            }
            match self.get(to) {
                Some(w) => { w.add(window); }
                None => {}
            }
        }
    }
}
