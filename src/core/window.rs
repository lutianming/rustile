extern crate libc;
extern crate x11;

use x11::xlib;
use std::ffi;
use std::ptr;
use std::mem;
use super::super::libx;

const CWX: libc::c_uint = 1<<0;
const CWY: libc::c_uint = 1<<1;
const CWWidth: libc::c_uint = 1<<2;
const CWHeight: libc::c_uint = 1<<3;
const CWBorderWidth: libc::c_uint = 1<<4;
const CWSibling: libc::c_uint =	1<<5;
const CWStackMode: libc::c_uint = 1<<6;

const TITLE_HEIGHT: libc::c_int = 20;

#[derive(Debug, Copy, Clone)]
pub struct Window {
    pub id: xlib::Window,
    pub client: Option<xlib::Window>,
    pub context: libx::Context,
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
            client: None,
            id: id
        }
    }

    pub fn root(context: libx::Context, screen_num: libc::c_int) -> Window {
        let root = libx::root_window(context, screen_num);
        Window {
            context: context,
            client: None,
            id: root,
        }
    }

    pub fn decorate(context: libx::Context, id: xlib::Window) -> Window {
        let res = libx::query_tree(context, id);

        match res {
            Some((root, _, _)) => {
                let attrs = libx::get_window_attributes(context, id);
                let parent = libx::create_window(context, root, attrs.x, attrs.y, attrs.width as libc::c_uint, attrs.height as libc::c_uint);
                libx::select_input(context, parent, xlib::SubstructureNotifyMask | xlib::SubstructureRedirectMask);
                libx::reparent(context, id, parent, 0, TITLE_HEIGHT);

                Window {
                    context: context,
                    client: Some(id),
                    id: parent
                }
            }
            None => {
                Window {
                    context: context,
                    client: None,
                    id: id
                }
            }
        }
    }

    pub fn get_top(&self) -> Option<Window>{
        let top = libx::get_top_window(self.context, self.id);
        match top {
            Some(id) => {
                let w = Window {
                    context: self.context,
                    client: if self.id==id { None } else {Some(self.id)},
                    id: id
                };
                Some(w)
            }
            None => { None }
        }
    }

    pub fn is_top(&self) -> bool {
        let top = self.get_top();
        match top {
            Some(w) => {
                w.id == self.id
            }
            Noen => false
        }
    }

    pub fn can_manage(&self) -> bool {
        let attrs = libx::get_window_attributes(self.context, self.id);
        let transientfor_hint = libx::get_transient_for_hint(self.context, self.id);
        attrs.override_redirect == 0 && transientfor_hint == 0
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

        match self.client {
            Some(c) => {
                let w = Window::new(self.context, c);
                println!("{} {} {}", c, width, height-TITLE_HEIGHT);
                w.configure(0, TITLE_HEIGHT, width, height-TITLE_HEIGHT);
            }
            None => {}
        }
    }

    pub fn map(&self) {
        libx::map_window(self.context, self.id);
        unsafe{
            if self.client.is_some(){
                xlib::XMapSubwindows(self.context.display, self.id);
            }

        }
    }

    pub fn unmap(&self) {
        unsafe {
            if self.client.is_some(){
                xlib::XUnmapSubwindows(self.context.display, self.id);
            }
        }
        libx::unmap_window(self.context, self.id);
    }

    pub fn destroy(&self) {
        unsafe{
            xlib::XDestroyWindow(self.context.display, self.id);
        }
    }

    pub fn focus(&self) {
        println!("focus {:?}", self);
        match self.client {
            Some(id) => {
                libx::set_input_focus(self.context, id);
                self.draw_titlebar(true);
            }
            None => {
                libx::set_input_focus(self.context, self.id);
            }
        }
    }

    pub fn unfocus(&self) {
        println!("unfocus {:?}", self);
        if self.client.is_some(){
            self.draw_titlebar(false);
        }
    }

    fn draw_titlebar(&self, focused: bool) {
        println!("draw titlebar {}", self.id);
        let attrs = libx::get_window_attributes(self.context, self.id);
        let screen = libx::default_screen(self.context);
        let root = libx::root_window(self.context, screen);
        let mut values: xlib::XGCValues = unsafe{ mem::zeroed() };
        // let gc = libx::create_gc(self.context, self.id, 0, values);
        let gc = libx::default_gc(self.context, screen);


        unsafe {
            let black = xlib::XBlackPixel(self.context.display, screen);
            let white = xlib::XWhitePixel(self.context.display, screen);

            xlib::XSetLineAttributes(self.context.display, gc, 5, 0, 0, 0);

            let cmap = xlib::XDefaultColormap(self.context.display, screen);
            let mut color: xlib::XColor = mem::zeroed();
            let name = ffi::CString::new("blue").unwrap().as_ptr();
            let r = xlib::XParseColor(self.context.display, cmap, name, &mut color);
            xlib::XAllocColor(self.context.display, cmap, &mut color);

            // let mut set_attrs: xlib::XSetWindowAttributes = mem::zeroed();
            // if focused {
            //     set_attrs.background_pixel = color.pixel;
            //     set_attrs.border_pixel = color.pixel;
            // }
            // else {
            //     set_attrs.background_pixel = black;
            //     set_attrs.border_pixel = black;
            // }
            // libx::set_window_attributes(self.context, self.id, xlib::CWBackPixel | xlib::CWBorderPixel, set_attrs);


            // try draw rectangle
            if focused {
                xlib::XSetBackground(self.context.display, gc,
                                     black);
                xlib::XSetForeground(self.context.display, gc,
                                     color.pixel);
            }
            else {
                xlib::XSetBackground(self.context.display, gc,
                                     black);
                xlib::XSetForeground(self.context.display, gc,
                                     black);
            }
            let r = xlib::XFillRectangle(self.context.display,
                                         self.id, gc,
                                         1, 1,
                                         attrs.width as libc::c_uint,
                                         TITLE_HEIGHT as libc::c_uint);

            match self.client {
                Some(id) => {
                    let p = libx::get_text_property(self.context, id, xlib::XA_WM_NAME);
                    match p {
                        Some(s) => {
                            let size = s.len();
                            xlib::XSetForeground(self.context.display, gc, white);
                            println!("draw string {} {}", id, s);
                            let title = ffi::CString::new("TITLE").unwrap();

                            xlib::XDrawString(self.context.display,
                                              self.id,
                                              gc,
                                              0, 0,
                                              title.as_ptr(),
                                              5);

                        }
                        None => {}
                    }
                }
                None => {}
            }
        }
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
