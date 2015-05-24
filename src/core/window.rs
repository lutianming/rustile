extern crate libc;
extern crate x11;

use x11::xlib;
use super::super::libx;

const CWX: libc::c_uint = 1<<0;
const CWY: libc::c_uint = 1<<1;
const CWWidth: libc::c_uint = 1<<2;
const CWHeight: libc::c_uint = 1<<3;
const CWBorderWidth: libc::c_uint = 1<<4;
const CWSibling: libc::c_uint =	1<<5;
const CWStackMode: libc::c_uint = 1<<6;

#[derive(Debug, Copy, Clone)]
pub struct Window {
    pub id: xlib::Window,
    context: libx::Context,
}

impl PartialEq for Window {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Window {
    pub fn new(context: libx::Context, id: xlib::Window) -> Window{
        Window {
            context: context,
            id: id
        }
    }

    pub fn root(context: libx::Context, screen_num: libc::c_int) -> Window {
        let root = libx::root_window(context, screen_num);
        Window {
            context: context,
            id: root,
        }
    }

    pub fn configure(&self, x: i32, y: i32, width: i32, height: i32) {
        let mask = CWX | CWY | CWHeight | CWWidth;

        let mut change = xlib::XWindowChanges {
            x: x,
            y: y,
            width: width,
            height: height,
            border_width: 0,
            sibling: 0,
            stack_mode: 0
        };
        libx::configure_window(self.context, self.id, mask, change);
    }

    pub fn map(&self) {
        libx::map_window(self.context, self.id);
    }

    pub fn unmap(&self) {
        libx::unmap_window(self.context, self.id);
    }

    pub fn focus(&self) {
        libx::set_input_focus(self.context, self.id);
    }
}

#[test]
fn window_eq() {
    use std::ptr;
    let c1 = libx::Context {
        display: ptr::null_mut()
    };
    let w1 = Window::new(c1, 1);
    let c2 = libx::Context {
        display: ptr::null_mut()
    };
    let w2 = Window::new(c2, 1);
    assert_eq!(w1, w2);
}
