extern crate libc;
extern crate x11;

use std::ptr;
use x11::xlib;
use x11::xlib::Window;
use super::super::libx;

use super::config::Config;
use super::container::{self, Container};
use super::layout;
use super::Workspaces;
use super::TaskBar;
use super::handler;

unsafe extern fn error_handler(display: *mut xlib::Display, event: *mut xlib::XErrorEvent) -> libc::c_int {
    // match event.error_code {
    //     xlib::BadAtom => {
    //         println!("bad atom");
    //     }
    //     _ => {}
    // }
    // x11::xmu::XmuSimpleErrorHandler(display, event);
    1
}

extern "C" {
    fn setlocale(category: i32, locale: *const i8) -> *mut i8;
}

fn load_resource(mut context: &mut libx::Context) {
    use std::mem;
    use std::ffi;
    let screen = context.screen_num;
    let root = context.root;
    let display = context.display;
    let mut values: xlib::XGCValues = unsafe{ mem::zeroed() };
    // let gc = libx::create_gc(self.context, self.id, 0, values);
    let gc = libx::default_gc(*context, screen);

    unsafe {
        let black = xlib::XBlackPixel(display, screen);
        let white = xlib::XWhitePixel(display, screen);

        xlib::XSetLineAttributes(display, gc, 5, 0, 0, 0);

        let cmap = xlib::XDefaultColormap(display, screen);
        let mut color: xlib::XColor = mem::zeroed();
        let name = ffi::CString::new("blue").unwrap().as_ptr();
        let r = xlib::XParseColor(display, cmap, name, &mut color);
        xlib::XAllocColor(display, cmap, &mut color);

        context.gc = gc;
        context.focus_bg = black;
        context.focus_fg = color.pixel;
        context.unfocus_bg = black;
        context.unfocus_fg = black;
        context.font_color = white;

        let s = ffi::CString::new("").unwrap().as_ptr();
        let p = setlocale(6, s);
        let res = xlib::XSupportsLocale();
        let res = xlib::XSetLocaleModifiers(s);

        let mut missing_charsets = ptr::null_mut();
        let mut num_missing_charsets: i32 = 0;
        let mut default_string = ptr::null_mut();
        // let name = "-misc-fixed-*-*-*-*-*-130-75-75-*-*-*-*";
        let name = "*";
        let fontbase = ffi::CString::new(name).unwrap().as_ptr();
        let fontset = xlib::XCreateFontSet(display,
                                           fontbase,
                                           &mut missing_charsets,
                                           &mut num_missing_charsets,
                                           &mut default_string);
        if num_missing_charsets > 0 {
            xlib::XFreeStringList(missing_charsets);
        }
        context.fontset = fontset;
    }
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
        load_resource(&mut context);

	let mut wm = WindowManager {
            context: context,
            config: Config::new(),
            workspaces: Workspaces::new(context)
        };
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
            let id = self.workspaces.current().raw_id();
            self.workspaces.set_focus(id);
        }
    }

    pub fn handle_expose(&mut self, event: &xlib::XExposeEvent) {
        let res = self.workspaces.get_container(event.window);
        match res {
            Some((_, c)) => {
                for client in c.clients.iter() {
                    client.decorate(client.is_focused());
                }
            }
            None => {}
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
            // container.focus();
            // change attributes before display
            let mask = 0x420010;
            let mask = xlib::EnterWindowMask | xlib::PropertyChangeMask;
            libx::select_input(self.context, container.raw_id(), mask);

            if self.config.titlebar_height > 0 {
                container.titlebar_height = self.config.titlebar_height;
            }
            self.workspaces.insert_window(container);
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

    }

    pub fn handle_button_press(&mut self, event: &xlib::XButtonEvent) {

        let id = match self.workspaces.get_container(event.window) {
            Some((_,c)) => {
                let client = c.query_point(event.x, event.y);
                match client {
                    Some(c) => {
                        c.raw_id()
                    }
                    None => { c.raw_id() }
                }
            }
            None => { return }
        };
        self.workspaces.set_focus(id);

        // test if press on boarder
        match self.workspaces.get_container(event.window) {
            Some((_,c)) => {
                let res = c.query_border(event.x, event.y);
                match res {
                    Some(i) => {
                        match c.mode {
                            container::Mode::Normal => {
                                c.mode = container::Mode::Resize(i, event.x, event.y)
                            }
                            container::Mode::Resize(index, x, y) => {

                            }
                            _ => {}
                        }
                    }
                    None => {}
                }
            }
            None => {}
        }
    }

    pub fn handle_button_release(&mut self, event: &xlib::XButtonEvent) {
        match self.workspaces.get_container(event.window) {
            Some((_, c)) => {
                match c.mode {
                    container::Mode::Resize(index, x, y) => {
                        let dx = event.x - x;
                        let dy = event.y - y;
                        let rec = c.rec();
                        let step = match c.direction {
                            layout::Direction::Vertical => {
                                dy as f32 / rec.height as f32
                            }
                            layout::Direction::Horizontal => {
                                dx as f32 / rec.width as f32
                            }
                            _ => { 0.05 }
                        };

                        c.resize_children(index-1, index, step);
                        c.update_layout();
                        c.mode = container::Mode::Normal;
                    }
                    _ => {}
                }
            }
            None => {}
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
            xlib::Expose => {
                let event: xlib::XExposeEvent = From::from(e);
                debug!("expose {}", event.window);
                self.handle_expose(&event);
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
                self.handle_button_motion(&event);
            }
            xlib::ButtonRelease => {
                let mut event: xlib::XButtonEvent = From::from(e);
                self.handle_button_release(&event);
            }
            xlib::ButtonPress => {
                let mut event: xlib::XButtonEvent = From::from(e);
                self.handle_button_press(&event);
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
            match self.workspaces.taskbar.as_mut() {
                Some(b) => {
                    b.handle(&e);
                }
                None => {}
            }
            self.handle(e);
        }
    }

    pub fn init(&mut self) {
        // init connection setting
        let mask = 0x1A0034;
        let mask = xlib::SubstructureRedirectMask | xlib::SubstructureNotifyMask | xlib::ButtonPressMask;

        unsafe{
            xlib::XSetErrorHandler(Some(error_handler));
        }
        let left_ptr: u32 = 68;
        libx::define_cursor(self.context, self.context.root, left_ptr);

        libx::select_input(self.context, self.context.root,
                           mask);

        self.init_workspaces();

        // load config file, run exec in config
        self.config.load();

        for bind in self.config.bindsyms.keys() {
            let code = libx::keysym_to_keycode(self.context, bind.key);
            libx::grab_key(self.context, code, bind.mask, self.context.root);
        }
        libx::sync(self.context, 0);
    }

    fn init_workspaces(&mut self) {
        let attrs = libx::get_window_attributes(self.context, self.context.root);
        let taskbar_height: u32 = 20;
        self.workspaces.rec = Some(layout::Rectangle {
            x: attrs.x,
            y: attrs.y + taskbar_height as i32,
            width: attrs.width as u32,
            height: attrs.height as u32 - taskbar_height,
        });
        self.workspaces.taskbar = Some(TaskBar::new(self.context, 20, 1));
        self.workspaces.switch_workspace('1', self.context);
    }
}
