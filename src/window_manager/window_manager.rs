extern crate libc;

use std::ptr;
use std::mem;
use std::ffi;
use std::str;
use std::slice::from_raw_parts;
use std::string::String;
use std::collections::HashMap;
use std::boxed::Box;
use x11::xlib;
use x11::keysym;

use super::super::libx;

use super::config::Config;
use super::workspace::{ Workspace, Workspaces };
use super::handler;

unsafe extern fn error_handler(display: *mut xlib::Display, event: *mut xlib::XErrorEvent) -> libc::c_int {
    // match event.error_code {
    //     xlib::BadAtom => {
    //         println!("bad atom");
    //     }
    //     _ => {}
    // }
    1
}

pub struct WindowManager {
    pub display: *mut xlib::Display,
    pub screen_num: libc::c_int,
    pub root:    xlib::Window,

    pub workspaces: Workspaces,
    config: Config
}

impl WindowManager {
    pub fn new() -> WindowManager {
	let display = libx::open_display(None);
        let display = match display {
            Some(p) => {
                p
            }
            None => {
                panic!("can't open display")
            }
        };

	let screen_num  = libx::default_screen(display);
	let root = libx::root_window(display, screen_num);
        println!("root window {}", root);
	let mut wm = WindowManager {
	    display: display,
            screen_num: screen_num,
	    root: root,

            config: Config::load(),
            workspaces: Workspaces::new()
        };
        wm.workspaces.create('1');
        wm
    }

    pub fn clean(&mut self) {
        libx::close_display(self.display);
    }

    pub fn handle_create(&mut self, event: &xlib::XCreateWindowEvent) {
        if event.override_redirect != 0 {
            return;
        }

        let attrs = libx::get_window_attributes(self.display, event.window);
        println!("before");
        println!("attr all event {}", attrs.all_event_masks);
        println!("attr you event {}", attrs.your_event_mask);



        // get window property
        let n = libx::get_text_property(self.display, event.window, xlib::XA_WM_NAME);
        match n {
            Some(s) => {debug!("create window {} for {}", event.window, s);}
            None => {}
        }

        // let atoms = libx::get_wm_protocols(self.display, event.window);
        // for a in atoms.iter() {
        //     println!("atom {}", a);
        //     let name = libx::get_atom_name(self.display, *a);
        //     match name {
        //         Some(n) => println!("name {}", n),
        //         None => println!("name {}", a),
        //     };
        // }
    }

    pub fn handle_destroy(&mut self, event: &xlib::XDestroyWindowEvent) {
        let workspace = self.workspaces.current();
        match workspace.contain(event.window) {
            Some(index) => {
                workspace.remove(event.window);
                workspace.config(self.display, self.screen_num);
            }
            None => {}
        }
    }

    pub fn handle_map_request(&mut self, event: &xlib::XMapRequestEvent) {
        unsafe{
            xlib::XMapWindow(self.display, event.window);

            // add app top-level window to workspace

            let attrs = libx::get_window_attributes(self.display, event.window);
            let transientfor_hint = libx::get_transient_for_hint(self.display, event.window);

            println!("trans hint {}", transientfor_hint);
            if attrs.override_redirect == 0 && transientfor_hint == 0 {
                debug!("top level window");
                let mut workspace = self.workspaces.current();
                if workspace.contain(event.window).is_none() {
                    workspace.add(event.window);
                    workspace.config(self.display, self.screen_num);
                }

                // change attributes before display
                unsafe{
                    let mut attrs: xlib::XSetWindowAttributes = mem::zeroed();
                    attrs.event_mask = xlib::KeyReleaseMask | xlib::FocusChangeMask | xlib::EnterWindowMask;
                    let valuemask = xlib::CWEventMask;
                    libx::change_window_attributes(self.display, event.window, valuemask, &mut attrs);
                }
            }
        }
    }

    pub fn handle_client_message(&mut self, event: &xlib::XClientMessageEvent) {
        println!("message type {}", event.message_type);
        let s = libx::get_atom_name(self.display, event.message_type);
        match s {
            Some(v) => println!("{}", &v),
            None => {}
        }

        match event.format {
            8 => {
                for i in 0..20 {
                    println!("{}", event.data.get_byte(i));
                }
            }
            16 => {
                for i in 0..10 {
                    println!("{}", event.data.get_short(i))
                }
            }
            32 => {
                for i in 0..5 {
                    let a = event.data.get_long(i);
                    println!("data {}", a);
                    // let s = libx::get_atom_name(self.display, a as xlib::Atom);
                    // match s {
                    //     Some(v) => println!("{}", v),
                    //     None => {}
                    // }
                }
            }
            _ => {}
        }
    }

    pub fn handle_key_release(&mut self, event: &mut xlib::XKeyEvent) {
        if event.state > 0 {
            let mut sym: xlib::KeySym = 0;
            let status: *mut xlib::XComposeStatus = ptr::null_mut();
            unsafe{
                xlib::XLookupString(event, ptr::null_mut(), 0, &mut sym, status);
            }
            let b = handler::KeyBind {
                key: sym,
                mask: event.state,
            };
            debug!("key {} {}", event.state, sym);
            // let mut h: Box<handler::Handler>;
            match self.config.bindsyms.get_mut(&b) {
                Some(handler) => {
                    handler.handle(&mut self.workspaces, self.display, self.screen_num);
                }
                None => {
                    println!("no bind");
                }
            }
        }
    }

    pub fn handle_configure_request(&mut self, event: &xlib::XConfigureRequestEvent) {
        let mut change = xlib::XWindowChanges {
            x: event.x,
            y: event.y,
            width: event.width,
            height: event.height,
            border_width: event.border_width,
            sibling: event.above,
            stack_mode: event.detail
        };
        debug!("config x: {}, y: {}, width: {}, height: {}",
               change.x, change.y, change.width, change.height);
        // xlib::XConfigureWindow(event.display, event.window, event.value_mask as u32, &mut change);

    }

    pub fn handle_focus_in(&mut self, event: &xlib::XFocusChangeEvent) {
        libx::set_input_focus(self.display, event.window, 0, 0);
        let (window, _) = libx::get_input_focus(self.display);
        println!("window {}", window);
      }

    pub fn handle_enter(&mut self, event: &xlib::XCrossingEvent){
        println!("set focus");
        libx::set_input_focus(self.display, event.window, 0, 0);

        // if event.focus == 0 {
        // }
    }

    pub fn handle(&mut self, e: xlib::XEvent) {
        let t = e.get_type();
        match t {
            xlib::CreateNotify => {
                let event: xlib::XCreateWindowEvent = From::from(e);
                self.handle_create(&event);
            }
            xlib::DestroyNotify => {
                debug!("destroy notify");
                let event: xlib::XDestroyWindowEvent = From::from(e);
                self.handle_destroy(&event);
            }
            xlib::MapNotify => {
                debug!("map notify");
                let event: xlib::XMapEvent = From::from(e);

            }
            xlib::UnmapNotify => {
                debug!("unmap notify");
                let event: xlib::XUnmapEvent = From::from(e);
            }
            xlib::MapRequest => {
                debug!("map request");
                let event: xlib::XMapRequestEvent = From::from(e);
                self.handle_map_request(&event);
            }
            xlib::ClientMessage => {
                debug!("client message");
                // let event = cast_event::<xlib::XClientMessageEvent>(&e);
                let event: xlib::XClientMessageEvent = From::from(e);
                self.handle_client_message(&event);
            }
            xlib::KeyRelease => {
                debug!("key release");
                let mut event: xlib::XKeyEvent = From::from(e);
                self.handle_key_release(&mut event);
            }
            xlib::ConfigureNotify => {
                debug!("configure notify");
            }
            xlib::ConfigureRequest =>{
                let mut event: xlib::XConfigureRequestEvent = From::from(e);
                debug!("configure request {}", event.window);
                self.handle_configure_request(&event);
            }
            xlib::FocusIn => {
                let mut event: xlib::XFocusChangeEvent = From::from(e);
                debug!("focus in {}", event.window);
                self.handle_focus_in(&event);

            }
            xlib::FocusOut => {
                let mut event: xlib::XFocusChangeEvent = From::from(e);
                debug!("focus out {}", event.window);
            }

            xlib::EnterNotify => {
                let mut event: xlib::XCrossingEvent = From::from(e);
                debug!("enter window {}", event.window);
                self.handle_enter(&event);
            }
            _ => {
                debug!("unhandled event {}", t);
            }
        }
    }
    pub fn run(&mut self) {
        loop {
            //handle events here
            unsafe {
                let mut e = libx::next_event(self.display);
                self.handle(e);
            }
        }
    }

    pub fn init(&mut self) {
        let mask = xlib::SubstructureRedirectMask | xlib::SubstructureNotifyMask;
        let keymask = xlib::KeyPressMask | xlib::KeyReleaseMask;
        unsafe{
            // xlib::XSetErrorHandler(Some(error_handler));
            xlib::XSelectInput(self.display, self.root,
                               mask | keymask);
            xlib::XSync(self.display, 0);
        }
    }
}
