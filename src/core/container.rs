extern crate libc;
extern crate x11;

use x11::xlib;
use std::ptr;
use super::layout::{ self, Rectangle };
use super::super::libx;

pub const TITLE_HEIGHT: libc::c_int = 20;

pub enum Mode {
    Normal,
    Fullscreen,
}

pub enum Type {
    App,
    Container,
    Workspace,
}

pub struct Container {
    pub id: Option<xlib::Window>,
    pub visible: bool,
    pub titlebar_height: u32,
    parent: *mut Container,
    pub clients: Vec<Container>,
    pub mode: Mode,
    pub category: Type,
    pub titlebar: Option<Rectangle>,
    pub portion: f32,
    pub context: libx::Context,

    pub layout: layout::Type,
    pub direction: layout::Direction,
}

impl PartialEq for Container {
    fn eq(&self, other: &Self) -> bool {
        self.raw_id() == other.raw_id()
    }
}


impl Container {
    pub fn new(context: libx::Context) -> Container{
        let pid = context.root;
        let attrs = libx::get_window_attributes(context, pid);

        let id = libx::create_window(context, pid,
                                     attrs.x, attrs.y,
                                     attrs.width as u32,
                                     attrs.height as u32);
        libx::select_input(context, id, xlib::SubstructureNotifyMask | xlib::SubstructureRedirectMask | xlib::ButtonPressMask);
        Container {
            context: context,
            clients: Vec::new(),
            visible: false,
            id: Some(id),
            mode: Mode::Normal,
            category: Type::Container,
            parent: ptr::null_mut(),
            titlebar: None,
            titlebar_height: 0,
            portion: 1.0,

            layout: layout::Type::Tiling,
            direction: layout::Direction::Horizontal,
        }
    }

    pub fn from_id(context: libx::Context, id: xlib::Window) -> Container {
        Container {
            context: context,
            clients: Vec::new(),
            visible: false,
            id: Some(id),
            mode: Mode::Normal,
            category: Type::App,
            parent: ptr::null_mut(),
            titlebar: None,
            titlebar_height: 0,
            portion: 1.0,

            layout: layout::Type::Tiling,
            direction: layout::Direction::Horizontal,
        }
    }

    pub fn print_tree(&self, indent: i32) {
        for i in 0..indent {
            print!(" ");
        }
        let attrs = libx::get_window_attributes(self.context, self.raw_id());
        let pid = match self.get_parent() {
            Some(p) => { p.raw_id() }
            None => { 0}
        };
        println!("C {} pid:{} x:{} y:{} w:{} h:{}", self.raw_id(), pid, attrs.x, attrs.y, attrs.width, attrs.height);
        for client in self.clients.iter() {
            client.print_tree(indent+4);
        }
    }

    pub fn raw_id(&self) -> xlib::Window {
        self.id.unwrap()
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

    pub fn be_parent(&mut self, client: &mut Container) {
        let need_reprent = match client.get_parent() {
            Some(p) => {
                if p.raw_id() == self.raw_id() { false } else { true }
            }
            None => {
                true
            }
        };
        if need_reprent {
            libx::reparent(self.context, client.raw_id(), self.raw_id(), 0, 0);
            client.parent = self;
        }
    }

    pub fn add(&mut self, mut client: Container) {
        self.be_parent(&mut client);
        let portion = 1.0 / (self.size() as f32 + 1.0);
        for client in self.clients.iter_mut() {
            client.portion = client.portion * (1.0-portion);
        }
        client.portion = portion;
        self.clients.push(client);
    }

    pub fn insert(&mut self, index: usize,  mut client: Container) {
        self.be_parent(&mut client);
        let portion = 1.0 / (self.size() as f32 + 1.0);
        client.portion = portion;
        for client in self.clients.iter_mut() {
            client.portion = client.portion * (1.0-portion);
        }
        self.clients.insert(index, client);
    }

    pub fn remove(&mut self, id: xlib::Window) -> Option<Container>{
        let res = self.contain(id);
        match res {
            Some(index) => {
                self.remove_by_index(index)
            }
            None => {
                None
            }
        }
    }

    fn remove_by_index(&mut self, index: usize) -> Option<Container>{
        if index >= self.size() {
            None
        }
        else{
            let r = self.clients.remove(index);
            let portion = 1.0 - r.portion;
            for client in self.clients.iter_mut() {
                client.portion = client.portion / portion;
            }
            Some(r)
        }
    }

    /// remove App container and destroy all its parent containers that are empty
    pub fn tree_remove(&mut self, id: xlib::Window) -> Option<Container> {
        println!("try remove {} from {}", id, self.raw_id());
        let res = self.remove(id);
        match res {
            Some(c) => {
                Some(c)
            }
            None => {
                let mut r: Option<Container> = None;
                let mut index: i32 = -1;
                for (i, c) in self.clients.iter_mut().enumerate() {
                    r = c.tree_remove(id);
                    if r.is_some(){
                        index = i as i32;
                        break
                    }
                }
                if index >= 0 {
                    let i = index as usize;
                    let size = self.get_child(i).unwrap().size();
                    if size == 0 {
                        let container = self.remove_by_index(i);
                        container.unwrap().destroy();
                    }
                }
                r
            }
        }
    }
    pub fn get_child(&mut self, index: usize) -> Option<&mut Container> {
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
        if self.raw_id() == id {
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
        self.clients.iter().position(|x| (*x).raw_id() == id)
    }

    pub fn configure(&mut self, x: i32, y: i32, width: u32, height: u32) {
        libx::resize_window(self.context, self.raw_id(), x, y, width, height);
        // layout for children clients
        self.update_layout();
    }

    pub fn resize_children(&mut self, index: usize, neighbor: usize, step: f32) {
        let size = self.size();
        if index >= 0 && index < size && neighbor >= 0 && neighbor < size {
            {
                let mut a = self.clients.get_mut(index).unwrap();
                a.portion = a.portion + step;
            }
            {
                let mut b = self.clients.get_mut(neighbor).unwrap();
                b.portion = b.portion - step;
            }
        }
    }

    pub fn update_layout(&mut self) {
        layout::update_layout(self);
    }

    pub fn map(&self) {
        // self.visible = true;
        libx::map_window(self.context, self.raw_id());
        for client in self.clients.iter() {
            client.map();
        }
    }

    pub fn unmap(&self) {
        // self.visible = false;
        for client in self.clients.iter() {
            client.unmap();
        }
        libx::unmap_window(self.context, self.raw_id());
    }

    pub fn destroy(&self) -> bool {
        // can distroy only if it has no clients
        if self.clients.is_empty() {
            unsafe{
                xlib::XDestroyWindow(self.context.display, self.raw_id());
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
                self.get_child(next)
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
                self.get_child(last)
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
        if !self.is_empty() || self.raw_id() == self.context.root {
            return false;
        }


        let mut container = Container::new(self.context);
        // add to parent
        match self.get_parent() {
            Some(p) => {
                let attrs = self.rec();

                libx::reparent(self.context, container.raw_id(), p.raw_id(),
                               0, 0);
                // configure to the same size and position like old one
                container.configure(attrs.x, attrs.y,
                                    attrs.width,
                                    attrs.height);
            }
            None => {}
        }

        // swap the property
        let id = self.id;
        self.id = container.id;
        container.id = id;
        container.titlebar_height = self.titlebar_height;
        self.add(container);
        self.map();
        true
    }

    pub fn focus(&self) {
        let id = self.raw_id();
        libx::set_input_focus(self.context, id);
        libx::raise_window(self.context, id);
        self.decorate(true);
    }

    pub fn unfocus(&self) {
        // libx::lower_window(self.context, self.id);
        self.decorate(false);
    }

    pub fn switch_client(&self) {
        let (x, _) = libx::get_input_focus(self.context);

    }

    pub fn decorate(&self, focused: bool) {
        match self.get_parent() {
            Some(p) => {
                layout::decorate(self, focused);
            }
            None => {}
        }

    }

    // fullscreen & normal toggle
    pub fn mode_toggle(&mut self) {
        let context = self.context;
        let id = self.raw_id();
        match self.mode {
            Mode::Normal => {
                self.mode = Mode::Fullscreen;
                println!("fullscreen {} {}", id, context.root);
                libx::reparent(context, id, context.root, 0, 0);

                let width = libx::display_width(context, context.screen_num);
                let height = libx::display_height(context, context.screen_num);
                libx::resize_window(context, id, 0, 0,
                                    width,
                                    height);
            }
            Mode::Fullscreen => {
                self.mode = Mode::Normal;

                match self.get_parent() {
                    Some(p) => {
                        let pid = p.raw_id();
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

    pub fn rec(&self) -> layout::Rectangle {
        let attrs = libx::get_window_attributes(self.context, self.raw_id());
        layout::Rectangle {
            x: attrs.x,
            y: attrs.y,
            width: attrs.width as u32,
            height: attrs.height as u32
        }
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
