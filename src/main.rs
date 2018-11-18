#![allow(dead_code)]

// TODO:
// - ...

extern crate ggez;

mod parser;
mod evaluator;

use std::io;
use std::sync::mpsc;
use std::thread;
use std::time;

use ggez::{conf, Context, ContextBuilder};
use ggez::event;
use ggez::graphics::{self, Point2};

const WIDTH:  u32 = 400;
const HEIGHT: u32 = 400;
const COLOR: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
const INJECTED_COMMANDS: &[&str] = &[
  "repeat 6 [fd 50 lt 120 repeat 6 [fd 10 rt 60] rt 120 rt 60]"
  // "repeat 6 [ fd 20 rt 60 ]"
  // "fd 50",
  // "lt 90",
  // "fd 30",
  // "rt 90",
  // "fd 30",
  // "lt 90",
  // "fd 20"
];

fn get_input_receiver() -> mpsc::Receiver<String> {
  let args: Vec<_> = std::env::args().collect();
  let inject_commands = args.len() > 1 && args[1] == "--inject";
  let commands = if inject_commands { INJECTED_COMMANDS } else { &[] };

  let (sender, receiver) = mpsc::channel();

  thread::spawn(move || {
    for command in commands {
      thread::sleep(time::Duration::from_millis(500));
      println!("Sender injecting: {:?}", command);
      sender.send(String::from(*command)).unwrap();
    }
    loop {
      // Get input from stdio and send it to receiver.
      let mut input = String::new();
      io::stdin().read_line(&mut input).unwrap();
      sender.send(input).unwrap();
    }
  });

  receiver
}

struct MainState {
  context: Context,
  continuing: bool,
  events: event::Events,
  receiver: mpsc::Receiver<String>,
  canvas: graphics::Canvas,
  turtle: Option<turtle::Turtle>,
}

impl MainState {
  fn new(mut context: Context) -> Self {
    let events = event::Events::new(&context).unwrap();
    // Create a canvas and set the background color.
    let canvas = graphics::Canvas::with_window_size(&mut context).unwrap();
    graphics::set_canvas(&mut context, Some(&canvas));
    graphics::set_background_color(&mut context, graphics::Color::from((0, 0, 0, 255)));
    graphics::clear(&mut context);
    graphics::set_canvas(&mut context, None);

    Self {
      context: context,
      continuing: true,
      events,
      // Receiver for stdio input.
      receiver: get_input_receiver(),
      canvas,
      turtle: Some(turtle::Turtle::new()),
    }
  }

  fn update(&mut self) {
    if let Ok(input) = self.receiver.try_recv() {
      // TODO: evaluator.feed(input, graphics)
      let mut parser = parser::Parser::new();
      parser.feed(&input);
      let commands = parser.parse_all();
      for command in commands.iter() {
        // Temporarily take turtle out of self so we can call exec_command,
        // otherwise we would have self.turtle.exec_command(command, self)
        // which would complain about mutably borring self twice.
        let mut turtle = self.turtle.take().unwrap();
        turtle.exec_command(command, self);
        self.turtle = Some(turtle);
      }
    }
  }

  // Needs to know screen width & height.  Returns implementation specific
  // point type.
  fn origin_to_screen_coords(&self, point: (f32, f32)) -> Point2 {
    let mut width  = WIDTH as f32;
    let mut height = HEIGHT as f32;
    #[cfg(target_os="macos")]
    {
      println!("We're on Mac!");
      width  /= 2.0;
      height /= 2.0;
    }
    #[cfg(target_os="linux")]
    println!("We're on Linux!");

    println!("{:?} {:?}", width, height);

    let ret = Point2::new(width  + point.0,
                          height - point.1);
    println!("{:?}", ret);
    ret
  }

  fn draw(&mut self) {
    graphics::set_background_color(&mut self.context, graphics::Color::from((0, 0, 0, 255)));
    graphics::clear(&mut self.context);

    // TODO: Use ? for this function.
    graphics::draw_ex(
      &mut self.context,
      &self.canvas,
      graphics::DrawParam {
        // color: Some(graphics::Color::from((255, 255, 255, 255))),
        scale: Point2::new(0.5, 0.5),
        ..Default::default()
      },
    ).unwrap();

    graphics::present(&mut self.context);
  }

  fn handle_events(&mut self) {
    // Tell the timer stuff a frame has happened.
    // Without this the FPS timer functions and such won't work.
    self.context.timer_context.tick();
    // Handle events
    for event in self.events.poll() {
      self.context.process_event(&event);
      match event {
        event::Event::Quit { .. }
        | event::Event::KeyDown {
          keycode: Some(event::Keycode::Escape),
          ..
        } => {
          println!("Quitting");
          self.continuing = false;
        }
        _x => {}, //println!("Event fired: {:?}", _x),
      }
    }
  }
}

impl turtle::TurtleGraphics for MainState {
  fn line(&mut self, p1: (f32, f32), p2: (f32, f32)) {
    let p1 = self.origin_to_screen_coords(p1);
    let p2 = self.origin_to_screen_coords(p2);
    graphics::set_canvas(&mut self.context, Some(&self.canvas));
    // TODO: Put ? on this function call at end, this do_commands function needs to return Result or Option.
    graphics::line(&mut self.context, &[p1, p2], 1.0).unwrap();
    graphics::set_canvas(&mut self.context, None);
  }

  fn clearscreen(&mut self) {
    graphics::set_canvas(&mut self.context, Some(&self.canvas));
    graphics::clear(&mut self.context);
    graphics::set_canvas(&mut self.context, None);
  }
}

pub fn main() {
  let cb = ContextBuilder::new("astroblasto", "ggez")
      .window_setup(conf::WindowSetup::default().title("PC Logo 4.0"))
      .window_mode(conf::WindowMode::default().dimensions(WIDTH, HEIGHT));
  let context = cb.build().unwrap();
  let mut state = MainState::new(context);

  while state.continuing {
    // Update.
    state.update();

    // Draw.
    state.draw();

    // Events.
    state.handle_events();
    ggez::timer::yield_now();
  }
}
