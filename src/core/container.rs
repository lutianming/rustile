extern crate libc;
extern crate x11;

use x11::xlib;

use std::rc::{ Rc, Weak };
use std::cell::RefCell;
use std::boxed::Box;
use std::ptr;
use super::layout;
use super::super::libx;

pub const TITLE_HEIGHT: libc::c_int = 20;

pub enum Mode {
    Normal,
    Fullscreen,
}

pub struct Container {
    pub id: xlib::Window,
    pub visible: bool,
    pub titlebar_height: usize,
    pub pid: Option<xlib::Window>,
    parent: *mut Container,
    pub clients: Vec<Container>,
    pub mode: Mode,
    pub titlebar: Option<layout::Rectangle>,
    pub context: libx::Context,

    pub layout: layout::Type,
    pub direction: layout::Direction,
}

impl PartialEq for Container {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}


impl Container {
    pub fn new(context: libx::Context) -> Container{
        let pid = context.root;
        let attrs = libx::get_window_attributes(context, pid);

        let id = libx::create_window(context, pid,
                                     attrs.x, attrs.y,
                                     attrs.width as libc::c_uint,
                                     attrs.height as libc::c_uint);
        libx::select_input(context, id, xlib::SubstructureNotifyMask | xlib::SubstructureRedirectMask | xlib::ButtonPressMask);
        let mut c = Container::from_id(context, id);
        c
    }

    pub fn from_id(context: libx::Context, id: xlib::Window) -> Container {
        Container {
            context: context,
            clients: Vec::new(),
            pid: None,
            visible: false,
            id: id,
            mode: Mode::Normal,
            parent: ptr::null_mut(),
            titlebar: None,
            titlebar_height: 0,

            layout: layout::Type::Tiling,
            direction: layout::Direction::Horizontal,
        }
    }

    pub fn print_tree(&self, indent: i32) {
        for i in 0..indent {
            print!(" ");
        }
        let attrs = libx::get_window_attributes(self.context, self.id);
        println!("C {} pid:{} x:{} y:{} w:{} h:{}", self.id, self.pid.unwrap_or(0), attrs.x, attrs.y, attrs.width, attrs.height);
        for client in self.clients.iter() {
            client.print_tree(indent+4);
        }
    }

    // pub fn is_top(&self) -> bool {
    //     let top = self.get_top();
    //     match top {
    //         Some(w) => {
    //             w.id == self.id
    //         }
    //         Noen => false
    //     }
    // }

    pub fn is_empty(&self) -> bool{
        self.clients.is_empty()
    }

    pub fn size(&self) -> usize {
        self.clients.len()
    }
    pub fn add(&mut self, mut client: Container) {
        if client.pid.is_none() || client.pid.unwrap() != self.id {
            libx::reparent(self.context, client.id, self.id, 0, 0);
            client.pid = Some(self.id);
        }
        client.parent = self;
        self.clients.push(client);
    }

    pub fn insert(&mut self, index: usize,  mut client: Container) {
        if client.pid.is_none() || client.pid.unwrap() != self.id {
            libx::reparent(self.context, client.id, self.id, 0, 0);
            client.pid = Some(self.id);
        }
        client.parent = self;
        self.clients.insert(index, client);
    }

    pub fn remove(&mut self, id: xlib::Window) -> Option<Container>{
        let res = self.contain(id);
        match res {
            Some(index) => {
                let r = self.clients.remove(index);
                Some(r)
            }
            None => {
                for c in self.clients.iter_mut() {
                    let r = c.remove(id);
                    if r.is_some(){
                        return r;
                    }
                }
                None
            }
        }
    }

    pub fn get(&mut self, index: usize) -> Option<&mut Container> {
        self.clients.get_mut(index)
    }

    pub fn get_parent(&self) -> Option<&mut Container> {
        if self.parent == ptr::null_mut() {
            None
        }
        else{
            Some(unsafe{ &mut *self.parent })
        }
    }

    pub fn tree_search(&mut self, id: xlib::Window) -> Option<&mut Container>{
        if self.id == id {
            return Some(self);
        }

        for client in self.clients.iter_mut() {
            let r = client.tree_search(id);
            if r.is_some(){
                return r
            }
        }
        None
    }

    pub fn contain(&self, id: xlib::Window) -> Option<usize>{
        self.clients.iter().position(|x| (*x).id == id)
    }

    pub fn configure(&mut self, x: i32, y: i32, width: usize, height: usize) {
        libx::resize_window(self.context, self.id, x, y, width, height);
        // layout for children clients
        self.update_layout();
    }

    pub fn update_layout(&mut self) {
        layout::update_layout(self);
    }

    pub fn map(&self) {
        // self.visible = true;
        libx::map_window(self.context, self.id);
        for client in self.clients.iter() {
            client.map();
        }
    }

    pub fn unmap(&self) {
        // self.visible = false;
        for client in self.clients.iter() {
            client.unmap();
        }
        libx::unmap_window(self.context, self.id);
    }

    pub fn destroy(&self) -> bool {
        // can distroy only if it has no clients
        if self.clients.is_empty() {
            unsafe{
                xlib::XDestroyWindow(self.context.display, self.id);
                true
            }
        }
        else {
            false
        }
    }

    pub fn next_client(&mut self, id: xlib::Window) -> Option<&mut Container>{
        match self.contain(id) {
            Some(i) => {
                let mut next = i+1;
                if next == self.size() {
                    next = 0;
                }
                self.get(next)
            }
            None => {
                None
            }
        }
    }

    pub fn last_client(&mut self, id: xlib::Window) -> Option<&mut Container>{
        match self.contain(id) {
            Some(i) => {
                let mut last = i-1;
                if last < 0 {
                    last = self.size() - 1
                }
                self.get(last)
            }
            None => {
                None
            }
        }
    }

    pub fn change_layout(&mut self, layout_type: layout::Type) {
        if self.layout == layout_type {
            match self.direction {
                layout::Direction::Horizontal => {
                    self.direction = layout::Direction::Vertical;
                }
                layout::Direction::Vertical => {
                    self.direction = layout::Direction::Horizontal;
                }
                _ => {}
            }
        }
        else{
            self.layout = layout_type;
            self.direction = layout::Direction::Horizontal;
        }
    }

    pub fn split(&mut self) -> bool {
        if !self.is_empty() {
            return false;
        }

        let mut container = Container::new(self.context);
        let id = self.id;
        self.id = container.id;
        container.id = id;
        self.add(container);
        true
    }

    pub fn focus(&self) {
        libx::set_input_focus(self.context, self.id);
        self.decorate(true);
    }

    pub fn unfocus(&self) {
        self.decorate(false);
    }

    pub fn switch_client(&self) {
        let (x, _) = libx::get_input_focus(self.context);

    }

    pub fn decorate(&self, focused: bool) {
        match self.get_parent() {
            Some(p) => {
                let pid = p.id;
                if focused {
                    layout::decorate_focus(self);
                }
                else {
                    layout::decorate_unfocus(self);
                }
            }
            None => {}
        }

    }

    // fullscreen & normal toggle
    pub fn mode_toggle(&mut self) {
        let context = self.context;
        match self.mode {
            Mode::Normal => {
                self.mode = Mode::Fullscreen;
                println!("fullscreen {} {}", self.id, context.root);
                libx::reparent(context, self.id, context.root, 0, 0);

                let width = libx::display_width(context, context.screen_num);
                let height = libx::display_height(context, context.screen_num);
                libx::resize_window(context, self.id, 0, 0,
                                    width as usize,
                                    height as usize);
            }
            Mode::Fullscreen => {
                self.mode = Mode::Normal;

                let id = self.id;
                match self.get_parent() {
                    Some(p) => {
                        let pid = p.id;
                        libx::reparent(context, id, pid, 0, 0);
                        p.update_layout();
                    }
                    None => {
                        let pid =  context.root;
                        libx::reparent(context, id, pid, 0, 0);
                    }
                };
            }
        }
    }

    // decide which client when click on titlebar
    pub fn query_point(&self, x: i32, y: i32) -> Option<&Container>{
        for client in self.clients.iter() {
            match client.titlebar {
                Some(rec) => {
                    if rec.contain(x, y) {
                        return Some(client);
                    }
                }
                None => {}
            }
        }
        None
    }
}

// #[test]
// fn window_eq() {
//     use std::ptr;
//     let c1 = libx::Context {
//         display: ptr::null_mut()
//     };
//     let w1 = Window::new(c1, 1);
//     let c2 = libx::Context {
//         display: ptr::null_mut()
//     };
//     let w2 = Window::new(c2, 1);
//     assert_eq!(w1, w2);
// }
