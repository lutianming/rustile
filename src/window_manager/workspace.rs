extern crate libc;

use x11::xlib::{ Window, Display };

use std::boxed::Box;
use std::collections::HashMap;
use super::layout;
use super::WindowManager;
use super::super::libx;
use super::super::libx::Context;

pub struct Workspace {
    pub screen_num: libc::c_int,
    focus: Option<Window>,
    pub clean: bool,                // if workspace is not clean, it needs to be reconfig before map
    pub visible: bool,
    windows: Vec<Window>,
    layout: Box<layout::Layout>
}

impl Workspace {
    pub fn new(screen_num: libc::c_int) -> Workspace {
        Workspace {
            screen_num: screen_num,
            focus: None,
            clean: true,
            visible: false,
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
            Some(i) => {
                // already there, do nothing
            }
            None => {
                self.windows.push(window) ;
                if self.visible {
                    //
                }
                self.clean = false;
            }
        }
    }

    pub fn remove(&mut self, window: Window) {
        let index = self.contain(window);
        match index {
            Some(i) => {
                self.windows.remove(i);
                self.clean = false;
                if self.visible {

                }
            }
            None => {}
        };
    }

    pub fn get(&self, index: usize) -> Window{
        self.windows[index]
    }

    pub fn size(&self) -> usize {
        self.windows.len()
    }

    pub fn hide(&mut self, context: Context) {
        for w in self.windows.iter() {
            libx::unmap_window(context, *w);
        }
        self.visible = false;
    }

    pub fn show(&mut self, context: Context) {
        for w in self.windows.iter() {
            libx::map_window(context, *w);
        }
        self.visible = true;
    }

    pub fn set_focus(&mut self, window: Window, context: Context) {
        match self.contain(window) {
            Some(i) => {
                libx::set_input_focus(context, window);
                self.focus = Some(window);
            }
            None => {}
        }
    }

    pub fn unset_focus(&mut self) {
        self.focus = None;
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

    pub fn config(&mut self, context: Context) {
        self.layout.configure(&self.windows, context, self.screen_num);
        self.clean = true;
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

    pub fn contain(&self, key: char) -> bool {
        self.spaces.contains_key(&key)
    }

    pub fn create(&mut self, key: char, screen_num: libc::c_int) {
        let space = Workspace::new(screen_num);
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

    pub fn switch_current(&mut self, new: char, context: Context){
        if new == self.current {
            return
        }

        let old = self.current;
        self.current = new;

        if !self.contain(old) || !self.contain(new) {
            return
        }

        match self.get(old) {
            Some(v) => {
                v.hide(context);
            }
            None => {}
        }

        match self.get(new) {
            Some(v) => {
                v.show(context);
            }
            None => {}
        }
    }

    pub fn move_window(&mut self, window: Window, from: char, to: char){
        if from == to {
            return
        }

        if !self.contain(from) || !self.contain(to) {
            return
        }

        match self.get(from) {
            Some(w) => { w.remove(window);}
            None => {}
        }

        match self.get(to) {
            Some(w) => { w.add(window);}
            None => {}
        }
    }
}
