use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
  LineEnd, // \n
  LineCont, // \\n
  Escape, // \ without a following newline
  Whitespace, // A block (1+ chars) of non-newline whitespace.

  // Can be builtin or user defined.
  Function(String),
  // TO FOO :A :B, or FD :A
  Var(String),
  // Used for assignment (MAKE "A 5) or as a word.
  Word(String),
  // Numbers (I think all numbers are float in PC Logo 4.0 actually).
  Num(i32),
  Float(f32),

  // Arithmetic operators.
  Plus,
  Minus,
  Multiply,
  Divide,
  Modulo,

  // Unary and prefix.  TODO: This maybe shouldn't be in here as lexer doesn't use it, only parser.
  Negation,
  Prefix,

  // Brackets.
  LParen,
  RParen,
  LBracket,
  RBracket,
  LBrace,
  RBrace,

  // Comparison.
  Less,
  LessEq,
  Greater,
  GreaterEq,
  Equal,
}

struct Lexer {
  input: VecDeque<char>,
  tokens: Vec<Token>,
  row: u32,
  col: u32,
}

impl Lexer {
  fn new(input: &str) -> Self {
    Self {
      input: input.chars().collect(),
      tokens: Vec::new(),
      row: 0,
      col: 0,
    }
  }

  fn error(&self, info: &str) -> Result<String, String> {
    Err(format!("Error at pos {},{}: {}", self.row, self.col, info))
  }

  fn peek_char(&self) -> Option<char> {
    self.input.front().map(|&c| c)
  }

  fn next_char(&mut self) -> Option<char> {
    let next = self.peek_char();
    self.input.pop_front();
    if let Some(c) = next {
      if c == '\n' {
        self.row += 1;
        self.col = 0;
      } else {
        self.col += 1;
      }
    }
    next
  }

  fn skip_whitespace(&mut self) -> bool {
    let mut skipped = false;
    while let Some(c) = self.peek_char() {
      match c {
        ' ' | '\t' => {
          self.next_char();
          skipped = true;
        },
        _ => {
          break;
        }
      }
    }
    skipped
  }

  // ?_.[a-z][A-Z][0-9]
  fn next_word(&mut self) -> Result<String, String> {
    let mut word = String::new();
    loop {
      let c = self.peek_char();
      match c {
        None => { break; },
        Some('\\') => {
          self.next_char();
          if let Some(cc) = self.peek_char() {
            if cc != '\n' {
              self.next_char();
              word.push(cc.to_ascii_uppercase());
            }
          }
        },
        Some(c @ 'a' ..= 'z') |
        Some(c @ 'A' ..= 'Z') |
        Some(c @ '0' ..= '9') |
        Some(c @ '_') |
        Some(c @ '.') |
        Some(c @ '?') => {
          self.next_char();
          word.push(c.to_ascii_uppercase());
        },
        _ => { break; }
      }
    }
    Ok(word)
  }

  fn process(&mut self) -> Result<Vec<Token>, String> {
    let mut line_begin = true;

    loop {
      // Skip whitespace, and collect the token if it's not the beginning of the line as it might be
      // needed in the parser.
      if self.skip_whitespace() && !line_begin {
        self.tokens.push(Token::Whitespace);
      }
      line_begin = false;

      // No more input, we're done.
      let c: char;
      match self.peek_char() {
        None => { break; },
        Some(x) => { c = x; },
      }

      let token: Token;
      match c {
        '\n' => {
          self.next_char();
          token = Token::LineEnd;
          line_begin = true;
        },
        '\\' => {
          self.next_char();
          if self.peek_char() == Some('\n') {
            self.next_char();
            token = Token::LineCont;
            line_begin = true;
          } else {
            token = Token::Escape;
          }
        },
        '+' => { self.next_char(); token = Token::Plus; },
        '-' => { self.next_char(); token = Token::Minus; },
        '*' => { self.next_char(); token = Token::Multiply; },
        '%' => { self.next_char(); token = Token::Modulo; },
        '/' => { self.next_char(); token = Token::Divide; },
        '(' => { self.next_char(); token = Token::LParen; },
        ')' => { self.next_char(); token = Token::RParen; },
        '[' => { self.next_char(); token = Token::LBracket; },
        ']' => { self.next_char(); token = Token::RBracket; },
        '{' => { self.next_char(); token = Token::LBrace; },
        '}' => { self.next_char(); token = Token::RBrace; },
        '=' => { self.next_char(); token = Token::Equal; },
        '<' => {
          self.next_char();
          if self.peek_char() == Some('=') {
            self.next_char();
            token = Token::LessEq;
          } else {
            token = Token::Less;
          }
        },
        '>' => {
          self.next_char();
          if self.peek_char() == Some('=') {
            self.next_char();
            token = Token::GreaterEq;
          } else {
            token = Token::Greater;
          }
        },
        ':' => {
          self.next_char();
          token = Token::Var(self.next_word()?);
        },
        '"' => {
          self.next_char();
          token = Token::Word(self.next_word()?);
        },
        _ => {
          let word = self.next_word()?;
          if let Ok(num) = word.parse::<i32>() {
            token = Token::Num(num);
          } else if let Ok(num) = word.parse::<f32>() {
            token = Token::Float(num);
          } else {
            if word.len() == 0 && self.peek_char().is_some() {
              let f = &format!("unknown char {:?}", self.peek_char().unwrap());
              self.error(f)?;
            }
            token = Token::Function(word);
          }
        }
      }
      self.tokens.push(token);
    }

    let mut tokens = std::mem::replace(&mut self.tokens, Vec::new());
    // Make sure we end with a LineEnd/LineCont.
    if tokens.last() != Some(&Token::LineEnd) && tokens.last() != Some(&Token::LineCont) {
      if tokens.last() == Some(&Token::Escape) {
        tokens.pop();
        tokens.push(Token::LineCont);
      } else {
        tokens.push(Token::LineEnd);
      }
    }
    Ok(tokens)
  }
}

pub fn process(input: &str) -> Result<Vec<Token>, String> {
  Lexer::new(input).process()
}

#[cfg(test)]
mod tests {
  use super::*;

  fn test_ok(input: &str, expected: &[Token]) {
    let lexed = Lexer::new(input).process();
    let expected = Ok(expected.to_vec());
    assert_eq!(expected, lexed);
  }

  fn test_err(input: &str, expected: &str) {
    let lexed = Lexer::new(input).process();
    let expected = Err(expected.to_string());
    assert_eq!(expected, lexed);
  }

  #[test]
  fn unknown_char() {
    test_err("fd 20`~",
             "Error at pos 0,5: unknown char '`'");
  }

  #[test]
  fn var() {
    test_ok("TO FOO :A\nFD :A\nEND\n", &[
      Token::Function("TO".to_string()),
      Token::Whitespace,
      Token::Function("FOO".to_string()),
      Token::Whitespace,
      Token::Var("A".to_string()),
      Token::LineEnd,
      Token::Function("FD".to_string()),
      Token::Whitespace,
      Token::Var("A".to_string()),
      Token::LineEnd,
      Token::Function("END".to_string()),
      Token::LineEnd,
    ]);
  }

  #[test]
  fn word() {
    test_ok("MAKE \"ASD \"SOMETHING\n", &[
      Token::Function("MAKE".to_string()),
      Token::Whitespace,
      Token::Word("ASD".to_string()),
      Token::Whitespace,
      Token::Word("SOMETHING".to_string()),
      Token::LineEnd,
    ]);
  }

  #[test]
  fn function() {
    test_ok("shown? []\n", &[
      Token::Function("SHOWN?".to_string()),
      Token::Whitespace,
      Token::LBracket,
      Token::RBracket,
      Token::LineEnd,
    ]);
  }

  #[test]
  fn number_float() {
    test_ok("bk 50.5 rt  .5 fd 19.\n", &[
      Token::Function("BK".to_string()),
      Token::Whitespace,
      Token::Float(50.5),
      Token::Whitespace,
      Token::Function("RT".to_string()),
      Token::Whitespace,
      Token::Float(0.5),
      Token::Whitespace,
      Token::Function("FD".to_string()),
      Token::Whitespace,
      Token::Float(19.),
      Token::LineEnd,
    ]);
  }

  #[test]
  fn number_num() {
    test_ok("repeat \n 50[\n", &[
      Token::Function("REPEAT".to_string()),
      Token::Whitespace,
      Token::LineEnd,
      Token::Num(50),
      Token::LBracket,
      Token::LineEnd,
    ]);
  }

  #[test]
  fn line_cont() {
    test_ok("REPEAT 4 [FD 40\\\nRT 90]fd 50\n", &[
      Token::Function("REPEAT".to_string()),
      Token::Whitespace,
      Token::Num(4),
      Token::Whitespace,
      Token::LBracket,
      Token::Function("FD".to_string()),
      Token::Whitespace,
      Token::Num(40),
      Token::LineCont,
      Token::Function("RT".to_string()),
      Token::Whitespace,
      Token::Num(90),
      Token::RBracket,
      Token::Function("FD".to_string()),
      Token::Whitespace,
      Token::Num(50),
      Token::LineEnd,
    ]);
  }

  #[test]
  fn add_line_end() {
    test_ok("4", &[
      Token::Num(4),
      Token::LineEnd,
    ]);
  }

  #[test]
  fn add_line_cont() {
    test_ok("4\\", &[
      Token::Num(4),
      Token::LineCont,
    ]);
  }

  #[test]
  fn skip_whitespace_line_begin() {
    test_ok("  4 5 \n 6\n", &[
      Token::Num(4),
      Token::Whitespace,
      Token::Num(5),
      Token::Whitespace,
      Token::LineEnd,
      Token::Num(6),
      Token::LineEnd,
    ]);
  }
}
