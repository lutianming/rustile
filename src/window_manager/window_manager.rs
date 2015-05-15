extern crate libc;
use std::ptr;
use std::mem;
use std::slice::from_raw_parts;
use std::string::String;
use std::collections::HashMap;
use std::boxed::Box;
use x11::xlib;
use x11::keysym;

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


fn get_text_property(display: *mut xlib::Display, window: xlib::Window, atom: xlib::Atom) -> Option<String>{
    unsafe{
        let mut prop: xlib::XTextProperty = mem::zeroed();
        let r = xlib::XGetTextProperty(display, window, &mut prop, atom);
        if r == 0 {
            None
        }else{
            let s = String::from_raw_parts(prop.value, prop.nitems as usize, prop.nitems as usize + 1 ).clone();
            let text = Some(s);
            xlib::XFree(prop.value as *mut libc::c_void);
            text
        }
    }
}

fn get_window_property(display: *mut xlib::Display, window: xlib::Window, atom: xlib::Atom) {
    unsafe {
        let mut actual_type_return: u64 = 0;
        let mut actual_format_return: i32 = 0;
        let mut nitem_return: libc::c_ulong = 0;
        let mut bytes_after_return: libc::c_ulong = 0;
        let mut prop_return: *mut libc::c_uchar = mem::zeroed();

        let r = xlib::XGetWindowProperty(display, window, xlib::XA_WM_NAME,
                                         0, 0xFFFFFFFF, 0, 0,
                                         &mut actual_type_return,
                                         &mut actual_format_return,
                                         &mut nitem_return,
                                         &mut bytes_after_return,
                                         &mut prop_return);

        debug!("result get wp {}", r);
        if r == 0 {

        }else{
            debug!("actual format return {}", actual_format_return);
            if actual_format_return == 0 {

            }
            else {
                // let s = from_raw_parts(prop_return as *const libc::c_ulong, nitems_return as usize).iter()
                //     .map(|&c| c as u64)
                //     .collect();
                let s = String::from_raw_parts(prop_return, nitem_return as usize, nitem_return as usize + 1 );
                println!("{}", s);

            }
        }
    }
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
	unsafe {
	    let display = xlib::XOpenDisplay(ptr::null());
            if display == ptr::null_mut() {
                panic!("can't open display")
            }
	    let screen_num  = xlib::XDefaultScreen(display);
	    let root    = xlib::XRootWindow(display, screen_num);

	    let mut wm = WindowManager {
	        display: display,
                screen_num: screen_num,
	        root:    root,

                config: Config::load(),
                workspaces: Workspaces::new()
            };
            wm.workspaces.create('1');
            wm
	}
    }

    pub fn run(&mut self) {
        let mask = xlib::SubstructureRedirectMask | xlib::SubstructureNotifyMask;
        let keymask = xlib::KeyPressMask | xlib::KeyReleaseMask;
        unsafe{
            xlib::XSetErrorHandler(Some(error_handler));
            xlib::XSelectInput(self.display, self.root,
                               mask | keymask);
            xlib::XSync(self.display, 0);
        }

        loop {
            //handle events here
            let mut e = xlib::XEvent{
                pad: [0; 24],
            };

            unsafe {
                xlib::XNextEvent(self.display, &mut e);
                let t = e.get_type();
                match t {
                    xlib::CreateNotify => {
                        let event: xlib::XCreateWindowEvent = From::from(e);

                        if event.override_redirect != 0 {
                            continue;
                        }

                        // change attributes before display
                        let mut attrs: xlib::XSetWindowAttributes = mem::zeroed();
                        attrs.event_mask = xlib::KeyReleaseMask;
                        let valuemask = xlib::CWEventMask;
                        xlib::XChangeWindowAttributes(self.display, event.window, valuemask, &mut attrs);

                        // add to workspace
                        // let mut w = self.workspaces.get_mut(&'1');
                        // match w {
                        //     Some(workspace) => {
                        //         workspace.add(event.window);
                        //         workspace.config(self);
                        //     }
                        //     None => {}
                        // }

                        // get window property
                        let n = get_text_property(self.display, event.window, xlib::XA_WM_NAME);
                        match n {
                            Some(s) => {debug!("create window {} for {}", event.window, s);}
                            None => {}
                        }
                    }
                    xlib::DestroyNotify => {
                        debug!("destroy notify");
                        let event: xlib::XDestroyWindowEvent = From::from(e);
                    }
                    xlib::MapNotify => {
                        debug!("map notify");
                        let event: xlib::XMapEvent = From::from(e);
                        println!("event {}", event.event);
                        println!("w {}", event.window);
                    }
                    xlib::UnmapNotify => {
                        debug!("unmap notify");
                        let event: xlib::XUnmapEvent = From::from(e);
                        let workspace = self.workspaces.current_workspace();
                        match workspace.contain(event.window) {
                            Some(index) => {
                                workspace.remove(event.window);
                                workspace.config(self.display, self.screen_num);
                            }
                            None => {}
                        }
                    }
                    xlib::MapRequest => {
                        debug!("map request");
                        let event: xlib::XMapRequestEvent = From::from(e);
                        println!("w {}", event.window);
                        xlib::XMapWindow(self.display, event.window);

                        // add app top-level window to workspace
                        let mut attrs: xlib::XWindowAttributes = mem::zeroed();
                        xlib::XGetWindowAttributes(self.display, event.window, &mut attrs);

                        let mut window_return: xlib::Window = 0;
                        let transientfor_hint = xlib::XGetTransientForHint(self.display, event.window, &mut window_return);
                        println!("trans hint {}", transientfor_hint);
                        if attrs.override_redirect == 0 && transientfor_hint == 0 {
                            debug!("top level window");
                            let mut workspace = self.workspaces.current();
                            if workspace.contain(event.window).is_none() {
                                workspace.add(event.window);
                                workspace.config(self.display, self.screen_num);
                            }
                        }

                    }
                    xlib::ClientMessage => {
                        debug!("client message");
                        // let event = cast_event::<xlib::XClientMessageEvent>(&e);
                        let event: xlib::XClientMessageEvent = From::from(e);
                    }
                    xlib::KeyRelease => {
                        debug!("key release");
                        let mut event: xlib::XKeyEvent = From::from(e);

                        if event.state > 0 {
                            let mut sym: xlib::KeySym = 0;
                            let status: *mut xlib::XComposeStatus = ptr::null_mut();
                            xlib::XLookupString(&mut event, ptr::null_mut(), 0, &mut sym, status);

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
                                    continue;
                                }
                            }
                        }
                    }
                    xlib::ConfigureNotify => {
                        debug!("configure notify");
                    }
                    xlib::ConfigureRequest =>{
                        debug!("configure request");
                        // let mut change = xlib::XWindowChanges {
                        //     x: event.x,
                        //     y: event.y,
                        //     width: event.width,
                        //     height: event.height,
                        //     border_width: event.border_width,
                        //     sibling: event.above,
                        //     stack_mode: event.detail
                        // };
                        // debug!("config x: {}, y: {}, width: {}, height: {}",
                        //        change.x, change.y, change.width, change.height);
                        let mut event: xlib::XConfigureRequestEvent = From::from(e);
                        // xlib::XConfigureWindow(event.display, event.window, event.value_mask as u32, &mut change);

                    }
                    _ => {
                        debug!("unhandled event {}", t);
                    }
                }
            }
        }
    }
}
