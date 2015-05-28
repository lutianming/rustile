extern crate libc;

use x11::xlib;
use x11::xlib::Window;
use super::super::libx;

use super::config::Config;
use super::Container;
use super::Workspaces;
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
    pub context: libx::Context,
    pub workspaces: Workspaces,
    config: Config
}

impl WindowManager {
    pub fn new() -> WindowManager {
	let res = libx::open_display(None);
        let mut context = match res {
            Some(c) => {
                c
            }
            None => {
                panic!("can't open display")
            }
        };

	context.screen_num  = libx::default_screen(context);
	context.root = libx::root_window(context, context.screen_num);

	let mut wm = WindowManager {
            context: context,
            config: Config::load(),
            workspaces: Workspaces::new(context)
        };
        wm.workspaces.create('1', context.screen_num);
        wm.workspaces.get('1').unwrap().visible = true;
        wm.workspaces.switch_current('1', context);
        wm
    }

    pub fn clean(&mut self) {
        libx::close_display(self.context);
    }

    pub fn handle_create(&mut self, event: &xlib::XCreateWindowEvent) {
        if event.override_redirect != 0 {
            return;
        }

        let attrs = libx::get_window_attributes(self.context, event.window);
        println!("{} attr {} {}", event.window, attrs.width, attrs.height);
        // get window property
        let n = libx::get_text_property(self.context, event.window, xlib::XA_WM_NAME);
        match n {
            Some(s) => {debug!("create window {} for {}", event.window, s);}
            None => {}
        }

        // let atoms = libx::get_wm_protocols(self.context, event.window);
        // for a in atoms.iter() {
        //     println!("atom {}", a);
        //     let name = libx::get_atom_name(self.context, *a);
        //     match name {
        //         Some(n) => println!("name {}", n),
        //         None => println!("name {}", a),
        //     };
        // }
    }

    pub fn handle_destroy(&mut self, event: &xlib::XDestroyWindowEvent) {
        self.workspaces.remove_window(event.window);
        if self.workspaces.get_focus().is_none() {
            self.workspaces.set_focus(self.context.root);
        }
    }

    pub fn handle_map_request(&mut self, event: &xlib::XMapRequestEvent) {

        // add app top-level window to workspace
        // let window = Window::new(self.context, event.window);
        let manage = Workspaces::can_manage(self.context, event.window);

        if manage {
            debug!("top level window");
            let mut container = Container::from_id(self.context, event.window);
            container.map();
            container.focus();
            // change attributes before display
            let mask = 0x420010;
            let mask = xlib::EnterWindowMask | xlib::PropertyChangeMask;
            libx::select_input(self.context, container.id, mask);

            if self.config.titlebar_height > 0 {
                container.titlebar_height = self.config.titlebar_height as usize;
            }
            self.workspaces.add_window(container, None);
        }
        else {
            libx::map_window(self.context, event.window);
        }
    }

    pub fn handle_client_message(&mut self, event: &xlib::XClientMessageEvent) {
        println!("message type {}", event.message_type);
        let s = libx::get_atom_name(self.context, event.message_type);
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
                    if a != 0{
                        let s = libx::get_atom_name(self.context, a as xlib::Atom);
                        match s {
                            Some(v) => println!("{}", v),
                            None => {}
                        }
                    }
                }
            }
            _ => {}
        }
    }

    pub fn handle_button_motion(&mut self, event: &xlib::XMotionEvent) {
        let (window, _) = libx::get_input_focus(self.context);
        if window != event.window {
            self.workspaces.set_focus(event.window);
        }
    }

    pub fn handle_key_release(&mut self, event: &xlib::XKeyEvent) {
        if event.state > 0 {
            let sym = libx::lookup_keysym(*event, 0);
            let b = handler::KeyBind {
                key: sym,
                mask: event.state,
            };
            debug!("key {} {}", event.state, sym);

            match self.config.bindsyms.get_mut(&b) {
                Some(handler) => {
                    handler.handle(&mut self.workspaces, self.context);
                }
                None => {
                    println!("no bind");
                }
            }
        }
    }

    pub fn handle_property(&mut self, event: &xlib::XPropertyEvent) {
        let usertime = libx::get_atom(self.context, "_NET_WM_USER_TIME");
        match event.atom {
            usertime => {
                debug!("_NET_WM_USER_TIME");
                // let window = Window::new(self.context, event.window);
                self.workspaces.set_focus(event.window);
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
        libx::configure_window(self.context, event.window, event.value_mask as u32, change);
    }

    pub fn handle_focus_in(&mut self, event: &xlib::XFocusChangeEvent) {
        self.workspaces.set_focus(event.window);
      }

    pub fn handle_enter(&mut self, event: &xlib::XCrossingEvent){
        if event.focus == 0 {
            self.workspaces.set_focus(event.window);
        }
    }

    pub fn handle(&mut self, e: xlib::XEvent) {
        let t = e.get_type();
        match t {
            xlib::CreateNotify => {
                let event: xlib::XCreateWindowEvent = From::from(e);
                debug!("create notify {}", event.window);
                self.handle_create(&event);
            }
            xlib::DestroyNotify => {
                let event: xlib::XDestroyWindowEvent = From::from(e);
                debug!("destroy notify {}", event.window);
                self.handle_destroy(&event);
            }
            xlib::MapNotify => {
                let event: xlib::XMapEvent = From::from(e);
                debug!("map notify {}", event.window);
            }
            xlib::UnmapNotify => {
                let event: xlib::XUnmapEvent = From::from(e);
                debug!("unmap notify {}", event.window);
            }
            xlib::MapRequest => {
                let event: xlib::XMapRequestEvent = From::from(e);
                debug!("map request {}", event.window);
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
            xlib::MotionNotify => {
                let mut event: xlib::XMotionEvent = From::from(e);
                debug!("button motion {}", event.window);
                self.handle_button_motion(&event);
            }
            xlib::ButtonRelease => {
                let mut event: xlib::XButtonEvent = From::from(e);
                debug!("button release {}", event.window);
            }
            xlib::ButtonPress => {
                let mut event: xlib::XButtonEvent = From::from(e);
                debug!("button press {}", event.window);
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
            xlib::PropertyNotify => {
                let mut event: xlib::XPropertyEvent = From::from(e);
                debug!("property notify {}", event.window);
                self.handle_property(&event);
            }
            _ => {
                debug!("unhandled event {}", t);
            }
        }
    }
    pub fn run(&mut self) {
        loop {
            //handle events here
            let mut e = libx::next_event(self.context);
            self.handle(e);
        }
    }

    pub fn init(&mut self) {
        let mask = 0x1A0034;
        let mask = xlib::SubstructureRedirectMask | xlib::SubstructureNotifyMask;

        unsafe{
            // xlib::XSetErrorHandler(Some(error_handler));
        }
        let left_ptr: u32 = 68;
        libx::define_cursor(self.context, self.context.root, left_ptr);


        libx::ungrab_button(self.context, 0, 0x8000, self.context.root);
        libx::select_input(self.context, self.context.root,
                           mask);

        for bind in self.config.bindsyms.keys() {
            let code = libx::keysym_to_keycode(self.context, bind.key);
            libx::grab_key(self.context, code, bind.mask, self.context.root);
        }
        libx::sync(self.context, 0);
    }
}
