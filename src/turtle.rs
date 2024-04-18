pub trait Graphics {
  // Draws a line from p1 to p2 using window center as origin point (0, 0), and
  // having the x-axis grow left->right, and y-axis down->up.
  fn line(&mut self, p1: (f32, f32), p2: (f32, f32));

  // Clears the screen.
  fn clearscreen(&mut self);
}

#[derive(Default, Debug)]
pub struct GraphicsStub {
  commands: Vec<String>,
}

impl Graphics for GraphicsStub {
  fn line(&mut self, p1: (f32, f32), p2: (f32, f32)) {
    self.commands.push(format!("line {},{} {},{}", p1.0, p1.1, p2.0, p2.1));
  }

  fn clearscreen(&mut self) {
    self.commands.push(format!("clearscreen"));
  }
}

impl GraphicsStub {
  pub fn new() -> GraphicsStub {
    GraphicsStub {
      ..Default::default()
    }
  }
}

pub struct Turtle<'a> {
  graphics: &'a mut dyn Graphics,
  heading: f32, // 0 .. 359 degrees
  x: f32,
  y: f32,
  pendown: bool,
}

impl<'a> Turtle<'a> {
  pub fn new(graphics: &mut dyn Graphics) -> Turtle {
    Turtle {
      graphics: graphics,
      heading: 0.0,
      x: 0.0,
      y: 0.0,
      pendown: true,
    }
  }

  pub fn setxy(&mut self, x: f32, y: f32) {
    if self.pendown {
      self.graphics.line((self.x, self.y), (x, y));
    }
    self.x = x;
    self.y = y;
  }

  pub fn getxy(&mut self) -> (f32, f32) {
    (self.x, self.y)
  }

  pub fn setheading(&mut self, heading: f32) {
    self.heading = heading;
  }

  pub fn heading(&mut self) -> f32 {
    self.heading
  }

  pub fn setx(&mut self, x: f32) {
    let y = self.y;
    self.setxy(x, y);
  }

  pub fn xcor(&mut self) -> f32 {
    self.x
  }

  pub fn sety(&mut self, y: f32) {
    let x = self.x;
    self.setxy(x, y);
  }

  pub fn ycor(&mut self) -> f32 {
    self.y
  }

  pub fn fd(&mut self, val: f32) {
    let phi = (self.heading + 90.0) * std::f32::consts::PI / 180.0;
    let new_x = self.x + val * phi.cos();
    let new_y = self.y + val * phi.sin();
    self.setxy(new_x, new_y);
  }

  pub fn bk(&mut self, val: f32) {
    self.fd(-val);
  }

  pub fn lt(&mut self, val: f32) {
    // TODO: Clamp the heading perhaps to only [0, 360).
    self.heading += val;
  }

  pub fn rt(&mut self, val: f32) {
    self.lt(-val);
  }

  pub fn home(&mut self) {
    self.setxy(0.0, 0.0);
    self.setheading(0.0);
  }

  pub fn clean(&mut self) {
    self.graphics.clearscreen();
  }

  pub fn clearscreen(&mut self) {
    self.home();
    self.clean();
  }

  pub fn pendown(&mut self) {
    self.pendown = true;
  }

  pub fn penup(&mut self) {
    self.pendown = false;
  }
}

// TODO: Turtle tests with NullGraphics.
#[cfg(test)]
mod tests {
  use super::*;

  // fn test_ok(input: &str, expected: &[Token]) {
  //   let lexed = Lexer::new(input).process();
  //   let expected = Ok(expected.to_vec());
  //   assert_eq!(expected, lexed);
  // }

  // fn test_err(input: &str, expected: &str) {
  //   let lexed = Lexer::new(input).process();
  //   let expected = Err(expected.to_string());
  //   assert_eq!(expected, lexed);
  // }

  #[test]
  fn draw_square() {
    //  let mut evaluator = Evaluator::new(Box::new(turtle::NullGraphics::new()));
    // TODO: let mut? g = NullGraphics::new();
    let mut stub = GraphicsStub::new();
    // let graphics = Box::new(spy);
    let mut turtle = Turtle::new(&mut stub);
    turtle.fd(10.0);
    let expected: Vec<String> = vec![];  
    println!("{:?}", turtle.x);
    println!("{:?}", turtle.y);
    println!("{:?}", turtle.heading);
    println!("{:?}", turtle.pendown);
    println!("{:?}", stub.commands);
    assert_eq!(true, false);
    // assert_eq!(expected, actual);
  }
}
