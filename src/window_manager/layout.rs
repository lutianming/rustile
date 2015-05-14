extern crate libc;

use x11::xlib::Window;
use x11::xlib;

use super::WindowManager;

const CWX: libc::c_uint = 1<<0;
const CWY: libc::c_uint = 1<<1;
const CWWidth: libc::c_uint = 1<<2;
const CWHeight: libc::c_uint = 1<<3;
const CWBorderWidth: libc::c_uint = 1<<4;
const CWSibling: libc::c_uint =	1<<5;
const CWStackMode: libc::c_uint = 1<<6;


pub trait Layout {
    fn configure(&self, windows: &[Window], wm: &WindowManager);
    fn toggle(&mut self) {}
    fn get_type(&self) -> Type;
}

pub enum Direction {
    Vertical,
    Horizontal,
}

#[derive(PartialEq)]
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
        }
    }
    /// once we add or remove a window, we need to reconfig
    fn configure(&self, windows: &[Window], wm: &WindowManager) {
        let size = windows.len();
        if size == 0 {
            return;
        }

        unsafe {
            let screen_height = xlib::XDisplayHeight(wm.display, wm.screen_num);
            let screen_width = xlib::XDisplayWidth(wm.display, wm.screen_num);
            let mask = CWX | CWY | CWHeight | CWWidth;

            let mut x = 0;
            let width = screen_width / size as i32;
            for w in windows {
                let mut change = xlib::XWindowChanges {
                    x: x,
                    y: 0,
                    width: width,
                    height: screen_height,
                    border_width: 0,
                    sibling: 0,
                    stack_mode: 0
                };
                x = x + width;
                debug!("config x: {}, y: {}, width: {}, height: {}",
                       change.x, change.y, change.width, change.height);
                xlib::XConfigureWindow(wm.display, *w, mask, &mut change);
            }
        }
    }
}
