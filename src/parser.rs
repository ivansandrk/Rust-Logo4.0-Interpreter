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

  fn test_tokenizer(input: &str, tokens: &[&str]) {
    let tokens: VecDeque<String> = tokens.iter().map(|&s| s.into()).collect();

    let mut parser = Parser::new();
    parser.feed(input);
    assert_eq!(tokens, parser.tokens);
  }

  #[test]
  fn tokenizer1() {
    let input = "rePEat 4[fd 5 rt   90] [lt 5  fd 10 ] ";
    let tokens = ["repeat", "4", "[", "fd", "5" , "rt", "90", "]", "[", "lt", "5", "fd", "10", "]"];
    test_tokenizer(&input, &tokens);
  }

  #[test]
  fn tokenizer2() {
    let input = " REPEAT 4[fd 5 Rt   90 [ bK  10 FD 50] ]  fd 30";
    let tokens = ["repeat", "4", "[", "fd", "5", "rt", "90", "[", "bk", "10", "fd", "50", "]", "]", "fd", "30"];
    test_tokenizer(input, &tokens);
  }
}

// Shunting-yard algorithm (c) Edsger Dijkstra
// Parse stuff like 1 + 5 % 3 * 3 - 4

// while there are tokens to be read:
//     read a token.
//     if the token is a number, then:
//         push it to the output queue.
//     if the token is an operator, then:
//         while (operator_stack.peek().precedence() >= current_op.precedence() &&
//                op_stack.peek() != left_paren):
//             output_queue.push(operator_stack.pop())
//         operator_stack.push(current_op)
//     if the token is a left bracket (i.e. "("), then:
//         push it onto the operator stack.
//     if the token is a right bracket (i.e. ")"), then:
//         while the operator at the top of the operator stack is not a left bracket:
//             pop the operator from the operator stack onto the output queue.
//         pop the left bracket from the stack.
//         /* if the stack runs out without finding a left bracket, then there are mismatched parentheses. */
// if there are no more tokens to read:
//     while there are still operator tokens on the stack:
//         /* if the operator token on the top of the stack is a bracket, then there are mismatched parentheses. */
//         pop the operator from the operator stack onto the output queue.
// exit.

fn precedence(token: &Token) -> u32 {
  match token {
    Token::Plus |
      Token::Minus => { 0 },
    Token::Multiply |
      Token::Divide |
      Token::Modulo => { 1 },
    _ => { panic!("Invalid token for precedence!"); }
  }
}

fn shunting_yard_algorithm(input: &str) -> (VecDeque<Token>, Vec<Token>) {
  let tokens = Lexer::new(input).process().unwrap();
  println!("{:?}", tokens);
  let mut output_queue: VecDeque<Token> = VecDeque::new();
  let mut operator_stack: Vec<Token> = Vec::new();
  for token in tokens {
    match token {
      Token::Num(_) | Token::Float(_) => {
        output_queue.push_back(token);
      },
      Token::Plus | Token::Minus | Token::Multiply | Token::Divide | Token::Modulo => {
        while operator_stack.last() != Some(&Token::OpenParen) &&
              precedence(operator_stack.last().unwrap()) >= precedence(&token) {
          output_queue.push_back(operator_stack.pop().unwrap());
        }
        // while operator_stack.len() > 0 {
        //   let top = operator_stack.last().unwrap().clone();
        //   if *top != Token::OpenParen && precedence(top) >= precedence(&token) {
        //     output_queue.push_back(operator_stack.pop().unwrap());
        //   } else {
        //     break;
        //   }
        // }
        // while operator_stack.len() > 0 {
        //   let top = operator_stack.last().unwrap().clone();
        //   if top != Token::OpenParen && precedence(&top) >= precedence(&token) {
        //     output_queue.push_back(operator_stack.pop().unwrap());
        //   } else {
        //     break;
        //   }
        // }
        operator_stack.push(token);
      },
      Token::OpenParen => {
        operator_stack.push(token);
      },
      Token::CloseParen => {
        while operator_stack.last() != Some(&Token::OpenParen){
          output_queue.push_back(operator_stack.pop().unwrap());
        }
        // TODO: pop can panic here if the expression is malformed
        // (no OpenParen at top of stack)
        assert_eq!(Token::OpenParen, operator_stack.pop().unwrap());
      },
      _ => { panic!("Unknown token!"); }
    }
  }
  while let Some(op) = operator_stack.pop() {
    output_queue.push_back(op);
  }
  (output_queue, operator_stack)
}

// Focus just on ()+*

#[derive(Debug)]
enum FB {
  Num(i32),
  Plus(Box<FB>, Box<FB>),
  Mult(Box<FB>, Box<FB>),
}

// Sometimes give back left (if last_op precedence >= current op precedence)
fn foo(queue: &mut VecDeque<Token>, last_op: Option<Token>) -> FB {
  let left;
  let token = queue.pop_front();
  match token {
    Some(Token::Num(i)) => {
      left = FB::Num(i);
    },
    Some(Token::OpenParen) => {
      // (2 * 3 + 4) * 5 -> processes left upto closing paren
      left = foo(queue, token);
    },
    // TODO: Merge these two.
    None => {
      panic!("missing operand after last_op {:?}", last_op);
    },
    _ => {
      panic!("Left must be an operand, got {:?}, last_op {:?}", token, last_op);
    }
  }
  let right;
  let token = queue.pop_front();
  match token {
    None => {
      // Hit end, propagate left to parents right.
      return left;
    },
    Some(Token::Num(_)) | Some(Token::OpenParen) => {
      panic!("operand {:?} cannot follow left operand", queue.front().unwrap());
    }
    Some(Token::Plus) => {
      // Plus always gives back left (upto paren).
      if last_op.is_some() && last_op != Some(Token::OpenParen) {
        return left;
      }
      right = foo(queue, token);
      return FB::Plus(Box::new(left), Box::new(right));
    },
    Some(Token::Multiply) => {
      // Multiply takes left.
      right = foo(queue, token);
      return FB::Mult(Box::new(left), Box::new(right));
    },
    Some(Token::CloseParen) => {
      return left;
    },
    _ => {
      panic!("Unknown token {:?}", queue.front().unwrap());
    },
  }
  // println!("{:?}", queue.front());
  // FB::Num(0)
}

fn main() {
  // let input = "(1 + 5) % 3 * 3 - 4 / 1";
  // 1 2 3 4 5 * + * 6 7 8 + * + 9 * +
  // 1 + (2 * (3 + 4 * 5) + 6 * (7 + 8)) * 9
  // let input = "1 + (2 * (3 + 4 * 5) + 6 * (7 + 8)) * 9";
  // let input = "1 + (2 * (3 + 4 * 5) + 6 + 7) * 8";
  let input = "1 + 2 + 3";
  let tokens = Lexer::new(input).process().unwrap();
  let mut queue: VecDeque<Token> = tokens.into_iter().collect();

  // println!("{:?}", shunting_yard_algorithm(input));
  println!("{:?}", foo(&mut queue, None));
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
