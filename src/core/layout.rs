extern crate libc;

use x11::xlib;

use super::super::libx;
use super::container::{self, Container};


const CWX: libc::c_uint = 1<<0;
const CWY: libc::c_uint = 1<<1;
const CWWidth: libc::c_uint = 1<<2;
const CWHeight: libc::c_uint = 1<<3;
const CWBorderWidth: libc::c_uint = 1<<4;
const CWSibling: libc::c_uint =	1<<5;
const CWStackMode: libc::c_uint = 1<<6;


pub trait Layout {
    fn configure(&self, container: &Container);
    fn toggle(&mut self) {}
    fn get_type(&self) -> Type;
}

pub enum Direction {
    Vertical,
    Horizontal,
    Up,
    Down,
    Left,
    Right,
}


#[derive(PartialEq, Clone)]
pub enum Type {
    Tiling,
    Tab,
}

pub struct TilingLayout {
    direction: Direction,
}

pub struct TabLayout;

impl TabLayout {
    pub fn new() -> TabLayout{
        TabLayout
    }
}
impl Layout for TabLayout {
    fn get_type(&self) -> Type { Type::Tab }
    fn configure(&self, container: &Container) {
        let size = container.clients.len();
        if size == 0{
            return;
        }

        let attrs = libx::get_window_attributes(container.context, container.id);

        for client in container.clients.iter() {
            client.configure(0, 0, attrs.width as usize, attrs.height as usize);
        }
    }
}

impl TilingLayout {
    pub fn new(d: Direction) -> TilingLayout{
        TilingLayout {
            direction: d,
        }
    }
}

impl Layout for TilingLayout {
    fn get_type(&self) -> Type { Type::Tiling }
    fn toggle(&mut self) {
        match self.direction {
            Direction::Vertical => self.direction = Direction::Horizontal,
            Direction::Horizontal => self.direction = Direction::Vertical,
            _ => {}
        }
    }
    /// once we add or remove a window, we need to reconfig
    fn configure(&self, container: &Container) {
        let size = container.clients.len();
        if size == 0 {
            return;
        }

        let attrs = libx::get_window_attributes(container.context, container.id);

        let width = attrs.width  / size as libc::c_int;
        let height = attrs.height / size as libc::c_int;

        for (i, client) in container.clients.iter().enumerate() {
            let mut x = 0;
            let mut y = 0;
            let mut w = attrs.width;
            let mut h = attrs.height;

            match self.direction {
                Direction::Vertical => {
                    y = height * i as libc::c_int;
                    h = height;
                }
                Direction::Horizontal => {
                    x = width * i as libc::c_int;
                    w = width;
                }
                _ => {}
            };
            client.configure(x, y+container.titlebar_height as i32, w as usize, h as usize);
        }
    }
}
