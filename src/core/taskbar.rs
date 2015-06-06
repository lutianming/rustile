extern crate x11;

use x11::xlib;
use std::mem;
use std::slice::Iter;
use super::super::libx;
pub struct TaskBar {
    context: libx::Context,
    id: xlib::Window,
    height: u32,
    current: Option<char>,
    workspaces: Vec<char>,
}

impl TaskBar {
    pub fn new(context: libx::Context, height: u32, position: i32) -> TaskBar {
        let pid = context.root;
        let attrs = libx::get_window_attributes(context, pid);
        let y: i32 = if position > 0 {
            attrs.y
        }else{
            attrs.y + attrs.height - height as i32
        };
        let id = libx::create_window(context, pid, attrs.x, y,
                                     attrs.width as u32,
                                     height);

        // attributes
        let mut attrs: xlib::XSetWindowAttributes = unsafe { mem::zeroed() };
        attrs.override_redirect = 1;
        libx::set_window_attributes(context, id, xlib::CWOverrideRedirect, attrs);

        // inputs
        let mask = xlib::ButtonPressMask | xlib::ExposureMask;
        libx::select_input(context, id, mask);
        libx::map_window(context, id);
        TaskBar {
            context: context,
            id: id,
            height: height,
            current: None,
            workspaces: Vec::new()
        }
    }

    pub fn load(&mut self, keys: Vec<char>) {
        self.workspaces = keys;
    }

    pub fn set_current(&mut self, current: char) {
        self.current = Some(current);
    }

    pub fn update(&mut self) {
        let context = self.context;
        let gc = context.gc;
        let display = context.display;
        for (i, v) in self.workspaces.iter().enumerate() {
            let x = (i as u32 * (self.height + 1)) as i32 + 1;
            let y = 0;
            let width = self.height - 2;
            let height = self.height - 2;
            unsafe{
                if self.current.is_some() && v.clone() == self.current.unwrap() {
                    xlib::XSetBackground(display, gc,
                                         context.focus_bg);
                    xlib::XSetForeground(display, gc,
                                         context.focus_fg);
                }
                else {
                    let white = xlib::XWhitePixel(display,
                                                  context.screen_num);
                    xlib::XSetBackground(display, gc,
                                         context.unfocus_bg);
                    xlib::XSetForeground(display, gc,
                                         white);

                }
                xlib::XFillRectangle(display, self.id, gc,
                                     x, y,
                                     width, height);
            }
        }
    }

    pub fn handle(&mut self, e: &xlib::XEvent) {
        let t = e.get_type();
        match t {
            xlib::Expose => {
                let event: xlib::XExposeEvent = From::from(*e);
                if event.window == self.id {
                    self.update();
                }
            }
            _ => {}
        }
    }
}
