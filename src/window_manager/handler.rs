use std::process::Command;

pub trait Handler {
    fn handle(&mut self);
}

pub struct ExecHandler {
    cmd: Command
}

impl Handler for ExecHandler {
    fn handle(&mut self) {
        self.cmd.spawn();
    }
}
impl ExecHandler {
    pub fn build(tokens: &[&str]) -> ExecHandler {
        let (name, args) = tokens.split_at(1);
        let mut cmd = Command::new(name[0]);

        for arg in args {
            println!("{}", arg);
            cmd.arg(arg);
        }

        let handler = ExecHandler {
            cmd: cmd,
        };
        handler
    }
}
