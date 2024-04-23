extern crate ggez;

mod lexer;
mod parser;
mod turtle;
mod evaluator;

use ggez::graphics::{Canvas, ScreenImage, ImageFormat, Color, Mesh, DrawParam};
use ggez::conf::{WindowSetup, WindowMode};
use ggez::{Context, ContextBuilder, GameResult, GameError};
use ggez::event;
use ggez::glam::Vec2;

const WIDTH:  f32 = 1000.;
const HEIGHT: f32 = 1000.;
const INJECTED_COMMANDS: &[&str] = &[
  "repeat 6 [fd 50 lt 120 repeat 6 [fd 10 rt 60] rt 120 rt 60]"
];

fn get_input_receiver() -> std::sync::mpsc::Receiver<String> {
  let args: Vec<_> = std::env::args().collect();
  let inject_commands = args.len() > 1 && args[1] == "--inject";
  let commands = if inject_commands { INJECTED_COMMANDS } else { &[] };
  let (sender, receiver) = std::sync::mpsc::channel();
  std::thread::spawn(move || {
    for command in commands {
      std::thread::sleep(std::time::Duration::from_millis(500));
      println!("Sender injecting: {:?}", command);
      sender.send(String::from(*command)).unwrap();
    }
    loop {
      // Get input from stdio and send it to receiver.
      let mut input = String::new();
      std::io::stdin().read_line(&mut input).unwrap();
      sender.send(input).unwrap();
    }
  });
  receiver
}

struct MainState {
  screen: ScreenImage,
  receiver: std::sync::mpsc::Receiver<String>,
  evaluator: evaluator::Evaluator,
  graphics: turtle::GraphicsStub,
}

impl MainState {
  fn new(ctx: &Context) -> GameResult<Self> {
    let graphics = turtle::GraphicsStub::new();

    Ok(Self {
      screen: ScreenImage::new(ctx, ImageFormat::Rgba8UnormSrgb, 1., 1., 1),
      receiver: get_input_receiver(),
      evaluator: evaluator::Evaluator::new(Box::new(graphics.clone())),
      graphics,
    })
  }
}

impl event::EventHandler<GameError> for MainState {
  fn update(&mut self, _ctx: &mut Context) -> GameResult {
    if let Ok(input) = self.receiver.try_recv() {
      self.evaluator.feed(&input);
    }
    Ok(())
  }
  fn draw(&mut self, ctx: &mut Context) -> GameResult {
    let invocations = self.graphics.invocations.replace(Vec::new());
    let mut canvas = Canvas::from_screen_image(ctx, &mut self.screen, None);
    for cmd in invocations {
      match cmd {
        turtle::Command::Line(p1, p2) => {
          let points = &[Vec2::new(p1.0, p1.1), Vec2::new(p2.0, p2.1)];
          let line = Mesh::new_line(ctx, points, 1., Color::WHITE)?;
          // Transform the Logo coords to screen coords:
          // * Flip y axis.
          // * Translate to screen center.
          let draw_param = DrawParam::new()
            .scale(Vec2::new(1., -1.))
            .dest(Vec2::new(WIDTH / 2., HEIGHT / 2.));
          canvas.draw(&line, draw_param);
        },
        turtle::Command::Clear => {
          canvas = Canvas::from_screen_image(ctx, &mut self.screen, Some(Color::BLACK))
        },
      }
    }
    canvas.finish(ctx)?;
    ctx.gfx.present(&self.screen.image(ctx))?;
    Ok(())
  }
}

fn main() -> GameResult {
  let cb = ContextBuilder::new("Logo", "ggez")
      .window_setup(WindowSetup::default().title("PC Logo 4.0"))
      .window_mode(WindowMode::default().dimensions(WIDTH, HEIGHT));
  let (ctx, event_loop) = cb.build()?;
  let state = MainState::new(&ctx)?;
  event::run(ctx, event_loop, state)
}
