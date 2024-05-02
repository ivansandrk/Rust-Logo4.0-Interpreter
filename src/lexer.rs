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
  input: Vec<char>,
  pos: usize,
  tokens: Vec<Token>,
}

impl Lexer {
  fn new(input: &str) -> Self {
    Self {
      input: input.chars().collect(),
      pos: 0,
      tokens: Vec::new(),
    }
  }

  fn error(&self, info: &str) -> Result<String, String> {
    Err(format!("Error at pos {} ({}): {}", self.pos, self.input.iter().collect::<String>(), info))
  }

  fn peek(&self) -> Option<char> {
    self.input.get(self.pos).map(|&c| c)
  }

  fn end(&self) -> bool {
    assert!(self.pos <= self.input.len());
    self.pos == self.input.len()
  }

  fn next(&mut self) -> Option<char> {
    let ret = self.peek();
    if !self.end() {
      self.pos += 1;
    }
    ret
  }

  fn undo(&mut self) {
    self.pos -= 1;
  }

  fn skip_whitespace(&mut self) -> bool {
    let mut skipped = false;
    while self.peek() == Some(' ') ||
          self.peek() == Some('\t') {
      self.next();
      skipped = true;
    }
    skipped
  }

  // ?_.[a-z][A-Z][0-9]
  fn next_word(&mut self) -> Result<String, String> {
    let mut word = String::new();
    loop {
      let c = self.next();
      match c {
        Some('\\') => {
          let cc = self.peek();
          if cc.is_none() || cc == Some('\n') {
            self.undo(); // Give back the '\\'.
            break;
          }
          self.next(); // Consume the escaped char.
          word.push(cc.unwrap().to_ascii_uppercase());
        },
        Some(c @ 'a' ..= 'z') |
        Some(c @ 'A' ..= 'Z') |
        Some(c @ '0' ..= '9') |
        Some(c @ '_') |
        Some(c @ '.') |
        Some(c @ '?') => {
          word.push(c.to_ascii_uppercase());
        },
        None => { break; },
        _ => {
          self.undo(); // Give back the char.
          break;
        }
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
      if self.end() {
        break;
      }

      let token: Token;
      match self.next().unwrap() {
        '\n' => {
          token = Token::LineEnd;
          line_begin = true;
        },
        '\\' => {
          if self.peek() == Some('\n') {
            self.next();
            token = Token::LineCont;
            line_begin = true;
          } else {
            token = Token::Escape;
          }
        },
        '+' => { token = Token::Plus; },
        '-' => { token = Token::Minus; },
        '*' => { token = Token::Multiply; },
        '%' => { token = Token::Modulo; },
        '/' => { token = Token::Divide; },
        '(' => { token = Token::LParen; },
        ')' => { token = Token::RParen; },
        '[' => { token = Token::LBracket; },
        ']' => { token = Token::RBracket; },
        '{' => { token = Token::LBrace; },
        '}' => { token = Token::RBrace; },
        '=' => { token = Token::Equal; },
        '<' => {
          if self.peek() == Some('=') {
            self.next();
            token = Token::LessEq;
          } else {
            token = Token::Less;
          }
        },
        '>' => {
          if self.peek() == Some('=') {
            self.next();
            token = Token::GreaterEq;
          } else {
            token = Token::Greater;
          }
        },
        ':' => {
          token = Token::Var(self.next_word()?);
        },
        '"' => {
          token = Token::Word(self.next_word()?);
        },
        _ => {
          self.undo(); // Give back the char, it's needed for word processing.
          let word = self.next_word()?;
          if let Ok(num) = word.parse::<i32>() {
            token = Token::Num(num);
          } else if let Ok(num) = word.parse::<f32>() {
            token = Token::Float(num);
          } else {
            if word.len() == 0 && self.peek().is_some() {
              // TODO: Quote ('\\') parsing should be done here.
              let f = &format!("unknown char {:?}", self.peek().unwrap());
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
    assert_eq!(expected, lexed, "'''{}'''", input);
  }

  fn test_err(input: &str, expected: &str) {
    let lexed = Lexer::new(input).process();
    let expected = Err(expected.to_string());
    assert_eq!(expected, lexed, "'''{}'''", input);
  }

  #[test]
  fn unknown_char() {
    test_err("fd 20`~",
             "Error at pos 5 (fd 20`~): unknown char '`'");
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
