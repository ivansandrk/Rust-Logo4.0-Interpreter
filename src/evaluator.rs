#![allow(dead_code)]

mod lexer;
mod parser;
// use lexer;
// use parser;

// use std;
// use parser;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::mem;
use parser::{AST, ListType, ExprList, ExprLines};
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

#[derive(Default)]
struct LocalState {
  vars: HashMap<String, AST>,
  remainder: VecDeque<AST>,
}

impl LocalState {
  fn new() -> LocalState {
    LocalState {
      ..Default::default()
    }
  }
}

#[derive(Default)]
struct Evaluator {
  parser: parser::Parser,
  turtle: Turtle,
  // Global variables.
  vars: HashMap<String, AST>,

  // Function local variables, or arguments.
  // local_vars: HashMap<String, AST>,
  // remainder: VecDeque<AST>,
  // ^ These two should be part of the function execution context.
  stack_local_state: Vec<LocalState>,

  // some kind of hashmap for functions which supports both builtin and user defined functions
  // User defined functions.
  user_functions: HashMap<String, (ExprList, ExprLines)>,
  // Name and args of the currently defined function.
  name: String,
  args: ExprList,
  lines: ExprLines,
}

impl Evaluator {
  fn new() -> Self {
    let mut evaluator = Evaluator {
      ..Default::default()
    };
    evaluator.stack_local_state.push(LocalState::new());
    evaluator
  }

  fn eval_number(&mut self, ast_node: &AST) -> Result<f32, String> {
    let evaluated_node = self.eval(ast_node)?;
    match evaluated_node {
      AST::Float(float) => {
        Ok(float)
      },
      _ => {
        Err(format!("Expr doesn't evaluate to a number {:?}", ast_node))
      }
    }
  }

  // TODO: Don't allow defining built-in functions.
  fn def_function(&mut self, ast_node: &AST) -> Result<(bool), String> {
    // Already started defining.
    if self.name != "" {
      let mut end = false;
      match ast_node {
        AST::ExprLine(expr_list) => {
          match expr_list.first() {
            Some(AST::Function(name, _)) if name == "END" => {
              end = true;
            },
            _ => {}
          }
        },
        _ => {}
      }
      if end {
        // End of function definition, save it.
        let name = mem::replace(&mut self.name, String::new());
        let args = mem::replace(&mut self.args, ExprList::new());
        let lines = mem::replace(&mut self.lines, ExprLines::new());
        self.user_functions.insert(name, (args, lines));
      } else {
        // Collect the line.
        self.lines.push(ast_node.clone());
      }
      return Ok(true);
    }
    let function;
    match ast_node {
      AST::Function(name, expr_list) if name == "TO" => { function = expr_list; },
      _ => { return Ok(false); }
    }
    match function.first() {
      Some(AST::Function(name, args)) => {
        for arg in args {
          match arg {
            AST::Var(_) => {},
            _ => {
              return Err(format!("The procedure TO does not like {:?} as input.", arg));
            }
          }
        }
        self.name = name.clone();
        self.args = args.clone();
      },
      Some(_) => {
        return Err(format!("The procedure TO needs a name as its first input."));
      },
      None => {
        return Err(format!("TO needs more input(s)."));
      }
    }
    return Ok(true);
  }

  fn pops(&mut self) -> Result<AST, String> {
    for (name, (args, lines)) in self.user_functions.iter() {
      print!("TO {}", name);
      for arg in args {
        match arg {
          AST::Var(var) => { print!(" :{}", var); }
          _ => {}
        }
      }
      println!();
      for line in lines {
        println!("{:?}", line);
      }
      println!("END");
    }
    Ok(AST::None)
  }

  fn local_state(&mut self) -> &mut LocalState {
    self.stack_local_state.last_mut().unwrap()
  }

  fn eval_user_function(&mut self, name: &str) -> Result<AST, String> {
    let args;
    let lines;
    // TODO: If user_functions was using Rc or RefCell, maybe I wouldn't have the problem here.
    match self.user_functions.get(name) {
      Some((_args, _lines)) => {
        args = _args.clone();
        lines = _lines.clone();
      },
      _ => { panic!("Invalid eval_user_function invocation {}", name); }
    }
    let mut new_local_state = LocalState::new();
    // Setup the args as local vars.
    for arg in args {
      let var;
      match arg {
        AST::Var(_var) => { var = _var; }
        _ => { panic!("Invalid function definition {}", name); }
      }
      let next_ast = self.local_state().remainder.pop_front();
      match next_ast {
        Some(ast) => {
          new_local_state.vars.insert(var.to_string(), self.eval(&ast)?);
        },
        None => {
          return Err(format!("{} needs more input(s).", name));
        }
      }
    }
    self.stack_local_state.push(new_local_state);
    let mut ret = AST::None;
    // Run the lines.
    for line in lines {
      ret = self.eval(&line)?;
      match ret {
        AST::FunctionReturn(_) => {
          break;
        },
        AST::None => {},
        _ => {
          return Err(format!(
              "You don't say what to do with the output of {:?}\n\
               In function {}\n\
               Statement   {:?}", ret, name, line));
        }
      }
    }
    self.stack_local_state.pop();
    assert!(self.stack_local_state.len() > 0);
    return Ok(ret);
  }

  fn eval(&mut self, ast_node: &AST) -> Result<AST, String> {
    // We're currently defining a function.
    if self.def_function(ast_node)? {
      return Ok(AST::None);
    }
    let mut ret = AST::None;
    match ast_node {
      AST::Function(name, expr_list) => {
        println!("{:?}", expr_list);
        assert!(self.local_state().remainder.is_empty(), format!("{:?}", self.local_state().remainder));
        self.local_state().remainder = VecDeque::from(expr_list.clone());
        // TODO: Check for user defined & builtin functions here.
        if self.user_functions.contains_key(name) {
          ret = self.eval_user_function(name)?;
        } else {
          match name.as_str() {
            "POPS" => { ret = self.pops()?; },
            _ => {
              return Err(format!("Unknown function {:?}", name));
            }
          }
        }
      },
      AST::ExprLine(expr_list) => {
        for expr in expr_list {
          let result = self.eval(expr)?;
          if result != AST::None {
            ret = result;
            break;
          }
        }
        self.local_state().remainder.clear();
      },
      AST::ExprList(expr_list) => {
        match expr_list.first() {
          Some(first_element) => {
            ret = self.eval(first_element)?;
          },
          None => {
            ret = AST::List(ListType::new());
          }
        }
        self.local_state().remainder.clear();
      },
      AST::Var(var_name) => {
        match self.vars.get(var_name) {
          Some(ast) => {
            ret = ast.clone();
          },
          None => {
            return Err(format!(":{} is not a Logo name.", var_name));
          }
        }
      },
      AST::Float(float) => {
        ret = AST::Float(*float);
      },
      AST::List(list) => {
        // TODO: Try to get rid of this clone somehow.
        ret = AST::List(list.clone());
      },
      AST::Word(string) => {
        ret = AST::Word(string.clone());
      },
      AST::Unary(Token::Negation, box_operand) => {
        let operand = self.eval_number(box_operand)?;
        ret = AST::Float(-operand);
      },
      // TODO: Need to implement all Binary operators.
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
        ret = AST::Float(result);
      },
      AST::Prefix(_operator, _expr_list) => {
        println!("Unimplemented prefix operators");
      },
      _x => {
        println!("Unimplemented eval AST {:?}", _x);
      }
    }
    return Ok(ret);
  }

  fn feed(&mut self, input: &str) {
    // println!("{:?}", input);
    let ast;
    match self.parser.parse(input) {
      Ok(val) => {
        ast = val;
        println!("{:?}", ast);
        parser::rek_print(&ast, "".to_string());
      },
      Err(err) => {
        println!("Parsing error: {:?}", err);
        return;
      },
    }
    println!("{}", format!("Eval: {:?}", self.eval(&ast)).replace("([", "[").replace("])", "]"));
    // TODO: Occasionally try to run the following to make sure nothing is being lost from ast.
    // println!("{}", format!("Eval: {:?}", self.eval(&ast)).replace("([", "[").replace("])", "]"));
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
