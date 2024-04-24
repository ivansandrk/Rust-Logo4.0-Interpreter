pub trait Graphics {
  // Draws a line from p1 to p2 using window center as origin point (0, 0), and
  // having the x-axis grow left->right, and y-axis down->up.
  fn line(&mut self, p1: (f32, f32), p2: (f32, f32));

  // Clears the screen.
  fn clear(&mut self);
}

pub struct Turtle {
  graphics: Box<dyn Graphics>,
  heading: f32, // 0 .. 359 degrees
  x: f32,
  y: f32,
  pendown: bool,
}

impl Turtle {
  pub fn new(graphics: Box<dyn Graphics>) -> Turtle {
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
    self.graphics.clear();
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

#[derive(Debug, Clone)]
pub enum Command {
  Line((f32, f32), (f32, f32)), // p1, p2
  Clear,
}

fn feq(a: f32, b: f32) -> bool {
  // TODO: There must be a better way of handling this.
  (a - b).abs() < (f32::EPSILON * 100.0)
}

impl PartialEq for Command {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (&Command::Line(s1, s2), &Command::Line(o1, o2)) => {
        feq(s1.0, o1.0) && feq(s1.1, o1.1) &&
        feq(s2.0, o2.0) && feq(s2.1, o2.1)
      },
      (&Command::Clear, &Command::Clear) => {true},
      _ => false,
    }
  }
}

// Returns a Vec of Command::Line connecting all the points:
// [(p0, p1), (p1, p2), ..., (pN-1, pN)]
#[allow(dead_code)]
pub fn points_to_line_commands(points: &Vec<(f32, f32)>) -> Vec<Command> {
  let mut ret = vec![];
  for i in 0..points.len().max(1)-1 {
    ret.push(Command::Line(points[i], points[i + 1]));
  }
  ret
}

#[macro_export]
macro_rules! CON {
  ( $( $x:expr ),* $(,)? ) => {
      {
          #[allow(unused_mut)] // The linter gets this totally wrong.
          let mut temp_vec = Vec::new();
          $(
              temp_vec.push($x);
          )*
          use turtle;
          turtle::points_to_line_commands(&temp_vec)
      }
  };
}

#[derive(Default, Debug, Clone)]
pub struct GraphicsStub {
  pub invocations: std::rc::Rc<std::cell::RefCell<Vec<Command>>>,
}

impl Graphics for GraphicsStub {
  fn line(&mut self, p1: (f32, f32), p2: (f32, f32)) {
    self.invocations.borrow_mut().push(Command::Line(p1, p2));
  }

  fn clear(&mut self) {
    self.invocations.borrow_mut().push(Command::Clear);
  }
}

impl GraphicsStub {
  pub fn new() -> GraphicsStub {
    GraphicsStub {
      ..Default::default()
    }
  }
}

#[cfg(test)]
mod tests {
  #![allow(non_snake_case)]
  use super::*;

  // TODO: Make a macro to compress the test code even more, test_turtle(|mut turtle| { ... }) can be shortened more.

  #[test]
  fn test_points_to_line_commands() {
    let points = vec![(0.0, 0.0), (0.0, 1.0), (1.0, 1.0), (1.0, 0.0)];
    let expected = vec![
      Command::Line((0.0, 0.0), (0.0, 1.0)),
      Command::Line((0.0, 1.0), (1.0, 1.0)),
      Command::Line((1.0, 1.0), (1.0, 0.0))
    ];
    let actual = points_to_line_commands(&points);
    assert_eq!(expected, actual);
  }

  fn test_turtle<F>(test: F) where F: FnOnce(Turtle) -> Vec<Command> {
    let stub = GraphicsStub::new();
    let turtle = Turtle::new(Box::new(stub.clone()));
    let expected = test(turtle);
    let actual = (*stub.invocations).take();
    assert_eq!(expected, actual);
  }

  #[test]
  fn test_fd() {
    test_turtle(
      |mut turtle| {
        turtle.fd(10.0);
        CON!((0.0, 0.0), (0.0, 10.0))
      }
    );
  }

  #[test]
  fn test_bk() {
    test_turtle(
      |mut turtle| {
        turtle.bk(10.0);
        CON!((0.0, 0.0), (0.0, -10.0))
      }
    );
  }

  #[test]
  fn test_rt() {
    test_turtle(
      |mut turtle| {
        turtle.rt(90.0);
        turtle.fd(10.0);
        CON!((0.0, 0.0), (10.0, 0.0))
      }
    );
  }

  #[test]
  fn test_lt() {
    test_turtle(
      |mut turtle| {
        turtle.lt(90.0);
        turtle.fd(10.0);
        CON!((0.0, 0.0), (-10.0, 0.0))
      }
    );
  }

  #[test]
  fn test_setxy() {
    test_turtle(
      |mut turtle| {
        turtle.setxy(10.0, 20.0);
        CON!((0.0, 0.0), (10.0, 20.0))
      }
    );
  }

  #[test]
  fn test_getxy() {
    let (mut x, mut y) = (0.0, 0.0);
    test_turtle(
      |mut turtle| {
        turtle.fd(20.0);
        turtle.rt(90.0);
        turtle.fd(10.0);
        (x, y) = turtle.getxy();
        CON!(
          (0.0, 0.0),
          (0.0, 20.0),
          (10.0, 20.0),
        )
      }
    );
    assert!(feq(10.0, x));
    assert!(feq(20.0, y));
  }

  #[test]
  fn test_setheading() {
    test_turtle(
      |mut turtle| {
        turtle.setheading(180.0);
        turtle.fd(10.0);
        CON!((0.0, 0.0), (0.0, -10.0))
      }
    );
  }

  #[test]
  fn test_getheading() {
    let mut heading = 0.0;
    test_turtle(
      |mut turtle| {
        turtle.lt(120.0);
        heading = turtle.heading();
        CON!()
      }
    );
    assert!(feq(120.0, heading));
  }

  #[test]
  fn test_setx() {
    test_turtle(
      |mut turtle| {
        turtle.setx(15.0);
        CON!((0.0, 0.0), (15.0, 0.0))
      }
    );
  }

  #[test]
  fn test_xcor() {
    let mut x = 0.0;
    test_turtle(
      |mut turtle| {
        turtle.lt(90.0);
        turtle.fd(5.0);
        x = turtle.xcor();
        CON!((0.0, 0.0), (-5.0, 0.0))
      }
    );
    assert!(feq(-5.0, x));
  }

  #[test]
  fn test_sety() {
    test_turtle(
      |mut turtle| {
        turtle.sety(25.0);
        CON!((0.0, 0.0), (0.0, 25.0))
      }
    );
  }

  #[test]
  fn test_ycor() {
    let mut y = 0.0;
    test_turtle(
      |mut turtle| {
        turtle.fd(30.0);
        y = turtle.ycor();
        CON!((0.0, 0.0), (0.0, 30.0))
      }
    );
    assert!(feq(30.0, y));
  }

  #[test]
  fn test_home() {
    test_turtle(
      |mut turtle| {
        turtle.rt(90.0);
        turtle.fd(10.0);
        turtle.home();
        turtle.fd(10.0);
        CON!((0.0, 0.0), (10.0, 0.0), (0.0, 0.0), (0.0, 10.0))
      }
    );
  }

  #[test]
  fn test_clean() {
    test_turtle(
      |mut turtle| {
        turtle.fd(10.0);
        turtle.clean();
        vec![
          Command::Line((0.0, 0.0), (0.0, 10.0)),
          Command::Clear,
        ]
      }
    );
  }

  #[test]
  fn test_clearscreen() {
    test_turtle(
      |mut turtle| {
        turtle.rt(90.0);
        turtle.fd(10.0);
        turtle.clearscreen();
        turtle.fd(10.0);
        vec![
          Command::Line((0.0, 0.0), (10.0, 0.0)),
          Command::Line((10.0, 0.0), (0.0, 0.0)),
          Command::Clear,
          Command::Line((0.0, 0.0), (0.0, 10.0)),
        ]
      }
    );
  }

  #[test]
  fn test_penupdown() {
    test_turtle(
      |mut turtle| {
        turtle.fd(10.0);
        turtle.penup();
        turtle.fd(10.0);
        turtle.pendown();
        turtle.fd(10.0);
        vec![
          Command::Line((0.0, 0.0), (0.0, 10.0)),
          Command::Line((0.0, 20.0), (0.0, 30.0)),
        ]
      }
    );
  }

  #[test]
  fn draw_square() {
    let expected = CON!(
      (0.0, 0.0), (0.0, 1.0), (1.0, 1.0), (1.0, 0.0), (0.0, 0.0));
    let code = |mut turtle: Turtle| {
      for _ in 0..4 {
        turtle.fd(1.0);
        turtle.rt(90.0);
      }
      expected
    };
    test_turtle(code);
  }
}
