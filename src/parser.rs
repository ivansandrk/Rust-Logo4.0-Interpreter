// *
// - perhaps a parsing subsystem with an interface that just gives available expressions
// - BoolExpr
// - NoneExpr?
// - ListExpr?
// - List type definitely
// - Logo has separate function and variable definitions.  It doesn't like builtin names for function names.
// - TODO: Prefix operators should take only the first two following expressions, not everything
//   (unless they open a paren, ie (+ 1 2 3 4) will give 10, while + 1 2 3 4 will give just 3)
// - TODO: Drop use of VecDeque?

// #![allow(unused_variables)]
// #![allow(dead_code)]

// use lexer;
mod lexer;

use lexer::Token;
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

// Line represented as a list of expressions, or sub-list inside of a line.
// Ie. "MAKE "R1 WORD (LIJEVI :R1 :K - 1) (DESNI :R1 (COUNT :R1) - :K)" is an ExprList,
// but so is "(LIJEVI :R1 :K - 1)".
type ExprList = Vec<AST>;
type ExprLines = Vec<ExprList>;
type ListType = VecDeque<AST>;

// NumExpr, TODO: Remove Clone?
#[derive(Debug, Clone, PartialEq)]
pub enum AST {
  Unary(Token, Box<AST>),  // TODO: enum Number here and below.
  Binary(Token, Box<AST>, Box<AST>),
  Prefix(Token, ExprList),  // Prefix style arithmetic operations, ie. + 3 5 = 8.
  // Int(i32),  // TODO: Have both int and float num types.
  Float(f32),
  DefFunction(String, ExprList, ExprLines),  // name, input args (all Var), body
  Function(String, ExprList),  // name, arguments and rest
  Var(String),  // :ASD
  Word(String),  // "BIRD
  List(ListType), // [1 2 MAKE "A "BSD]
  ExprList(ExprList),  // Exprs inside of parens
  ExprLine(ExprList),  // Line of exprs
  // ExprList line & ExprList lines ? Because line is only evaluated once, ie. ? 1 * 2 3 -> 2 (3 is ignored).
  // Parser returns None in case it doesn't have a fully parsed expression.  Ie. a function
  // definition, or a LineCont might cause the expression to span multiple input lines.
  None,
}

fn precedence(token: &Option<Token>) -> i32 {
  match token {
    None => { -1 },
    Some(token) => {
      match token {
        Token::LParen |
        Token::LBracket |
        Token::Prefix |
        Token::Function(_) => { -1 },
        Token::Less |
        Token::LessEq |
        Token::Greater |
        Token::GreaterEq |
        Token::Equal => { 0 },
        Token::Plus |
          Token::Minus => { 1 },
        Token::Multiply |
          Token::Divide |
          Token::Modulo => { 2 },
        Token::Negation => { 3 }
        _ => { panic!("Invalid token for precedence {:?}", token) }
      }
    }
  }
}

fn parse_left(queue: &mut VecDeque<Token>, last_token: &Option<Token>) -> Result<AST, String> {
  let left;
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
      let mut expr_list = ExprList::new(); // 3 * (2 * ADD 1 2) + 5
      while queue.len() > 0 && queue.front() != Some(&Token::LineEnd) &&
                               queue.front() != Some(&Token::RParen) &&
                               queue.front() != Some(&Token::RBracket) {
        expr_list.push(parse_one(queue, &Some(Token::Function("".to_string())))?);
      }
      left = AST::Function(name, expr_list);
    },
    Some(Token::Var(var)) => {
      left = AST::Var(var);
    },
    Some(Token::Word(word)) => {
      left = AST::Word(word);
    },
    // TODO: (+ 1 2 3) needs to be added as well.
    Some(Token::LParen) => {
      let mut expr_list = ExprList::new();
      while queue.len() > 0 && queue.front() != Some(&Token::LineEnd) &&
                               queue.front() != Some(&Token::RParen) &&
                               queue.front() != Some(&Token::RBracket) {
        expr_list.push(parse_one(queue, &token)?);
      }
      // RParen should be next, which is consumed by this LParen.
      if queue.pop_front() != Some(Token::RParen) {
        return Err(format!("unmatched left paren operand {:?} last_token {:?}", expr_list, last_token));
      }
      left = AST::ExprList(expr_list);
    },
    Some(Token::LBracket) => {
      let mut list = ListType::new();
      while queue.len() > 0 && queue.front() != Some(&Token::LineEnd) &&
                               queue.front() != Some(&Token::RParen) &&
                               queue.front() != Some(&Token::RBracket) {
        list.push_back(parse_one(queue, &token)?);
      }
      // RBracket is next, and it's consumed by this LBracket.
      if queue.pop_front() != Some(Token::RBracket) {
        return Err(format!("unmatched left bracket list {:?} last_token {:?}", list, last_token));
      }
      left = AST::List(list);
    },
    Some(Token::Minus) if queue.front() != Some(&Token::Whitespace) => {
      match queue.front() {
        Some(&Token::Num(_)) | Some(&Token::LParen) => {
          let operand = parse_left(queue, &Some(Token::Negation))?;
          left = AST::Unary(Token::Negation, Box::new(operand));
        },
        _ => {
          return Err(format!("bad token after Minus queue {:?}", queue));
        }
      }
    },
    // Prefix-style arithmetic or comparison operators.
    Some(Token::Plus) | Some(Token::Minus) | Some(Token::Multiply) | Some(Token::Divide) |
    Some(Token::Modulo) | Some(Token::Less) | Some(Token::LessEq) | Some(Token::Greater) |
    Some(Token::GreaterEq) | Some(Token::Equal) => {
      let mut expr_list = ExprList::new();
      while queue.len() > 0 && queue.front() != Some(&Token::LineEnd) &&
                               queue.front() != Some(&Token::RParen) &&
                               queue.front() != Some(&Token::RBracket) {
        expr_list.push(parse_one(queue, &Some(Token::Prefix))?);
      }
      left = AST::Prefix(token.unwrap(), expr_list);
    },
    _ => {
      return Err(format!("missing operand or not an operand {:?} last_token {:?} queue {:?}", token, last_token, queue));
    }
  }

  return Ok(left);
}

fn parse_one(queue: &mut VecDeque<Token>, last_token: &Option<Token>) -> Result<AST, String> {
  let mut left = parse_left(queue, last_token)?;

  loop {
    // Lookahead for unary minus / negation.
    if queue.len() >= 3 && queue[0] == Token::Whitespace &&
        queue[1] == Token::Minus && queue[2] != Token::Whitespace {
      break;
    }
    if queue.front() == Some(&Token::Whitespace) {
      queue.pop_front();
    }
    // Deals with left-only tokens, and right brackets.
    match queue.front() {
      // Left only tokens or end - propagate left to parents right.
      None | Some(Token::LineEnd) | Some(Token::Num(_)) | Some(Token::Float(_)) |
      Some(Token::Function(_)) | Some(Token::Var(_)) | Some(Token::Word(_)) |
      Some(Token::LParen) | Some(Token::LBracket) => {
        break;
      },
      Some(e @ Token::RParen) |
      Some(e @ Token::RBracket) => {
        // RParen/RBracket propagates back until the last left one which consumes it.
        if last_token.is_none() {
          return Err(format!("unmatched right bracket {:?} queue {:?} last_token {:?}", e, queue, last_token));
        }
        break;
      },
      // Infix-style arithmetic or comparison operators.
      Some(Token::Plus) | Some(Token::Minus) | Some(Token::Multiply) | Some(Token::Divide) |
      Some(Token::Modulo) | Some(Token::Less) | Some(Token::LessEq) | Some(Token::Greater) |
      Some(Token::GreaterEq) | Some(Token::Equal) => {
        // Needs parsing, handled just below this match (because otherwise we would have double
        // reference to queue).  TODO: Is that true?  Could that code from below be put here?
      },
      _ => {
        return Err(format!("Unknown token {:?}", queue.front()));
      },
    }

    // TODO: Get rid of cloned (dependent on precedence function).
    let token = queue.front().cloned();
    // Give the left operand back to the previous operator if the precedence is higher
    // or equal.
    if precedence(last_token) >= precedence(&token) {
      break;
    }
    queue.pop_front();
    let right = parse_one(queue, &token)?;
    left = AST::Binary(token.unwrap(), Box::new(left), Box::new(right));
  }

  return Ok(left);
}

#[derive(Default)]
pub struct Parser {
  saved_tokens: Vec<Token>,
}

impl Parser {
  pub fn new() -> Parser {
    Parser {
      ..Default::default()
    }
  }

  // Should take in only one line (for now).
  pub fn parse(&mut self, input: &str) -> Result<AST, String> {
    let mut tokens = lexer::process(input)?;

    // In case we have a LineCont save, or load saved tokens.
    if tokens.last() == Some(&Token::LineCont) {
      tokens.pop();
      self.saved_tokens.append(&mut tokens);
      return Ok(AST::None);
    }
    if !self.saved_tokens.is_empty() {
      self.saved_tokens.append(&mut tokens);
      tokens = std::mem::replace(&mut self.saved_tokens, Vec::new());
    }

    let mut tokens: VecDeque<Token> = tokens.into_iter().collect();
    let mut expr_list = ExprList::new();
    while tokens.front().is_some() &&
          tokens.front() != Some(&Token::LineEnd) {
      expr_list.push(parse_one(&mut tokens, &None)?);
    }
    if tokens.front() == Some(&Token::LineEnd) {
      tokens.pop_front();
    }
    if !tokens.is_empty() {
      return Err(format!("parse should get only one line of input!"));
    }
    return Ok(AST::ExprLine(expr_list));
  }
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
    AST::Function(name, expr_list) => {
      println!("{}", name);
      rek_print(&AST::ExprList(expr_list.clone().to_vec()), prefix.clone() + "  ");
    },
    AST::Prefix(token, expr_list) => {
      println!("Prefix {:?}", token);
      rek_print(&AST::ExprList(expr_list.clone().to_vec()), prefix.clone() + "  ");
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
    AST::ExprLine(expr_list) => {
      println!("Expression line");
      for (i, element) in expr_list.iter().enumerate() {
        rek_print(element, prefix.clone() + if i < expr_list.len()-1 { "| " } else { "  " });
      }
    },
    AST::None => {
      println!("None");
    },
    _ => {
      println!("Not implemented in rek_print {:?}", item);
    }
  }
}

#[cfg(test)]
mod tests {
  #![allow(non_snake_case)]
  use super::*;
  // use AST::*;

  fn test_line_ok(input: &str, expected: &[AST]) {
    let ast = Parser::new().parse(input).unwrap();
    // rek_print(&ast, "".to_string());
    assert_eq!(AST::ExprLine(expected.to_vec()), ast, "\ninput: {}", input);
  }

  fn Negation(operand: AST) -> AST {
    AST::Unary(Token::Negation, Box::new(operand))
  }

  fn F(float: f32) -> AST {
    AST::Float(float)
  }

  fn I(int: i32) -> AST {
    AST::Float(int as f32)
  }

  fn Prefix(token: Token, expr_list: &[AST]) -> AST {
    AST::Prefix(token, expr_list.to_vec())
  }
  fn PPlus(expr_list: &[AST]) -> AST {
    Prefix(Token::Plus, expr_list)
  }

  fn Binary(token: Token, left: AST, right: AST) -> AST {
    AST::Binary(token, Box::new(left), Box::new(right))
  }

  macro_rules! gen_binary {
    ($name:ident) => {
      fn $name(left: AST, right: AST) -> AST {
        Binary(Token::$name, left, right)
      }
    }
  }
  gen_binary!(Plus);
  gen_binary!(Minus);
  gen_binary!(Multiply);
  gen_binary!(Divide);

  // #[test]
  // fn prefix_list_list() {
  //   test_line_ok("+ 1 2 ", &[
  //     prefix(Token::Plus, &[i(1), i(2), i(3)])
  //   ]);
  // }

  #[test]
  fn batch_tests() {
    for (input, expected) in &[
      ("+ 1 2", &[PPlus(&[I(1), I(2)])][..]),
      ("1 + 2 - 3", &[Minus(Plus(I(1), I(2)),
                            I(3))][..]),
      ("1+2-3", &[Minus(Plus(I(1), I(2)),
                            I(3))][..]),
      ("1 + 2 -3", &[Plus(I(1), I(2)),
                     Negation(I(3))][..]),
      ("1 + (2 * (3 + 4 * -5) + -6 * -(-7 + -8)) * 9", &[
        Plus(I(1),
         Multiply(
          AST::ExprList(vec!(Plus(
           Multiply(
            I(2),
            AST::ExprList(vec!(Plus(
             I(3),
             Multiply(
              I(4),
              Negation(I(5))
             )
            )))
           ),
           Multiply(
            Negation(I(6)),
            Negation(
             AST::ExprList(vec!(Plus(
              Negation(I(7)),
              Negation(I(8))
             )))
            )
           )
          ))),
          I(9)
         )
        )
      ][..]),
    ] {
      test_line_ok(input, expected);
    }
  }

  #[test]
  fn line_cont() {
    let mut parser = Parser::new();
    assert_eq!(AST::None, parser.parse("1 2\\\n").unwrap());
    assert_eq!(AST::ExprLine(vec![AST::Float(1.0), AST::Float(2.0), AST::Float(3.0)]),
               parser.parse("3").unwrap());
  }
}

fn main() {
  use std;
  // 1 + (2 * (3 + 4 * -5) + -6 * -(-7 + -8)) * 9
  let mut parser = Parser::new();
  loop {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    match parser.parse(&input) {
      Ok(val) => {
        println!("{:?}", val);
        rek_print(&val, "".to_string());
      },
      Err(err) => {
        println!("Parsing error: {:?}", err);
      },
    }
  }
}
