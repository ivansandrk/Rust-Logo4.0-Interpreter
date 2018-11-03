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

// NumExpr, 
#[derive(Debug)]
enum AST {
  Unary(Token, Box<AST>),
  Binary(Token, Box<AST>, Box<AST>),
  Leaf(i32),
}

fn precedence(token: &Option<Token>) -> i32 {
  match token {
    None => { -1 },
    Some(token) => {
      match token {
        Token::LParen => { -1 },
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
  let mut left;
  let token = queue.pop_front();
  match token {
    Some(Token::Num(i)) => {
      left = AST::Leaf(i);
    },
    Some(Token::Minus) => {
      let operand = foo(queue, &token)?;
      left = AST::Unary(token.unwrap(), Box::new(operand));
    },
    Some(Token::LParen) => {
      left = foo(queue, &token)?;
      // RParen should be next, which is consumed by this LParen.
      if queue.pop_front() != Some(Token::RParen) {
        return Err(format!("unmatched left paren operand {:?} last_token {:?}", left, last_token));
      }
    },
    _ => {
      return Err(format!("missing operand or not an operand {:?} after last_token {:?}", token, last_token));
    }
  }
  loop {
    let token = queue.front().cloned();
    match token {
      None => {
        // Hit end, propagate left to parents right.
        return Ok(left);
      },
      Some(Token::Plus) |
      Some(Token::Minus) |
      Some(Token::Multiply) |
      Some(Token::Divide) |
      Some(Token::Modulo) => {
        // Give the left operand back to the previous operator if the precedence is higher
        // or equal.
        // TODO: Make sure A && B evaluation order in Rust is A first, only then lazy B.
        // if last_token != &Some(Token::LParen) &&
        //     precedence(last_token)? >= precedence(&token)? {
        // let operator = token_to_operator(&token, false);
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
      Some(e @ Token::Num(_)) | Some(e @ Token::LParen) => {
        return Err(format!("operand {:?} cannot follow left operand", e));
      }
      _ => {
        return Err(format!("Unknown token {:?}", token));
      },
    }
  }
}

fn rek_print(item: &AST, prefix: String) {
  let len = prefix.len();
  match item {
    AST::Leaf(num) => {
      println!("{}+-{:?}", &prefix[..len-2], num);
    },
    AST::Unary(operator, operand) => {
      println!("{}+-{:?}", &prefix[..len-2], operator);
      rek_print(operand, prefix.clone() + "  ");
    },
    AST::Binary(operator, left_operand, right_operand) => {
      println!("{}+-{:?}", &prefix[..len-2], operator);
      rek_print(left_operand, prefix.clone() + "| ");
      rek_print(right_operand, prefix.clone() + "  ");
    },
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
  match foo(&mut queue, &None) {
    Ok(val) => {
      println!("{:?}", val);
      rek_print(&val, "  ".to_string());
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
