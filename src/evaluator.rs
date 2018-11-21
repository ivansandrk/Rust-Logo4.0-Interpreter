#![allow(dead_code)]

// *** TODOs
// Define WordType
// user functions should store args as Strings, not AST::Var(String)

mod lexer;
mod parser;
// use lexer;
// use parser;

// use std;
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
struct Evaluator {
  parser: parser::Parser,
  turtle: Turtle,

  // Global variables.
  vars: HashMap<String, AST>,

  // Function local variables.
  stack_vars: Vec<HashMap<String, AST>>,
  // Remaining expressions.  ExprLine evaluation, List evaluation, and function evaluation each get
  // their own (the parent ones are preserved).
  stack_rem: Vec<VecDeque<AST>>,

  builtin_functions: HashMap<String, Box<Fn(&mut Evaluator) -> Result<AST, String>>>,
  user_functions: HashMap<String, (ExprList, ExprLines)>,

  // Name, args, and lines of the currently defined function.
  name: String,
  args: ExprList,
  lines: ExprLines,
}

impl Evaluator {
  fn new() -> Self {
    let mut evaluator = Evaluator {
      ..Default::default()
    };
    evaluator.stack_vars.push(HashMap::new());
    evaluator.define_builtins();
    evaluator
  }

  fn define_builtins(&mut self) {
    self.builtin_functions.insert("OP".to_string(), Box::new(|evaluator| {
      Ok(AST::FunctionReturn(Box::new(evaluator.eval_next_remainder()?)))
    }));
    self.builtin_functions.insert("OUTPUT".to_string(), Box::new(|evaluator| {
      Ok(AST::FunctionReturn(Box::new(evaluator.eval_next_remainder()?)))
    }));
    self.builtin_functions.insert("POPS".to_string(), Box::new(|evaluator| {
      for (name, (args, lines)) in evaluator.user_functions.iter() {
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
    }));
    self.builtin_functions.insert("PONS".to_string(), Box::new(|evaluator| {
      evaluator.print_locals();
      evaluator.print_globals();
      Ok(AST::None)
    }));
    self.builtin_functions.insert("MAKE".to_string(), Box::new(|evaluator| {
      let var = evaluator.get_next_word()?;
      let expr = evaluator.eval_next_remainder()?;
      if evaluator.local_vars().contains_key(&var) {
        evaluator.local_vars().insert(var, expr);
      } else {
        evaluator.vars.insert(var, expr);
      }
      Ok(AST::None)
    }));
    self.builtin_functions.insert("REPEAT".to_string(), Box::new(|evaluator| {
      let repeat = evaluator.get_next_number()?;
      let list = evaluator.get_next_list()?;
      for _ in 0 .. repeat as i32 {
        evaluator.eval_list(&list)?;
      }
      Ok(AST::None)
    }));
  }

  fn print_locals(&mut self) {
    println!("Locals:");
    for (var, expr) in self.local_vars().iter() {
      println!("{} is {:?}", var, expr);
    }
  }

  fn print_globals(&mut self) {
    println!("Globals:");
    for (var, expr) in self.vars.iter() {
      println!("{} is {:?}", var, expr);
    }
  }

  // TODO: eval_next_as_number, eval_next, as_number ?
  fn get_number(&mut self, ast_node: &AST) -> Result<f32, String> {
    match self.eval(ast_node)? {
      AST::Float(float) => { Ok(float) },
      _ => { Err(format!("Expr doesn't evaluate to a number {:?}", ast_node)) }
    }
  }

  fn get_list(&mut self, ast_node: &AST) -> Result<ListType, String> {
    match self.eval(ast_node)? {
      AST::List(list) => { Ok(list) },
      _ => { Err(format!("Expr doesn't evaluate to a list {:?}", ast_node)) }
    }
  }

  fn get_word(&mut self, ast_node: &AST) -> Result<String, String> {
    match self.eval(ast_node)? {
      AST::Word(word) => { Ok(word) },
      _ => { Err(format!("Expr doesn't evaluate to a word {:?}", ast_node)) }
    }
  }

  fn get_next_number(&mut self) -> Result<f32, String> {
    let next_ast = self.eval_next_remainder()?;
    self.get_number(&next_ast)
  }

  fn get_next_list(&mut self) -> Result<ListType, String> {
    let next_ast = self.eval_next_remainder()?;
    self.get_list(&next_ast)
  }

  fn get_next_word(&mut self) -> Result<String, String> {
    let next_ast = self.eval_next_remainder()?;
    self.get_word(&next_ast)
  }

  fn eval_list(&mut self, list: &ListType) -> Result<(), String> {
    self.stack_rem.push(VecDeque::new());
    let mut ret = Ok(());
    for item in list {
      match self.eval(&item) {
        Ok(AST::None) => {},
        Err(e) => {
          ret = Err(e);
          break;
        },
        Ok(other) => {
          ret = Err(format!("You don't say what to do with the output of {:?}", other));
          break;
        }
      }
    }
    while let Some(expr) = self.remainder().pop_front() {
      match self.eval(&expr) {
        Ok(AST::None) => {},
        Err(e) => {
          ret = Err(e);
          break;
        },
        Ok(other) => {
          ret = Err(format!("You don't say what to do with the output of {:?}", other));
          break;
        }
      }
    }
    self.stack_rem.pop();
    return ret;
  }

  fn def_function(&mut self, ast_node: &AST) -> Result<bool, String> {
    // Already started defining.
    if self.name != "" {
      let mut end = false;
      match ast_node {
        AST::ExprLine(expr_list) => {
          match expr_list.first() {
            Some(AST::Function(name, _)) => {
              if name == "TO" {
                return Err(format!("TO inside of function definition {}", self.name));
              } else if name == "END" {
                end = true;
              }
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
        if self.builtin_functions.contains_key(name) {
          return Err(format!("{} is already in use. Try a different name.", name));
        }
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

  fn local_vars(&mut self) -> &mut HashMap<String, AST> {
    self.stack_vars.last_mut().unwrap()
  }

  fn remainder(&mut self) -> &mut VecDeque<AST> {
    self.stack_rem.last_mut().unwrap()
  }

  fn eval_next_remainder(&mut self) -> Result<AST, String> {
    let next_ast = self.remainder().pop_front();
    match next_ast {
      Some(ast) => {
        return self.eval(&ast);
      },
      None => {
        return Err(format!("Need more input(s)."));
      }
    }
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
    let mut local_vars: HashMap<String, AST> = HashMap::new();
    // Setup the args as local vars.
    for arg in args {
      let var;
      match arg {
        AST::Var(_var) => { var = _var; }
        _ => { panic!("Invalid function definition, arg not a AST::Var: {}", name); }
      }
      local_vars.insert(var.clone(), self.eval_next_remainder()?);
    }
    self.stack_vars.push(local_vars);
    self.stack_rem.push(VecDeque::new());
    let mut ret = AST::None;
    // Run the lines.
    let mut err = None;
    for line in lines {
      match self.eval(&line) {
        Ok(result) => {
          ret = result;
        },
        e @ Err(_) => {
          err = Some(e);
          break;
        },
      }
      match ret {
        AST::FunctionReturn(ast) => {
          ret = *ast;
          break;
        },
        AST::None => {},
        _ => {
          err = Some(Err(format!(
              "You don't say what to do with the output of {:?}\n\
               In function {}\n\
               Statement   {:?}", ret, name, line)));
        }
      }
    }
    self.stack_vars.pop();
    self.stack_rem.pop();
    match err {
      None => { Ok(ret) },
      Some(err) => { err }
    }
  }

  fn eval(&mut self, ast_node: &AST) -> Result<AST, String> {
    // self.print_locals();
    // self.print_globals();
    // println!("{:?}", ast_node);
    // We're currently defining a function.
    if self.def_function(ast_node)? {
      return Ok(AST::None);
    }
    let mut ret = AST::None;
    match ast_node {
      AST::Function(name, expr_list) => {
        assert!(self.remainder().is_empty(), format!("{:?}", self.remainder()));
        self.remainder().extend(expr_list.clone());
        if self.builtin_functions.contains_key(name) {
          // TODO: Try to do this part without taking the closure out.
          // (*self.builtin_functions.get(name).unwrap())(self);
          let closure = self.builtin_functions.remove(name).unwrap();
          match closure(self) {
            Ok(result) => {
              self.builtin_functions.insert(name.clone(), closure);
              ret = result;
            },
            e @ Err(_) => {
              self.builtin_functions.insert(name.clone(), closure);
              return e;
            }
          }
          // ret = closure(self)?;
          // self.builtin_functions.insert(name.clone(), closure);
        } else if self.user_functions.contains_key(name) {
          ret = self.eval_user_function(name)?;
        } else {
          return Err(format!("Unknown function {:?}", name));
        }
      },
      AST::ExprLine(expr_list) => {
        self.stack_rem.push(VecDeque::new());
        self.remainder().clear();
        let mut has_remainder = false;
        for expr in expr_list {
          assert!(!has_remainder);
          let result = self.eval(expr)?;
          if self.remainder().len() > 0 {
            has_remainder = true;
          }
          if result != AST::None {
            ret = result;
            break;
          }
        }
        while let Some(expr) = self.remainder().pop_front() {
          ret = self.eval(&expr)?;
          if ret != AST::None {
            break;
          }
        }
        self.stack_rem.pop();
      },
      AST::ExprList(expr_list) => {
        // Evaluates only the first expr and returns result (if any).
        self.stack_rem.push(VecDeque::new());
        match expr_list.first() {
          Some(first_element) => {
            ret = self.eval(first_element)?;
          },
          None => {
            ret = AST::List(ListType::new());
          }
        }
        self.stack_rem.pop();
      },
      AST::Var(var_name) => {
        let mut has_local = false;
        if let Some(ast) = self.local_vars().get(var_name) {
          has_local = true;
          ret = ast.clone();
        }
        if !has_local {
          if let Some(ast) = self.vars.get(var_name) {
            ret = ast.clone();
          } else {
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
        let operand = self.get_number(box_operand)?;
        ret = AST::Float(-operand);
      },
      // TODO: Need to implement all Binary operators.
      AST::Binary(operator, left_box, right_box) => {
        let left = self.get_number(left_box)?;
        let right = self.get_number(right_box)?;
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
        // println!("{:?}", ast);
        // parser::rek_print(&ast, "".to_string());
      },
      Err(err) => {
        println!("Parsing error: {:?}", err);
        return;
      },
    }
    println!("{}", format!("Eval: {:?}", self.eval(&ast)).replace("([", "[").replace("])", "]"));
    // TODO: Occasionally try to run the following to make sure nothing is being lost from ast.
    // println!("{}", format!("Eval: {:?}", self.eval(&ast)).replace("([", "[").replace("])", "]"));
    while let Some(rem) = self.stack_rem.pop() {
      println!("Remainder: {:?}", rem);
    }
    assert!(self.stack_vars.len() > 0);
    assert_eq!(0, self.stack_vars[0].len());
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
