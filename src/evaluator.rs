#![allow(dead_code)]

mod lexer;
mod parser;
// use lexer;
// use parser;

// use std;
// use parser;
use std::collections::VecDeque;
use parser::AST;
use lexer::Token;

pub trait Graphics {
  // Draws a line from p1 to p2 using window center as origin point (0, 0), and
  // having the x-axis grow left->right, and y-axis down->up.
  fn line(&mut self, p1: (f32, f32), p2: (f32, f32));

  // Clears the screen.
  fn clearscreen(&mut self);
}

struct NullGraphics {
  commands: Vec<String>,
}

impl Graphics for NullGraphics {
  fn line(&mut self, p1: (f32, f32), p2: (f32, f32)) {
    self.commands.push(format!("line {},{} {},{}", p1.0, p1.1, p2.0, p2.1));
  }

  fn clearscreen(&mut self) {
    self.commands.push(format!("clearscreen"));
  }
}

#[derive(Default, Debug)]
pub struct Turtle {
  heading: f32, // 0 .. 359 degrees
  x: f32,
  y: f32,
}

impl Turtle {
  pub fn new() -> Turtle {
    Turtle { ..Default::default() }
  }

  fn fd(&mut self, val: f32, graphics: &mut Graphics) {
    let phi = (self.heading + 90.0) * std::f32::consts::PI / 180.0;
    let new_x = self.x + val * phi.cos();
    let new_y = self.y + val * phi.sin();
    graphics.line((self.x, self.y), (new_x, new_y));
    self.x = new_x;
    self.y = new_y;
  }

  fn bk(&mut self, val: f32, graphics: &mut Graphics) {
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

struct Evaluator {
  turtle: Turtle,
}

impl Evaluator {
  fn new() -> Self {
    Self {
      turtle: Turtle::new(),
    }
  }

// type ExprList = Vec<AST>;
// type ExprLines = Vec<ExprList>;
// #[derive(Debug, Clone, PartialEq)]
// pub enum AST {
//   Unary(Token, Box<AST>),  // TODO: enum Number here and below.
//   Binary(Token, Box<AST>, Box<AST>),
//   Prefix(Token, ExprList),  // Prefix style arithmetic operations, ie. + 3 5 = 8.
//   Float(f32),
//   DefFunction(String, ExprList, ExprLines),  // name, input args (all Var), body
//   CallFunction(String, ExprList),  // name, arguments and rest
//   Var(String),  // :ASD
//   Word(String),  // "BIRD
//   List(VecDeque<AST>), // [1 2 MAKE "A "BSD]
//   ExprList(ExprList),  // Exprs inside of parens
//   ExprLine(ExprList),  // Line of exprs
//   // ExprList line & ExprList lines ? Because line is only evaluated once, ie. ? 1 * 2 3 -> 2 (3 is ignored).

  // TODO: Eval number, or eval float/int, what should be the return type?
  fn eval_number(&mut self, ast_node: &AST) -> Result<f32, String> {
    let evaluated_node = self.eval(ast_node)?;
    match evaluated_node {
      Some(AST::Float(float)) => {
        Ok(float)
      },
      _ => {
        Err(format!("Expr doesn't evaluate to a number {:?}", ast_node))
      }
    }
  }

  fn eval(&mut self, ast_node: &AST) -> Result<Option<AST>, String> {
    let mut ret = None;
    match ast_node {
      AST::ExprLine(expr_list) | AST::ExprList(expr_list) => {
        for expr in expr_list {
          let result = self.eval(expr)?;
          if result.is_some() {
            ret = result;
          }
        }
      },
      AST::Float(float) => {
        ret = Some(AST::Float(*float));
      },
      AST::Unary(operator, box_operand) => {
        let operand = self.eval_number(box_operand)?;
        let result = match operator {
          Token::Negation => { -operand },
          _ => {
            panic!("Unknown unary operator {:?}", operator);
          }
        };
        ret = Some(AST::Float(result));
      },
      AST::Binary(operator, left_box, right_box) => {
        let left = self.eval_number(left_box)?;
        let right = self.eval_number(right_box)?;
        let result = match operator {
          Token::Plus => { left + right },
          Token::Minus => { left - right },
          Token::Multiply => { left * right },
          Token::Divide => { left / right },
          _ => {
            panic!("Unknown binary operator {:?}", operator);
          }
        };
        ret = Some(AST::Float(result));
      },
      _x => {
        println!("Unimplemented eval AST {:?}", _x);
      }
    }
    return Ok(ret);
  }

  fn feed(&mut self, input: &str) {
    println!("{:?}", input);
    let tokens;
    // TODO: Don't do any parsing as long as tokens end on LineCont.
    match lexer::Lexer::new(input).process() {
      Ok(val) => tokens = val,
      Err(err) => { println!("Tokenizing error: {:?}", err); return; }
    }
    let mut queue: VecDeque<lexer::Token> = tokens.into_iter().collect();
    println!("{:?}", queue);
    let ast;
    match parser::parse_line(&mut queue) {
      Ok(val) => {
        ast = val;
        println!("{:?}", ast);
        // rek_print(&val, "".to_string());
      },
      Err(err) => {
        println!("Parsing error: {:?}", err);
        return;
      },
    }
    println!("Eval: {:?}", self.eval(&ast));
    println!("Eval: {:?}", self.eval(&ast));
  }
}

fn main() {
  // 1 + (2 * (3 + 4 * -5) + -6 * -(-7 + -8)) * 9
  let mut evaluator = Evaluator::new();
  loop {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    // pratt_parse_debug(input.trim());
    evaluator.feed(&input);
  }
}

  // pub fn exec_command(&mut self, command: &parser::Command, graphics: &mut Graphics) {
  //   match *command {
  //     parser::Command::Fd(val) => self.fd(val, graphics),
  //     parser::Command::Bk(val) => self.bk(val, graphics),
  //     parser::Command::Lt(val) => self.lt(val),
  //     parser::Command::Rt(val) => self.rt(val),
  //     parser::Command::Cs      => graphics.clearscreen(),
  //     parser::Command::Repeat(cnt, ref boxed_command) => {
  //       for _ in 0 .. cnt {
  //         self.exec_command(boxed_command, graphics);
  //       }
  //     },
  //     parser::Command::Block(ref block_commands) => {
  //       for command in block_commands.iter() {
  //         self.exec_command(command, graphics);
  //       }
  //     },
  //     _ => (),
  //   }
  // }

