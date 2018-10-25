#![allow(dead_code)]

// TODO:
// - ...

use std;

use parser;

pub trait TurtleGraphics {
  // Draws a line from p1 to p2 using window center as origin point (0, 0), and
  // having the x-axis grow left->right, and y-axis down->up.
  fn line(&mut self, p1: (f32, f32), p2: (f32, f32));

  // Clears the screen.
  fn clearscreen(&mut self);
}

#[derive(Default, Debug,)]
pub struct Turtle {
  heading: f32, // 0 .. 359 degrees
  x: f32,
  y: f32,
}

impl Turtle {
  pub fn new() -> Turtle {
    Turtle { ..Default::default() }
  }

  pub fn exec_command(&mut self, command: &parser::Command, graphics: &mut TurtleGraphics) {
    match *command {
      parser::Command::Fd(val) => self.fd(val, graphics),
      parser::Command::Bk(val) => self.bk(val, graphics),
      parser::Command::Lt(val) => self.lt(val),
      parser::Command::Rt(val) => self.rt(val),
      parser::Command::Cs      => graphics.clearscreen(),
      parser::Command::Repeat(cnt, ref boxed_command) => {
        for _ in 0 .. cnt {
          self.exec_command(boxed_command, graphics);
        }
      },
      parser::Command::Block(ref block_commands) => {
        for command in block_commands.iter() {
          self.exec_command(command, graphics);
        }
      },
      _ => (),
    }
  }

  fn fd(&mut self, val: f32, graphics: &mut TurtleGraphics) {
    let phi = (self.heading + 90.0) * std::f32::consts::PI / 180.0;
    let new_x = self.x + val * phi.cos();
    let new_y = self.y + val * phi.sin();
    graphics.line((self.x, self.y), (new_x, new_y));
    self.x = new_x;
    self.y = new_y;
  }

  fn bk(&mut self, val: f32, graphics: &mut TurtleGraphics) {
    self.fd(-val, graphics);
  }

  fn lt(&mut self, val: f32) {
    // TODO: Clamp the heading perhaps to only [0, 360).
    self.heading += val;
  }

  fn rt(&mut self, val: f32) {
    self.lt(-val);
  }
}
