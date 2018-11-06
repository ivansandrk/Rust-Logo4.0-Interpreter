// *
// - perhaps a parsing subsystem with an interface that just gives available expressions
// - BoolExpr
// - NoneExpr?
// - ListExpr?
// - List type definitely
#![allow(unused_variables)]
#![allow(dead_code)]

mod lexer;

use lexer::{Token, Lexer};
use std::collections::VecDeque;

#[derive(Debug)]
pub enum ValueExpr {
  Num(i32),
  Float(f32),
  Var(String),

  Add(Box<ValueExpr>, Box<ValueExpr>),
  Subtract(Box<ValueExpr>, Box<ValueExpr>),
  Multiply(Box<ValueExpr>, Box<ValueExpr>),
  Divide(Box<ValueExpr>, Box<ValueExpr>),
}

#[derive(Debug)]
pub enum Expr {
  // Control flow commands.
  Block(Vec<Expr>),
  Repeat(u32, Box<Expr>),

  // Values.
  // Can be 180, 4.5, -10, 360/4, 4*10, 2+3, :mad, 20+100/:n
  ValueExpr(),

  // Turtle commands.
  Fd(f32),
  Bk(f32),
  Rt(f32),
  Lt(f32),

  // Canvas commands.
  Cs,

  Unknown,
  // Move { x: i32, y: i32 },
}

// #[derive(Default)]
pub struct Parser {
  // iter: std::iter::Peekable<Token>,
  tokens: Result<Vec<Token>, String>,
  // current_token: String,
  // stack: Vec<>, // vec of what?
}

impl Parser {
  pub fn new(input: &str) -> Parser {
    Parser {
      // iter: Lexer::new(input).process(),//.peekable(),
      tokens: Lexer::new(input).process(),
    }
  }

  fn process(&mut self) -> Expr {
    loop {
      // match self.iter.next() {
      //   Token::Keyword("REPEAT".as_string()) => {
      //     // Expr::Repeat(ValueExpr, Expr List)
      //     // Repeat( parse next as value expr, parse next as expr list )
      //   },
      //   _ => {},
      // }
    }
    // let ret = match self.next_token().as_str() {
    //   // TODO: Separate functions for parsing different expressions.
    //   "fd" => Expr::Fd(self.next_token_as_f32()),
    //   "bk" => Expr::Bk(self.next_token_as_f32()),
    //   "rt" => Expr::Rt(self.next_token_as_f32()),
    //   "lt" => Expr::Lt(self.next_token_as_f32()),
    //   "cs" => Expr::Cs,
    //   "repeat" => {
    //     let count = self.next_token_as_u32();
    //     let repeated_command = self.parse();
    //     Expr::Repeat(count, Box::new(repeated_command))
    //   },
    //   "[" => {
    //     // TODO: What if there's no "]" at the end?
    //     // TODO: What if we have mismatched number of "[" and "]"?
    //     let mut commands: Vec<Expr> = Vec::new();
    //     while self.peek_token() != "]" {
    //       commands.push(self.parse());
    //     }
    //     // Consume the "]".
    //     assert!(self.next_token() == "]");
    //     Expr::Block(commands)
    //   },
    //   _ => Expr::Unknown,
    // };
    // return ret;
  }

  // pub fn parse_all(&mut self) -> Vec<Expr> {
  //   let mut commands: Vec<Expr> = Vec::new();
  //   while self.has_tokens() {
  //     commands.push(self.parse());
  //   }
  //   commands
  // }
}

#[cfg(test)]
mod tests {
  use super::*;

  // TODO: Fix these tests.
  // fn test_tokenizer(input: &str, tokens: &[&str]) {
  //   let tokens: VecDeque<String> = tokens.iter().map(|&s| s.into()).collect();

  //   let mut parser = Parser::new();
  //   parser.feed(input);
  //   assert_eq!(tokens, parser.tokens);
  // }

  // #[test]
  // fn tokenizer1() {
  //   let input = "rePEat 4[fd 5 rt   90] [lt 5  fd 10 ] ";
  //   let tokens = ["repeat", "4", "[", "fd", "5" , "rt", "90", "]", "[", "lt", "5", "fd", "10", "]"];
  //   test_tokenizer(&input, &tokens);
  // }

  // #[test]
  // fn tokenizer2() {
  //   let input = " REPEAT 4[fd 5 Rt   90 [ bK  10 FD 50] ]  fd 30";
  //   let tokens = ["repeat", "4", "[", "fd", "5", "rt", "90", "[", "bk", "10", "fd", "50", "]", "]", "fd", "30"];
  //   test_tokenizer(input, &tokens);
  // }
}


// MAKE "K 0 WHILE [:K < COUNT :R1] [MAKE "K :K + 1 IF (ITEM :K :R1) = (ITEM :K :R2) THEN [MAKE "BP :BP + 1 MAKE "R1 WORD (LIJEVI :R1 :K - 1) (DESNI :R1 (COUNT :R1) - :K) MAKE "R2 WORD (LIJEVI :R2 :K - 1) (DESNI :R2 (COUNT :R2) - :K) MAKE "K :K - 1]]
/*
MAKE "K 0
WHILE [:K < COUNT :R1] [
  MAKE "K :K + 1
  IF (ITEM :K :R1) = (ITEM :K :R2) THEN [
    MAKE "BP :BP + 1
    MAKE "R1 WORD (LIJEVI :R1 :K - 1) (DESNI :R1 (COUNT :R1) - :K)
    MAKE "R2 WORD (LIJEVI :R2 :K - 1) (DESNI :R2 (COUNT :R2) - :K)
    MAKE "K :K - 1
  ]
]
*/
// Focus just on ()+*, add -/%, and then add prefix -

// Line represented as a list of expressions, or sub-list inside of a line.
// Ie. "MAKE "R1 WORD (LIJEVI :R1 :K - 1) (DESNI :R1 (COUNT :R1) - :K)" is an ExprList,
// but so is "(LIJEVI :R1 :K - 1)".
type ExprList = Vec<AST>;
type ExprLines = Vec<ExprList>;

// NumExpr, 
#[derive(Debug, Clone)]
enum AST {
  Unary(Token, Box<AST>),  // TODO: enum Number here and below.
  Binary(Token, Box<AST>, Box<AST>),
  Prefix(Token, ExprList),  // Prefix style arithmetic operations, ie. + 3 5 = 8.
  // Int(i32),  // TODO: Have both int and float num types.
  Float(f32),
  DefFunction(String, ExprList, ExprList),  // name, input args (all Var), body
  CallFunction(String),  // name
  Var(String),  // :ASD
  Word(String),  // "BIRD
  List(VecDeque<AST>), // [1 2 MAKE "A "BSD]
  ExprList(ExprList),
  // ExprList line & ExprList lines ? Because line is only evaluated once, ie. ? 1 * 2 3 -> 2 (3 is ignored).
}

fn precedence(token: &Option<Token>) -> i32 {
  match token {
    None => { -1 },
    Some(token) => {
      match token {
        Token::LParen |
        Token::LBracket |
        Token::Prefix => { -1 },
        Token::Plus |
          Token::Minus => { 0 },
        Token::Multiply |
          Token::Divide |
          Token::Modulo => { 1 },
        Token::Negation => { 2 }
        _ => { panic!("Invalid token for precedence {:?}", token) }
      }
    }
  }
}

// Logo has separate function and variable definitions.  It doesn't like builtin names
// for function names.
fn foo(queue: &mut VecDeque<Token>, last_token: &Option<Token>) -> Result<AST, String> {
println!("start {:?} {:?}", queue, last_token);
  let mut left;
  if queue.front() == Some(&Token::Whitespace) {
    queue.pop_front();
  }
  let token = queue.pop_front();
  match token {
    Some(Token::Num(i)) => {
      left = AST::Float(i as f32);
    },
    Some(Token::Float(f)) => {
      left = AST::Float(f);
    },
    Some(Token::Function(name)) => {
      // TODO: Special Function "TO FOO :A :B\n...\nEND" -> have a special function for parsing it.
      return Ok(AST::CallFunction(name));
    },
    Some(Token::Var(var)) => {
      left = AST::Var(var);
    },
    Some(Token::Word(word)) => {
      left = AST::Word(word);
    },
    Some(Token::LParen) => {
      let mut expr_list = ExprList::new();
      while queue.len() > 0 && queue.front() != Some(&Token::RParen) {
        expr_list.push(foo(queue, &token)?);
      }
      left = AST::ExprList(expr_list);
      // RParen should be next, which is consumed by this LParen.
      if queue.pop_front() != Some(Token::RParen) {
        return Err(format!("unmatched left paren operand {:?} last_token {:?}", left, last_token));
      }
    },
    Some(Token::LBracket) => {
      let mut list = VecDeque::new();
      while queue.front().is_some() && queue.front() != Some(&Token::RBracket) {
        list.push_back(foo(queue, &token)?);
      }
      // RBracket is next, and it's consumed by this LBracket.
      if queue.pop_front() != Some(Token::RBracket) {
        return Err(format!("unmatched left bracket list {:?} last_token {:?}", list, last_token));
      }
      left = AST::List(list);
    },
    Some(Token::Minus) if queue.front() != Some(&Token::Whitespace) => {
      match queue.front() {
        Some(&Token::Whitespace) => {
          // TODO: Remove at some point.
          panic!("Should not be here!");
        },
        Some(&Token::Num(_)) | Some(&Token::LParen) => {
          let operand = foo(queue, &Some(Token::Negation))?;
          left = AST::Unary(Token::Negation, Box::new(operand));
        },
        _ => {
          return Err(format!("bad token after Minus queue {:?}", queue));
        }
      }
    },
    Some(Token::Plus) |
    Some(Token::Minus) |
    Some(Token::Multiply) |
    Some(Token::Divide) |
    Some(Token::Modulo) => {
      let mut expr_list = ExprList::new();
      // TODO: Is this parsing until LineEnd dangerous?
      while queue.len() > 0 && queue.front() != Some(&Token::Line) {
        expr_list.push(foo(queue, &Some(Token::Prefix))?);
      }
      left = AST::Prefix(token.unwrap(), expr_list);
    },
    _ => {
      return Err(format!("missing operand or not an operand {:?} last_token {:?} queue {:?}", token, last_token, queue));
    }
  }
  // TODO: This could probably go away if I had parse_left and parse_right functions?
  if last_token == &Some(Token::Negation) {
    return Ok(left);
  }
  loop {
    let token = queue.front().cloned();
    match token {
      // Left only tokens.
      None |  // Hit end, propagate left to parents right.
      Some(Token::Line) |
      Some(Token::Num(_)) |
      Some(Token::Float(_)) |
      Some(Token::Function(_)) |
      Some(Token::Var(_)) |
      Some(Token::Word(_)) |
      Some(Token::LParen) |
      Some(Token::LBracket) => {
        return Ok(left);
      }
      Some(Token::Whitespace) => {
        if queue.len() >= 3 && queue[0] == Token::Whitespace &&
           queue[1] == Token::Minus && queue[2] != Token::Whitespace {
          // Next one is unary minus, return here.
          return Ok(left);
        } else {
          // Eat whitespace and continue onto next iteration of loop.
          queue.pop_front();
        }
      },
      Some(Token::Plus) |
      Some(Token::Minus) |
      Some(Token::Multiply) |
      Some(Token::Divide) |
      Some(Token::Modulo) => {
        // Give the left operand back to the previous operator if the precedence is higher
        // or equal.
        if precedence(last_token) >= precedence(&token) {
          return Ok(left);
        }
        // let token = queue.pop_front().unwrap();
        queue.pop_front();
        let right = foo(queue, &token)?;
        left = AST::Binary(token.unwrap(), Box::new(left), Box::new(right));
      },
      Some(Token::RParen) => {
        // RParen propagates back until the last LParen which consumes it.
        if last_token.is_none() {
          return Err(format!("unmatched right paren queue {:?}", queue));
        }
        return Ok(left);
      },
      Some(Token::RBracket) => {
        // RBracket goes back to LParen.
        if last_token.is_none() {
          return Err(format!("unmatched right bracket queue {:?} last_token {:?}", queue, last_token));
        }
        return Ok(left);
      },
      _ => {
        return Err(format!("Unknown token {:?}", token));
      },
    }
  }
}

fn parse_line(queue: &mut VecDeque<Token>) -> Result<AST, String> {
  let mut expr_list = ExprList::new();
  while queue.front().is_some() &&
        queue.front() != Some(&Token::Line) {
println!("{:?}", expr_list);
    expr_list.push(foo(queue, &None)?);
  }
  if queue.front() == Some(&Token::Line) {
    queue.pop_front();
  }
  return Ok(AST::ExprList(expr_list));
}

fn rek_print(item: &AST, prefix: String) {
  let len = prefix.len();
  if prefix.len() >= 2 {
    print!("{}+- ", &prefix[..len-2]);
  }
  match item {
    AST::Float(num) => {
      println!("{:?}", num);
    },
    AST::Var(var) => {
      println!(":{}", var);
    },
    AST::Word(word) => {
      println!("\"{}", word);
    },
    AST::CallFunction(name) => {
      println!("{}", name);
    },
    AST::Unary(operator, operand) => {
      println!("{:?}", operator);
      rek_print(operand, prefix.clone() + "  ");
    },
    AST::Binary(operator, left_operand, right_operand) => {
      println!("{:?}", operator);
      rek_print(left_operand, prefix.clone() + "| ");
      rek_print(right_operand, prefix.clone() + "  ");
    },
    AST::Prefix(token, expr_list) => {
      println!("Prefix {:?}", token);
      rek_print(&AST::ExprList(expr_list.clone().to_vec()), prefix.clone() + "  ");
    },
    AST::List(list) => {
      println!("{:?}", "LIST");
      for (i, element) in list.iter().enumerate() {
        rek_print(element, prefix.clone() + if i < list.len()-1 { "| " } else { "  " });
      }
    },
    AST::ExprList(expr_list) => {
      println!("EXPRESSION LIST");
      for (i, element) in expr_list.iter().enumerate() {
        rek_print(element, prefix.clone() + if i < expr_list.len()-1 { "| " } else { "  " });
      }
    },
    _ => {
      println!("Not implemented in rek_print {:?}", item);
    }
  }
}

fn pratt_parse_debug(input: &str) {
  println!("{:?}", input);
  let tokens;
  match Lexer::new(input).process() {
    Ok(val) => tokens = val,
    Err(err) => { println!("Tokenizing error: {:?}", err); return; }
  }
  let mut queue: VecDeque<Token> = tokens.into_iter().collect();
  println!("{:?}", queue);
  match parse_line(&mut queue) {
    Ok(val) => {
      println!("{:?}", val);
      rek_print(&val, "".to_string());
    },
    Err(err) => {
      println!("Parsing error: {:?}", err);
    },
  }
}

fn main() {
  // let input = "(1 + 5) % 3 * 3 - 4 / 1";
  // 1 2 3 4 5 * + * 6 7 8 + * + 9 * +
  // 1 + (2 * (3 + 4 * 5) + 6 * (7 + 8)) * 9
  // 1 + (2 * (3 + 4 * -5) + -6 * -(-7 + -8)) * 9
  // let input = "1 + (2 * (3 + 4 * 5) + 6 * (7 + 8)) * 9";
  // let input = "1 + (2 * (3 + 4 * 5) + 6 + 7) * 8";
  loop {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    pratt_parse_debug(input.trim());
  }
}

// fn main() {
//   let str1 = " rePEat 4[fd 5 rt   90] [lt 5  fd 10 ] ";
//   let str2 = " REPEAT 4[fd 5 Rt   90 [ bK  10 FD 50] ] fd 10";
//   let str3 = "fd "; // TODO: Crashes.

//   // let mut input = String::new();
//   // std::io::stdin().read_line(&mut input).unwrap();
//   println!("{:?}", lexer::Lexer::new(str2).process());
//   // println!("{:?}", Parser::new(str2).process());
// }
