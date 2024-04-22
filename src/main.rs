#![allow(dead_code)]

mod lexer;
mod parser;
mod turtle;
mod evaluator;

// use std::io;
// use std::sync::mpsc;
// use std::thread;
// use std::time;
// use std::rc::Rc;
// use std::cell::RefCell;

// use ggez::{conf, Context, ContextBuilder};
// use ggez::event;
// use ggez::graphics;
// use ggez::mint::Point2;

const WIDTH:  f32 = 1000.;
const HEIGHT: f32 = 1000.;
const COLOR: [f32; 4] = [0., 1., 0., 1.];
const INJECTED_COMMANDS: &[&str] = &[
  "repeat 6 [fd 50 lt 120 repeat 6 [fd 10 rt 60] rt 120 rt 60]"
];

// TO MN :A :N
// MAKE "L []
// REPEAT :N [FD :A RT 360 / :N MAKE "L LPUT GETXY :L]
// FOR "I 1 :N [FOR "J 1 :N [SETXY ITEM :I :L SETXY ITEM :J :L]]
// END


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


//   fn draw(&mut self) {
//     graphics::set_background_color(&mut self.context.borrow_mut(), graphics::Color::from((0, 0, 0, 255)));
//     graphics::clear(&mut self.context.borrow_mut());
//     // Draw canvas.
//     // TODO: Use ? for this function.
//     graphics::draw_ex(
//       &mut self.context.borrow_mut(),
//       &*self.canvas.borrow(),
//       graphics::DrawParam {
//         // color: Some(graphics::Color::from((255, 255, 255, 255))),
//         scale: Point2::new(0.5, 0.5),
//         ..Default::default()
//       },
//     ).unwrap();
//     // TODO: Draw the turtle.
//     // println!("{} {}", self.evaluator.turtle.x, self.evaluator.turtle.y);
//     graphics::present(&mut self.context.borrow_mut());
//   }

// // Needs to know screen width & height.  Returns implementation specific point type.
// fn origin_to_screen_coords(point: (f32, f32)) -> Point2 {
//   let mut width  = WIDTH as f32;
//   let mut height = HEIGHT as f32;
//   #[cfg(target_os="macos")]
//   {
//     width  /= 2.0;
//     height /= 2.0;
//   }
//   Point2::new(width  + point.0,
//               height - point.1)
// }

// struct GgezGraphics {
//   context: Rc<RefCell<ggez::Context>>,
//   canvas: Rc<RefCell<graphics::Canvas>>,
// }

// impl GgezGraphics {
//   fn new(context: Rc<RefCell<ggez::Context>>,
//          canvas: Rc<RefCell<graphics::Canvas>>) -> GgezGraphics {
//     GgezGraphics {
//       context,
//       canvas,
//       // context: context.clone(),
//       // canvas: canvas.clone(),
//     }
//   }
// }

// impl evaluator::Graphics for GgezGraphics {
//   fn line(&mut self, p1: (f32, f32), p2: (f32, f32)) {
//     let p1 = origin_to_screen_coords(p1);
//     let p2 = origin_to_screen_coords(p2);
//     graphics::set_canvas(&mut self.context.borrow_mut(), Some(&self.canvas.borrow()));
//     // TODO: Put ? on this function call at end, this do_commands function needs to return Result or Option.
//     graphics::line(&mut self.context.borrow_mut(), &[p1, p2], 1.0).unwrap();
//     graphics::set_canvas(&mut self.context.borrow_mut(), None);
//   }

//   fn clearscreen(&mut self) {
//     graphics::set_canvas(&mut self.context.borrow_mut(), Some(&self.canvas.borrow()));
//     graphics::clear(&mut self.context.borrow_mut());
//     graphics::set_canvas(&mut self.context.borrow_mut(), None);
//   }
// }

struct MainState {
  canvas: ggez::graphics::Canvas,
  receiver: std::sync::mpsc::Receiver<String>,
  graphics: turtle::GraphicsStub,
  evaluator: evaluator::Evaluator,
}

impl MainState {
  fn new(ctx: &ggez::Context) -> ggez::GameResult<Self> {
    let canvas = ggez::graphics::Canvas::from_frame(
      ctx,
      ggez::graphics::Color::from([1., 1., 1., 1.]));
    // graphics::set_canvas(&mut context, Some(&canvas));
    // graphics::set_background_color(&mut context, graphics::Color::from((0, 0, 0, 255)));
    // graphics::clear(&mut context);
    // graphics::set_canvas(&mut context, None);

    let receiver = get_input_receiver();
    let graphics = turtle::GraphicsStub::new();
    let evaluator = evaluator::Evaluator::new(Box::new(graphics.clone()));

    Ok(Self {
      canvas,
      receiver,
      graphics,
      evaluator,
    })
  }
}

impl ggez::event::EventHandler<ggez::GameError> for MainState {
  fn update(&mut self, _ctx: &mut ggez::Context) -> ggez::GameResult {
    if let Ok(input) = self.receiver.try_recv() {
      self.evaluator.feed(&input);
    }
    Ok(())
  }
  fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
    // let invocations = self.graphics.invocations.replace(Vec::new());
    let mut canvas = ggez::graphics::Canvas::from_frame(
      ctx,
      ggez::graphics::Color::from([0., 0., 0., 1.]));
    for cmd in self.graphics.invocations.borrow().iter() {
      match cmd {
        turtle::Command::Line(p1, p2) => {
          let line = ggez::graphics::Mesh::new_line(
            ctx,
            &[ggez::glam::Vec2::new(p1.0, p1.1), ggez::glam::Vec2::new(p2.0, p2.1)],
            1.,
            [1., 1., 1., 1.].into()
          )?;
          canvas.draw(&line, ggez::glam::Vec2::new(WIDTH/2., HEIGHT/2.));
        },
        turtle::Command::Clear => {
          // TODO: Implement clear.
        },
      }
    }
    canvas.finish(ctx)?;
    // Canvas::from_frame draws straight to screen.
    // let circle = ggez::graphics::Mesh::new_circle(
    //   ctx,
    //   ggez::graphics::DrawMode::fill(),
    //   ggez::glam::Vec2::new(0.0, 0.0),
    //   100.0,
    //   2.0,
    //   ggez::graphics::Color::WHITE,
    // )?;
    // canvas.draw(&circle, ggez::glam::Vec2::new(10.0, 380.0));
    // canvas.finish(ctx)?;
    Ok(())
  }
}

fn main() -> ggez::GameResult {
  let cb = ggez::ContextBuilder::new("Logo", "ggez")
      .window_setup(ggez::conf::WindowSetup::default().title("PC Logo 4.0"))
      .window_mode(ggez::conf::WindowMode::default().dimensions(WIDTH, HEIGHT));
  let (ctx, event_loop) = cb.build()?;
  let state = MainState::new(&ctx)?;
  ggez::event::run(ctx, event_loop, state)
}
