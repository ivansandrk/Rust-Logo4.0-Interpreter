#![allow(dead_code)]

// *** TODOs
// Define WordType
// user functions should store args as Strings, not AST::Var(String)
// Logo has separate function and variable definitions.  It doesn't like builtin names for function names.

// mod lexer;
// mod parser;
use lexer;
use parser;

use std;

use std::collections::HashMap;
use std::collections::VecDeque;
use std::mem;
use parser::{AST, ListType, WordType, NumType};
use lexer::Token;

pub trait Graphics {
  // Draws a line from p1 to p2 using window center as origin point (0, 0), and
  // having the x-axis grow left->right, and y-axis down->up.
  fn line(&mut self, p1: (f32, f32), p2: (f32, f32));

  // Clears the screen.
  fn clearscreen(&mut self);
}

#[derive(Default)]
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

impl NullGraphics {
  fn new() -> Box<Graphics> {
    Box::new(NullGraphics {
      ..Default::default()
    })
  }
}

pub struct Turtle {
  heading: f32, // 0 .. 359 degrees
  x: f32,
  y: f32,
  graphics: Box<Graphics>,
}

impl Default for Turtle {
  fn default() -> Turtle {
    Turtle {
      heading: 0.0,
      x: 0.0,
      y: 0.0,
      graphics: NullGraphics::new(),
    }
  }
}

impl Turtle {
  pub fn new() -> Turtle {
    Turtle {
      ..Default::default()
    }
  }

  fn fd(&mut self, val: f32) {
    let phi = (self.heading + 90.0) * std::f32::consts::PI / 180.0;
    let new_x = self.x + val * phi.cos();
    let new_y = self.y + val * phi.sin();
    self.graphics.line((self.x, self.y), (new_x, new_y));
    self.x = new_x;
    self.y = new_y;
  }

  fn bk(&mut self, val: f32) {
    self.fd(-val);
  }

  fn lt(&mut self, val: f32) {
    // TODO: Clamp the heading perhaps to only [0, 360).
    self.heading += val;
  }

  fn rt(&mut self, val: f32) {
    self.lt(-val);
  }

  fn clearscreen(&mut self) {
    self.graphics.clearscreen();
    self.x = 0.0;
    self.y = 0.0;
    self.heading = 0.0;
  }
}

type ArgsType = Vec<String>;
type BuiltinFunctionType = Fn(&mut Evaluator) -> Result<AST, String>;

#[derive(Default)]
pub struct Evaluator {
  parser: parser::Parser,
  turtle: Turtle,

  // Global variables.
  vars: HashMap<String, AST>,

  // Function local variables.
  stack_vars: Vec<HashMap<String, AST>>,
  // Current expression, (iterator) list.
  stack_expr: Vec<ListType>,

  builtin_functions: HashMap<String, std::rc::Rc<BuiltinFunctionType>>,
  user_functions: HashMap<String, (ArgsType, ListType)>,

  // Name, args, and lines of the currently defined function.
  name: String,
  args: ArgsType,
  lines: ListType,
}

impl Evaluator {
  pub fn new() -> Self {
    let mut evaluator = Evaluator {
      ..Default::default()
    };
    evaluator.stack_vars.push(HashMap::new());
    evaluator.define_builtins();
    evaluator
  }

  pub fn set_graphics(&mut self, graphics: Box<Graphics>) {
    self.turtle.graphics = graphics;
  }

  fn define_builtins(&mut self) {
    #![allow(unused_parens)]
    macro_rules! add_builtin {
      ($name:ident, $closure:tt) => {
        self.builtin_functions.insert(
            stringify!($name).to_string(),
            std::rc::Rc::new($closure));
      };
      ($name1:ident, $name2:ident, $closure:tt) => {
        let rc = std::rc::Rc::new($closure);
        self.builtin_functions.insert(stringify!($name1).to_string(), rc.clone());
        self.builtin_functions.insert(stringify!($name2).to_string(), rc.clone());
      };
    }
    add_builtin!(OP, OUTPUT, (|evaluator: &mut Evaluator| {
      Ok(AST::FunctionReturn(Box::new(evaluator.eval_next_expr()?)))
    }));
    add_builtin!(POPS, (|evaluator| {
      for (name, (args, lines)) in evaluator.user_functions.iter() {
        print!("TO {}", name);
        for arg in args {
          print!(" :{}", arg);
        }
        println!();
        for line in lines {
          println!("{:?}", line);
        }
        println!("END");
      }
      Ok(AST::None)
    }));
    add_builtin!(PONS, (|evaluator| {
      evaluator.print_locals();
      evaluator.print_globals();
      Ok(AST::None)
    }));
    add_builtin!(MAKE, (|evaluator| {
      let var = evaluator.get_next_word()?;
      let expr = evaluator.eval_next_expr()?;
      if evaluator.local_vars().contains_key(&var) {
        evaluator.local_vars().insert(var, expr);
      } else {
        evaluator.vars.insert(var, expr);
      }
      Ok(AST::None)
    }));
    add_builtin!(REPEAT, (|evaluator| {
      let repeat = evaluator.get_next_number()?;
      let list = evaluator.get_next_list()?;
      for _ in 0 .. repeat as i32 {
        evaluator.eval_list(&list)?;
      }
      Ok(AST::None)
    }));
    add_builtin!(FD, FORWARD, (|evaluator: &mut Evaluator| {
      let num = evaluator.get_next_number()?;
      evaluator.turtle.fd(num);
      Ok(AST::None)
    }));
    add_builtin!(BK, BACK, (|evaluator: &mut Evaluator| {
      let num = evaluator.get_next_number()?;
      evaluator.turtle.bk(num);
      Ok(AST::None)
    }));
    add_builtin!(RT, RIGHT, (|evaluator: &mut Evaluator| {
      let num = evaluator.get_next_number()?;
      evaluator.turtle.rt(num);
      Ok(AST::None)
    }));
    add_builtin!(LT, LEFT, (|evaluator: &mut Evaluator| {
      let num = evaluator.get_next_number()?;
      evaluator.turtle.lt(num);
      Ok(AST::None)
    }));
    add_builtin!(CS, CLEARSCREEN, (|evaluator: &mut Evaluator| {
      evaluator.turtle.clearscreen();
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
  fn get_number(&mut self, ast_node: &AST) -> Result<NumType, String> {
    match self.eval(ast_node)? {
      AST::Num(num) => { Ok(num) },
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

  fn get_next_number(&mut self) -> Result<NumType, String> {
    let next_ast = self.eval_next_expr()?;
    self.get_number(&next_ast)
  }

  fn get_next_list(&mut self) -> Result<ListType, String> {
    let next_ast = self.eval_next_expr()?;
    self.get_list(&next_ast)
  }

  fn get_next_word(&mut self) -> Result<String, String> {
    let next_ast = self.eval_next_expr()?;
    self.get_word(&next_ast)
  }

  // TODO: REPEAT 4 [4] complains about what to do with 4, while EVAL [4] just returns [4].
  // EVAL [1 2 FD 50 3] should return [1 2 3]
  fn eval_list(&mut self, list: &ListType) -> Result<(), String> {
    self.stack_expr.push(list.clone());
    let mut ret = Ok(());
    while let Some(expr) = self.current_expr_list().pop_front() {
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
    self.stack_expr.pop();
    return ret;
  }

  fn define_user_function(&mut self, ast_node: &AST) -> Result<bool, String> {
    // Already started defining.
    if self.name != "" {
      if let AST::ExprLine(expr_list) = ast_node {
        if let Some(AST::Function(name)) = expr_list.front() {
          if name == "TO" {
            return Err(format!("TO inside of function definition {}", self.name));
          } else if name == "END" {
            // End of function definition, save it.
            let name = mem::replace(&mut self.name, String::new());
            let args = mem::replace(&mut self.args, ArgsType::new());
            let lines = mem::replace(&mut self.lines, ListType::new());
            self.user_functions.insert(name, (args, lines));
          } else {
            // Collect the line.
            self.lines.push_back(ast_node.clone());
          }
        }
      }
      return Ok(true);
    }
    if ast_node != &AST::Function("TO".to_string()) {
      return Ok(false);
    }
    match self.current_expr_list().pop_front() {
      Some(AST::Function(name)) => {
        if self.builtin_functions.contains_key(&name) {
          return Err(format!("{} is already in use. Try a different name.", name));
        }
        let mut args = ArgsType::new();
        while let Some(arg) = self.current_expr_list().pop_front() {
          if let AST::Var(arg) = arg {
            args.push(arg);
          } else {
            return Err(format!("The procedure TO does not like {:?} as input.", arg));
          }
        }
        self.name = name;
        self.args = args;
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

  fn current_expr_list(&mut self) -> &mut VecDeque<AST> {
    self.stack_expr.last_mut().unwrap()
  }

  fn eval_next_expr(&mut self) -> Result<AST, String> {
    let next_ast = self.current_expr_list().pop_front();
    match next_ast {
      Some(ast) => {
        return self.eval(&ast);
      },
      None => {
        return Err(format!("Need more input(s)."));
      }
    }
  }

  fn eval_builtin_function(&mut self, name: &str) -> Result<AST, String> {
    let closure = self.builtin_functions.get(name).unwrap().clone();
    return closure(self);
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
      local_vars.insert(arg.clone(), self.eval_next_expr()?);
    }
    self.stack_vars.push(local_vars);
    // TODO: Probably don't need this push here?
    self.stack_expr.push(VecDeque::new());
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
    self.stack_expr.pop();
    match err {
      None => { Ok(ret) },
      Some(err) => { err }
    }
  }

  fn eval(&mut self, ast_node: &AST) -> Result<AST, String> {
    // self.print_locals();
    // self.print_globals();
    // println!("{:?}", ast_node);
    if self.define_user_function(ast_node)? {
      // We're currently defining a function.
      return Ok(AST::None);
    }
    let mut ret = AST::None;
    match ast_node {
      AST::Function(name) => {
        if self.builtin_functions.contains_key(name) {
          ret = self.eval_builtin_function(name)?;
        } else if self.user_functions.contains_key(name) {
          ret = self.eval_user_function(name)?;
        } else {
          return Err(format!("Unknown function {:?}", name));
        }
      },
      // TODO: Type that pushes during construction, and pops during destruction.
      AST::ExprLine(expr_list) => {
        self.stack_expr.push(expr_list.clone());
        while let Some(expr) = self.current_expr_list().pop_front() {
          ret = self.eval(&expr)?;
          if ret != AST::None {
            break;
          }
        }
        self.stack_expr.pop();
      },
      // TODO: Builtin functions behave differently if they open Parens.
      AST::Parens(expr_list) => {
        // Evaluates only the first expr and returns result (if any).  If the expression list is
        // empty, returns the empty list.
        if expr_list.is_empty() {
          ret = AST::List(ListType::new());
        } else {
          self.stack_expr.push(expr_list.clone());
          let next_expr = self.current_expr_list().pop_front().unwrap();
          ret = self.eval(&next_expr)?;
          self.stack_expr.pop();
        }
      },
      AST::Var(var_name) => {
        if let Some(ast) = self.stack_vars.last().unwrap().get(var_name) {
          ret = ast.clone();
        } else if let Some(ast) = self.vars.get(var_name) {
          ret = ast.clone();
        } else {
          return Err(format!(":{} is not a Logo name.", var_name));
        }
      },
      AST::Num(num) => {
        ret = AST::Num(*num);
      },
      AST::List(list) => {
        ret = AST::List(list.clone());
      },
      AST::Word(string) => {
        ret = AST::Word(string.clone());
      },
      AST::Negation(box_operand) => {
        let operand = self.get_number(box_operand)?;
        ret = AST::Num(-operand);
      },
      AST::Comparison(operator, left_box, right_box) => {
        let left = self.eval(left_box)?;
        let right = self.eval(right_box)?;
        let mut one_word = false;
        for word in vec![&left, &right] {
          match word {
            AST::Num(_) => {},
            AST::Word(_) => { one_word = true; },
            _ => { return Err(format!("The comparison procedure needs a name or number.")); }
          }
        }
        let result;
        if !one_word {
          let left = self.get_number(&left);
          let right = self.get_number(&right);
          result = match operator {
            Token::Less => { left < right },
            Token::LessEq => { left <= right },
            Token::Greater => { left > right },
            Token::GreaterEq => { left >= right },
            Token::Equal => { left == right },
            _ => {
              panic!("Unknown comparison operator {:?}", operator);
            }
          };
        } else {
          let left = match left {
            AST::Num(num) => {
              format!("{}", num)
            },
            AST::Word(string) => { string },
            _ => { unreachable!() }
          };
          let right = match right {
            AST::Num(num) => {
              format!("{}", num)
            },
            AST::Word(string) => { string },
            _ => { unreachable!() }
          };
          result = match operator {
            Token::Less => { left < right },
            Token::LessEq => { left <= right },
            Token::Greater => { left > right },
            Token::GreaterEq => { left >= right },
            Token::Equal => { left == right },
            _ => {
              panic!("Unknown comparison operator {:?}", operator);
            }
          };
        }
        ret = AST::Word( (if result { "TRUE" } else { "FALSE" }).to_string() );
      },
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
        ret = AST::Num(result);
      },
      AST::Nary(operator, expr_list_orig) => {
        let mut expr_list = expr_list_orig.clone();
        let mut result;
        if operator == &Token::Plus {
          result = 0.0;
        } else if operator == &Token::Multiply {
          result = 1.0;
        } else {
          panic!("Unknown prefix operator {:?}", operator);
        }
        while let Some(operand) = expr_list.pop_front() {
          let operand = self.get_number(&operand)?;
          if operator == &Token::Plus {
            result += operand;
          } else if operator == &Token::Multiply {
            result *= operand;
          } else {
          }
        }
        ret = AST::Num(result);
      },
      _x => {
        println!("Unimplemented eval AST {:?}", _x);
      }
    }
    return Ok(ret);
  }

  pub fn feed(&mut self, input: &str) {
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
    let result = self.eval(&ast);
    if result != Ok(AST::None) {
      println!("{}", format!("Eval: {:?}", self.eval(&ast)).replace("([", "[").replace("])", "]"));
      // TODO: Occasionally try to run the following to make sure nothing is being lost from ast.
      // println!("{}", format!("Eval: {:?}", self.eval(&ast)).replace("([", "[").replace("])", "]"));
    }
    while let Some(rem) = self.stack_expr.pop() {
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
