use std::ptr;
use std::mem;
use x11::xlib;
use x11::keysym;

use super::config::Config;
use super::handler;

pub struct WindowManager {
    display: *mut xlib::Display,
    root:    xlib::Window,

    config: Config
}

impl WindowManager {
    pub fn new() -> WindowManager {
	unsafe {
	    let display = xlib::XOpenDisplay(ptr::null());
            if display == ptr::null_mut() {
                panic!("can't open display")
            }
	    let screen  = xlib::XDefaultScreenOfDisplay(display);
	    let root    = xlib::XRootWindowOfScreen(screen);

	    WindowManager {
	        display: display,
	        root:    root,

                config: Config::load(),
	    }
	}
    }

    pub fn run(&mut self) {
        let mask = xlib::SubstructureRedirectMask | xlib::SubstructureNotifyMask;
        let keymask = xlib::KeyPressMask | xlib::KeyReleaseMask;
        unsafe{
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
                        debug!("create notify");
                        let event = mem::transmute_copy::<xlib::XEvent, xlib::XCreateWindowEvent>(&e);

                        // chance attributes before display
                        let mut attrs: xlib::XSetWindowAttributes = mem::zeroed();
                        attrs.event_mask = xlib::KeyReleaseMask;
                        let valuemask = xlib::CWEventMask;
                        xlib::XChangeWindowAttributes(self.display, event.window, valuemask, &mut attrs);

                        // display window
                        xlib::XMapWindow(self.display, event.window);
                    }
                    xlib::DestroyNotify => {
                        debug!("destroy notify");
                        let event = mem::transmute_copy::<xlib::XEvent, xlib::XDestroyWindowEvent>(&e);

                    }
                    xlib::KeyRelease => {
                        debug!("key release");
                        let mut event = mem::transmute_copy::<xlib::XEvent, xlib::XKeyEvent>(&e);

                        if event.state > 0 {
                            let mut sym: xlib::KeySym = 0;
                            let status: *mut xlib::XComposeStatus = ptr::null_mut();
                            xlib::XLookupString(&mut event, ptr::null_mut(), 0, &mut sym, status);

                            let b = handler::KeyBind {
                                key: sym,
                                mask: event.state,
                            };
                            debug!("key {} {}", event.state, sym);
                            match self.config.bindsyms.get_mut(&b) {
                                Some(handler) => {
                                    handler.handle();
                                }
                                None => {
                                    println!("no bind")
                                }
                            }
                        }
                    }
                    _ => {

                    }
                }
            }
        }
    }
}
