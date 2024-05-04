use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
  LineEnd, // \n
  LineCont, // \\n
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

const CHAR_TO_TOKEN_MAP: [(&str, Token); 20] = [
  // Two-char tokens and their one-char versions.
  ("\\\n", Token::LineCont),
  ("<=", Token::LessEq),
  ("<", Token::Less),
  (">=", Token::GreaterEq),
  (">", Token::Greater),
  // One-char tokens.
  (" ", Token::Whitespace),
  ("\t", Token::Whitespace),
  ("\n", Token::LineEnd),
  ("+", Token::Plus),
  ("-", Token::Minus),
  ("*", Token::Multiply),
  ("%", Token::Modulo),
  ("/", Token::Divide),
  ("(", Token::LParen),
  (")", Token::RParen),
  ("[", Token::LBracket),
  ("]", Token::RBracket),
  ("{", Token::LBrace),
  ("}", Token::RBrace),
  ("=", Token::Equal),
];

// Initial implementation exposed peek/end/next/undo, but next/undo are footguns
// where it's only a matter of time when a bug sneaks in (check commit
// 71fa22a7539636b8673bc5f9a43593deec2cfdcf - forgot to call an undo).
// Exposing peek/peek2/end/advance is much safer (need to explicitly think when
// to advance).
// Lexer needs 2-lookahead at times therefore the peek2.

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

  fn error(&self, info: &str) -> Result<Vec<Token>, String> {
    Err(format!("Error at pos {} ({}): {}", self.pos, self.input.iter().collect::<String>(), info))
  }

  fn peek(&self) -> Option<char> {
    self.input.get(self.pos).map(|&c| c)
  }

  fn peek2(&self) -> Option<char> {
    self.input.get(self.pos + 1).map(|&c| c)
  }

  fn end(&self) -> bool {
    assert!(self.pos <= self.input.len());
    self.pos == self.input.len()
  }

  fn advance(&mut self) -> &mut Self {
    if !self.end() {
      self.pos += 1;
    }
    self
  }

  // ?_.[a-z][A-Z][0-9]
  fn collect_word(&mut self) -> String {
    let mut word = String::new();
    loop {
      let c = self.peek();
      match c {
        Some('\\') => {
          let cc = self.peek2();
          if cc.is_none() || cc == Some('\n') {
            break;
          }
          self.advance().advance();
          word.push(cc.unwrap().to_ascii_uppercase());
        },
        Some(c @ 'a' ..= 'z') |
        Some(c @ 'A' ..= 'Z') |
        Some(c @ '0' ..= '9') |
        Some(c @ '_') |
        Some(c @ '.') |
        Some(c @ '?') => {
          self.advance();
          word.push(c.to_ascii_uppercase());
        },
        _ => {
          break;
        }
      }
    }
    word
  }

  fn process(&mut self) -> Result<Vec<Token>, String> {
    // Make sure we end with a newline which gets converted to LineEnd or LineCont later.
    if self.input.last() != Some(&'\n') {
      self.input.push('\n');
    }
    let mapping: HashMap<&str, Token> = HashMap::from(CHAR_TO_TOKEN_MAP);

    while let Some(c1) = self.peek() {
      let c = format!("{}", c1);
      let cc = if let Some(c2) = self.peek2() {
        format!("{}{}", c1, c2)
      } else {
        "".to_string()
      };

      let token;
      if let Some(t) = mapping.get(cc.as_str()) {
        self.advance().advance();
        token = t.clone();
      } else if let Some(t) = mapping.get(c.as_str()) {
        self.advance();
        token = t.clone();
      } else if c == ":" {
        self.advance();
        token = Token::Var(self.collect_word());
      } else if c == "\"" {
        self.advance();
        token = Token::Word(self.collect_word());
      } else {
        let word = self.collect_word();
        if let Ok(num) = word.parse::<i32>() {
          token = Token::Num(num);
        } else if let Ok(num) = word.parse::<f32>() {
          token = Token::Float(num);
        } else if word.len() > 0 {
          token = Token::Function(word);
        } else { // word.len() == 0
          let f = &format!("unknown char {}", c);
          return self.error(f);
        }
      }

      if !(token == Token::Whitespace && self.tokens.last() == Some(&Token::Whitespace)) {
        self.tokens.push(token.clone());
      }
    }

    let tokens = std::mem::replace(&mut self.tokens, Vec::new());
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
    test_err("fd 20`~\n",
             "Error at pos 5 (fd 20`~\n): unknown char `");
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
    test_ok("repeat \n50[\n", &[
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
  fn dont_skip_whitespace_line_begin() {
    test_ok("  4 5 \n 6\n", &[
      Token::Whitespace,
      Token::Num(4),
      Token::Whitespace,
      Token::Num(5),
      Token::Whitespace,
      Token::LineEnd,
      Token::Whitespace,
      Token::Num(6),
      Token::LineEnd,
    ]);
  }
}
