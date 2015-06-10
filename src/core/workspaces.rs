extern crate x11;
extern crate libc;

use std::collections::HashMap;
use x11::xlib::{Window};
use super::container::{ self, Container };
use super::layout;
use super::TaskBar;
use super::super::libx::{ self, Context };

pub struct Workspaces {
    current: char,
    pub context: Context,
    pub mode: container::Mode,
    pub rec: Option<layout::Rectangle>,
    pub taskbar: Option<TaskBar>,
    pub spaces: HashMap<char, Container>,
}

impl Workspaces {
    pub fn new(context: Context) -> Workspaces {
        Workspaces {
            current: '0',
            mode: container::Mode::Normal,
            context: context,
            spaces: HashMap::new(),
            taskbar: None,
            rec: None,
        }
    }

    pub fn contain(&self, key: char) -> bool {
        self.spaces.contains_key(&key)
    }

    pub fn create(&mut self, key: char) {
        let mut space = Container::new(self.context);
        if self.rec.is_some() {
            let r = self.rec.unwrap();
            space.configure(r.x, r.y, r.width, r.height);
        }
        space.category = container::Type::Workspace;
        self.spaces.insert(key, space);

        // update taskbar
        if let Some(bar) = self.taskbar.as_mut() {
            let keys = self.spaces.keys().map(|c| c.clone()).collect::<Vec<char>>();
            bar.load(keys);
            bar.update();
        }
    }

    pub fn delete(&mut self, key: char) {
        self.spaces.remove(&key);
    }

    pub fn get(&mut self, key: char) -> Option<&mut Container>{
        self.spaces.get_mut(&key)
    }

    pub fn current(&mut self) -> &mut Container {
        self.spaces.get_mut(&self.current).unwrap()
    }

    pub fn current_name(&self) -> char {
        self.current
    }

    pub fn switch_workspace(&mut self, new: char){
        if new == self.current {
            return
        }

        if !self.contain(new) {
            // let s = libx::default_screen(context);
            self.create(new);
        }

        let old = self.current;
        if let Some(v) = self.get(old) {
            v.unmap();
            v.update_layout();
        }

        self.current = new;
        if let Some(v) = self.get(new) {
            println!("workspace {}", v.raw_id());
            v.map();
            v.focus();
            v.update_layout();
        }

        // update taskbar
        if let Some(bar) = self.taskbar.as_mut() {
            bar.set_current(self.current);
            bar.update();
        }

    }

    pub fn can_manage(context: libx::Context, id: Window) -> bool {
        let attrs = libx::get_window_attributes(context, id);
        let transientfor_hint = libx::get_transient_for_hint(context, id);
        attrs.override_redirect == 0 && transientfor_hint == 0
    }

    pub fn move_window(&mut self, window: Window, from: char, to: char){
        if from == to {
            return
        }

        if !self.contain(from) {
            self.create(from);
        }
        if !self.contain(to) {
            self.create(to);
        }

        let res = match self.get(from) {
            Some(w) => { w.remove(window) }
            None => { None }
        };

        if let Some(w) = self.get(to) {
            if res.is_some(){
                w.add(res.unwrap());
            }
        }
    }

    pub fn add_window(&mut self, container: Container, workspace: Option<char>) {
        let w = match workspace {
            Some(k) => {
                match self.get(k) {
                    Some(v) => {
                        v
                    }
                    None => {
                        return
                    }
                }
            }
            None => {
                self.current()
            }
        };
        w.add(container);
        w.update_layout();
        w.print_tree(0);
    }

    // insert window just next to old focus
    pub fn insert_window(&mut self, container: Container) {
        if let Some(c) = self.get_focus() {
            if let Some(p) = c.get_parent(){
                let index = p.contain(c.raw_id()).unwrap();
                p.insert(index+1, container);
                p.update_layout();
                p.print_tree(0);
                return
            }
        }
        self.add_window(container, None);
    }

    pub fn remove_window(&mut self, window: Window) -> Option<Container>{
        for (k, workspace) in self.spaces.iter_mut() {
            let res =  workspace.tree_remove(window);
            if let Some(w) = res {
                workspace.update_layout();
                workspace.print_tree(0);
                return Some(w)
            }
        }
        None
    }

    pub fn set_focus(&mut self, window: Window) {
        if let Some(w) = self.get_focus() {
            w.unfocus();
        }

        if let Some((_, c)) = self.get_container(window) {
            c.focus()
        }
    }

    pub fn get_focus(&mut self) -> Option<&mut Container> {
        let (w, _) = libx::get_input_focus(self.context);
        let res = self.get_container(w);
        match res {
            Some((k, c)) => {
                Some(c)
            }
            None => { None }
        }
    }

    pub fn get_container(&mut self, id: Window) -> Option<(char, &mut Container)>{
        for (k, w) in self.spaces.iter_mut() {
            let r = w.tree_search(id);
            if r.is_some(){
                return Some((*k, r.unwrap()))
            }
        }
        None
    }

    // pub fn find_window(&self, window: Window) -> Option<(char, usize)> {
    //     for (k, w) in self.spaces.iter() {
    //         let index = w.contain(window);
    //         match index {
    //             Some(i) => {
    //                 return Some((*k, i))
    //             }
    //             None => {}
    //         }
    //     }
    //     None
    // }
}
