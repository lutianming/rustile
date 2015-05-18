extern crate libc;

use x11::xlib::Window;
use x11::xlib;

use super::super::libx;

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
}

pub struct TilingLayout {
    direction: Direction,
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
    fn configure(&self, windows: &[Window], context: libx::Context,screen_num: libc::c_int,) {
        let size = windows.len();
        if size == 0 {
            return;
        }
        let screen_height = libx::display_height(context, screen_num);
        let screen_width = libx::display_width(context, screen_num);

        let mask = CWX | CWY | CWHeight | CWWidth;

        let width = screen_width  / size as libc::c_int;
        let height = screen_height / size as libc::c_int;
        println!("screen width {} height {}", width, height);
        for (i, w) in windows.iter().enumerate() {
            let mut change = xlib::XWindowChanges {
                x: 0,
                y: 0,
                width: screen_width,
                height: screen_height,
                border_width: 0,
                sibling: 0,
                stack_mode: 0
            };
            match self.direction {
                Direction::Vertical => {
                    change.y = height * i as libc::c_int;
                    change.height = height;
                }
                Direction::Horizontal => {
                    change.x = width * i as libc::c_int;
                    change.width = width;
                }
                _ => {}
            };

            debug!("config window {} x: {}, y: {}, width: {}, height: {}",
                   *w, change.x, change.y, change.width, change.height);
            libx::configure_window(context, *w, mask, change);
        }
    }
}
