#![allow(dead_code)]

// TODOs:
// - perhaps a parsing subsystem with an interface that just gives available expressions
// - split into tokenizer / lexer / parser (or something like that)

use std::collections::VecDeque;
use std::mem;

// TODO: Have the Command enum outside of parser so then turtle depends just on
// command, instead of full parser.
#[derive(Debug)]
pub enum Command {
  // Control flow commands.
  Block(Vec<Command>),
  Repeat(u32, Box<Command>),

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

#[derive(Default)]
pub struct Parser {
  tokens: VecDeque<String>,
  current_token: String,
}

impl Parser {
  pub fn new() -> Parser {
    Parser { ..Default::default() }
  }

  fn push_current_token_if_non_empty(&mut self) {
    if self.current_token.len() > 0 {
      let current_token = mem::replace(&mut self.current_token, String::new());
      self.tokens.push_back(current_token.to_lowercase());
    }
  }

  fn push_current_token_if_non_empty_and_capture_delimiter(&mut self, c: char) {
    self.push_current_token_if_non_empty();
    self.tokens.push_back(c.to_string());
  }

  fn is_discardable_delimiter(c: char) -> bool {
    c.is_whitespace()
  }

  fn is_capturable_delimiter(c: char) -> bool {
    c == '[' || c == ']'
  }

  // Current input must end at token boundary.
  pub fn feed(&mut self, input: &str) {
    for c in input.chars() {
      match c {
        _ if Parser::is_discardable_delimiter(c) => self.push_current_token_if_non_empty(),
        _ if Parser::is_capturable_delimiter(c)  => self.push_current_token_if_non_empty_and_capture_delimiter(c),
        _                                        => self.current_token.push(c),
      };
    }
    self.push_current_token_if_non_empty();
  }

  // TODO: Convert peek_token/next_token to return &str ?
  fn has_tokens(&self) -> bool {
    self.tokens.len() > 0
  }

  fn peek_token(&self) -> String {
    self.tokens.front().unwrap().to_string()
  }

  fn next_token(&mut self) -> String {
    self.tokens.pop_front().unwrap()
  }

  fn next_token_as_u32(&mut self) -> u32 {
    self.next_token().parse::<u32>().unwrap_or(0)
  }

  fn next_token_as_f32(&mut self) -> f32 {
    self.next_token().parse::<f32>().unwrap_or(0.0)
  }

  // Block(Vec<Command>),
  // Repeat(u32, Box<Command>),

  // TODO: Careful, might return more commands / or have leftover tokens.
  fn parse(&mut self) -> Command {
    let ret = match self.next_token().as_str() {
      // TODO: Separate functions for parsing different expressions.
      "fd" => Command::Fd(self.next_token_as_f32()),
      "bk" => Command::Bk(self.next_token_as_f32()),
      "rt" => Command::Rt(self.next_token_as_f32()),
      "lt" => Command::Lt(self.next_token_as_f32()),
      "cs" => Command::Cs,
      "repeat" => {
        let count = self.next_token_as_u32();
        let repeated_command = self.parse();
        Command::Repeat(count, Box::new(repeated_command))
      },
      "[" => {
        // TODO: What if there's no "]" at the end?
        // TODO: What if we have mismatched number of "[" and "]"?
        let mut commands: Vec<Command> = Vec::new();
        while self.peek_token() != "]" {
          commands.push(self.parse());
        }
        // Consume the "]".
        assert!(self.next_token() == "]");
        Command::Block(commands)
      },
      _ => Command::Unknown,
    };
    return ret;
  }

  pub fn parse_all(&mut self) -> Vec<Command> {
    let mut commands: Vec<Command> = Vec::new();
    while self.has_tokens() {
      commands.push(self.parse());
    }
    commands
  }
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

fn main() {
  #[allow(unused_variables)]
  let str1 = " rePEat 4[fd 5 rt   90] [lt 5  fd 10 ] ";
  let str2 = " REPEAT 4[fd 5 Rt   90 [ bK  10 FD 50] ] fd 10";
  #[allow(unused_variables)]
  let str3 = "fd "; // TODO: Crashes.

  // let mut input = String::new();
  // std::io::stdin().read_line(&mut input).unwrap();
  let mut parser = Parser::new();

  // parser.feed(str1);
  parser.feed(str2);

  println!("{:?}", parser.parse_all());
}
