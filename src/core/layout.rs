extern crate libc;

use x11::xlib;

use super::super::libx;
use super::Window;

const CWX: libc::c_uint = 1<<0;
const CWY: libc::c_uint = 1<<1;
const CWWidth: libc::c_uint = 1<<2;
const CWHeight: libc::c_uint = 1<<3;
const CWBorderWidth: libc::c_uint = 1<<4;
const CWSibling: libc::c_uint =	1<<5;
const CWStackMode: libc::c_uint = 1<<6;


pub trait Layout {
    fn configure(&self, windows: &[Window], context: libx::Context,screen_num: libc::c_int);
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
    fn configure(&self, windows: &[Window], context: libx::Context, screen_num: libc::c_int) {
        let size = windows.len();
        if size == 0{
            return;
        }

        let window = windows[0];

        let screen_height = libx::display_height(context, screen_num);
        let screen_width = libx::display_width(context, screen_num);

        let width = screen_width;
        let height = screen_height;

        window.configure(0, 0, width, height);
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
    fn configure(&self, windows: &[Window], context: libx::Context,screen_num: libc::c_int) {
        let size = windows.len();
        if size == 0 {
            return;
        }
        let screen_height = libx::display_height(context, screen_num);
        let screen_width = libx::display_width(context, screen_num);

        let mask = CWX | CWY | CWHeight | CWWidth;

        let width = screen_width  / size as libc::c_int;
        let height = screen_height / size as libc::c_int;
        println!("screen width {} height {}", screen_width, screen_height);
        for (i, window) in windows.iter().enumerate() {
            let mut x = 0;
            let mut y = 0;
            let mut w = screen_width;
            let mut h = screen_height;

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
            println!("{} {} {} {} {}", window.id, x, y,  w, h);
            window.configure(x, y, w, h);
        }
    }
}
