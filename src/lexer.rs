// *
// - token for whitespace? it's possible newline is needed

use std::collections::HashSet;
use std::iter::FromIterator;

const DELIMITERS: &str = "()[]{}+-*/<>=";
const SIMPLE_TOKEN_MAP: &[(char, Token)] = &[
  ('+', Token::Plus),
  ('-', Token::Minus),
  ('*', Token::Multiply),
  ('/', Token::Divide),
  ('(', Token::OpenParen),
  (')', Token::CloseParen),
  ('[', Token::OpenBracket),
  (']', Token::CloseBracket),
  ('{', Token::OpenBrace),
  ('}', Token::CloseBrace),
  ('<', Token::Less),
  ('>', Token::Greater),
  ('=', Token::Equal),
];
// TODO: Purposeful error in FORWARD just to see how the whole program
// deals with it.
// GRAPHICS
// ...
const KEYWORDS: &str = "
  FD FFORWARD BK BACK LT LEFT RT RIGHT
  SETPOS SETXY SETX SETY SETHEADING HOME ARC
  GETXY POS XCOR YCOR HEADING TOWARDS
  SCRUNCH SETSCRUNCH
  ST SHOWTURTLE HT HIDETURTLE CLEAN CS CLEARSCREEN
  WRAP WINDOW FENCE TURTLEMODE
  FILLED 
  LABEL SETLABELHEIGHT LABELSIZE
  TS TEXTSCREEN FS FULLSCREEN SS SPLITSCREEN SCREENMODE
  REFRESH NOREFRESH
  SHOWNP SHOWN?
  PD PENDOWN PU PENUP PPT PENPAINT PE PENERASE PX PENREVERSE
  SETPC SETPENCOLOR SETPALETTE SETPENSIZE SETPENPATTERN SETPEN
  SETBG SETBACKGROUND BG BACKGROUND
  PENDOWNP PENDOWN? PENMODE PC PENCOLOR PALETTE PENSIZE PEN
  SAVEPICT LOADPICT EPSPICT
  MOUSEPOS CLICKPOS BUTTONP BUTTON?

  TO END
  REPEAT FOR WHILE 
  LOCAL MAKE OP
  LIST LPUT FPUT
  COUNT ITEM
  READ
";

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
  // TODO: Word can be function or var.
  // What about strings?
  Word(String),
  Keyword(String),

  // Numbers and variables.
  Num(i32),
  Float(f32),
  Var(String), // :R
  Assignment(String), // MAKE "O 123

  // Arithmetic operators.
  Plus,
  Minus,
  Multiply,
  Divide,

  // Brackets.
  OpenParen,
  CloseParen,
  OpenBracket,
  CloseBracket,
  OpenBrace,
  CloseBrace,

  // Comparison.
  Less,
  Greater,
  Equal,

  // Not known yet.
  Unknown,
}

pub struct Lexer<'a> {
  iter: std::iter::Peekable<std::str::Chars<'a>>,
  tokens: Vec<Token>,
  row: u32,
  col: u32,
  keyword_set: std::collections::HashSet<&'static str>,
}

impl<'a> Lexer<'a> {
  fn new(input: &'a str) -> Self {
    Self {
      iter: input.chars().peekable(),
      tokens: Vec::new(),
      row: 0,
      col: 0,
      keyword_set: HashSet::from_iter(KEYWORDS.trim().split([' ', '\n'].as_ref())),
    }
  }

  fn error(&mut self, info: &str) -> Result<String, String> {
    Err(format!("Error at pos {},{}: {}", self.row, self.col, info))
  }

  fn check_error_delimiter(&mut self, word: &str) -> Result<(), String> {
    if let Some(c) = self.peek_char() {
      if !self.is_delimiter(c) {
        self.error(&format!("invalid delimiter '{}' after token '{}'", c, word))?;
      }
    }
    Ok(())
  }

  fn peek_char(&mut self) -> Option<char> {
    // Have the return types of peek & next consistent.
    match self.iter.peek() {
      Some(&c) => Some(c),
      None     => None,
    }
  }

  fn next_char(&mut self) -> Option<char> {
    let next = self.iter.next();
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

  fn skip_whitespace(&mut self) {
    while let Some(c) = self.peek_char() {
      if !c.is_whitespace() {
        break;
      }
      self.next_char();
    }
  }

  fn is_delimiter(&mut self, c: char) -> bool {
    if c.is_whitespace() {
      return true;
    }
    DELIMITERS.contains(c)
  }

  // [a-z][a-z0-9]*[?]?
  fn next_word(&mut self) -> Result<String, String> {
    let mut word = String::new();
    // Must start with [a-z].
    if let Some(c) = self.peek_char() {
      if !c.is_alphabetic() {
        self.error(&format!("word starts with a letter, got '{}'", c))?;
      }
    }
    // Collect [a-z0-9].
    while let Some(c) = self.peek_char() {
      if !c.is_alphanumeric() {
        break;
      }
      word.push(c.to_ascii_uppercase());
      self.next_char();
    }
    if word.len() == 0 {
      self.error(&format!("missing word"))?;
    }
    if let Some('?') = self.peek_char() {
      word.push('?');
      self.next_char();
    }
    self.check_error_delimiter(word.as_str())?;
    // Only keywords can end with a '?'.
    if word.ends_with('?') && !self.keyword_set.contains(word.as_str()) {
      self.error(&format!("identifier cannot end with a ? {}", word))?;
    }
    Ok(word)
  }

  // 123, 3.5, .5 , 1.; . -> error.
  // Collect digits and periods -> digits{1,}, periods{,1}
  fn next_number(&mut self) -> Result<Token, String> {
    let mut num = String::new();
    let mut digits = 0;
    let mut periods = 0;
    loop {
      match self.peek_char() {
        None => {
          break;
        },
        Some(c) => {
          if !c.is_digit(10) && c != '.' {
            break;
          }
          num.push(c);
          if c == '.' {
            periods += 1;
          } else {
            digits += 1;
          }
        }
      }
      self.next_char();
    }
    self.check_error_delimiter(num.as_str())?;
    if digits < 1 || periods > 1 {
      self.error(&format!("invalid number '{}'", num))?;
    }
    if periods == 1 {
      // TODO: Get rid of unwrap here after some testing.
      Ok(Token::Float(num.parse::<f32>().unwrap()))
    } else {
      // TODO: Get rid of unwrap here after some testing.
      Ok(Token::Num(num.parse::<i32>().unwrap()))
    }
  }

  fn lex(&mut self) -> Result<Vec<Token>, String> {
    let simple_token_map: std::collections::HashMap<char, Token> =
        SIMPLE_TOKEN_MAP.iter().cloned().collect();

    loop {
      // Skip that pesky whitespace!
      self.skip_whitespace();

      // No more input, we're done.
      let c: char;
      match self.peek_char() {
        None => { break; },
        Some(x) => { c = x; },
      }

      // Simple one char token, emit it immediately and go to next one.
      if let Some(token) = simple_token_map.get(&c) {
        self.tokens.push(token.clone());
        self.next_char();
        continue;
      }

      let token: Token;
      match c {
        ':' => {
          self.next_char();
          token = Token::Var(self.next_word()?);
        },
        '"' => {
          self.next_char();
          token = Token::Assignment(self.next_word()?);
        },
        // TODO: Probably need to think about 0xA3, 0b1101, 0o70.
        _ if c.is_digit(10) || c == '.' => {
          // Integer or floating point number.
          token = self.next_number()?;
        },
        _ if c.is_alphabetic() => {
          let word = self.next_word()?;
          token = if self.keyword_set.contains(word.as_str()) {
              Token::Keyword(word)
          } else {
              Token::Word(word)
          };
        },
        _ => {
          self.error(&format!("unknown char '{}'", c))?;
          token = Token::Unknown;
        },
      }
      self.tokens.push(token);
    }

    let tokens = std::mem::replace(&mut self.tokens, Vec::new());
    Ok(tokens)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn test_ok(input: &str, expected: &[Token]) {
    let lexed = Lexer::new(input).lex();
    let expected = Ok(expected.to_vec());
    assert_eq!(expected, lexed);
  }

  fn test_err(input: &str, expected: &str) {
    let lexed = Lexer::new(input).lex();
    let expected = Err(expected.to_string());
    assert_eq!(expected, lexed);
  }

  #[test]
  fn word_doesnt_start_with_a_letter() {
    test_err("fd :2BAR",
             "Error at pos 0,4: word starts with a letter, got '2'");
  }

  #[test]
  fn missing_word() {
    test_err("MaKE \"",
             "Error at pos 0,6: missing word");
  }

  #[test]
  fn word_bad_delimiter() {
    test_err("EMPTY?5 FD",
             "Error at pos 0,6: invalid delimiter '5' after token 'EMPTY?'");
  }

  #[test]
  fn function_identifier_cannot_end_with_question_mark() {
    test_err("TO FOO? :BAR",
             "Error at pos 0,7: identifier cannot end with a ? FOO?");
  }

  #[test]
  fn var_identifier_cannot_end_with_question_mark() {
    test_err("TO FOO :BAR?",
             "Error at pos 0,12: identifier cannot end with a ? BAR?");
  }

  #[test]
  fn keyword_can_end_with_question_mark() {
    test_ok("shown? []", &[
      Token::Keyword("SHOWN?".to_string()),
      Token::OpenBracket,
      Token::CloseBracket,
    ]);
  }

  #[test]
  fn number_bad_delimiter() {
    test_err("fd 50a",
             "Error at pos 0,5: invalid delimiter 'a' after token '50'");
  }

  #[test]
  fn number_too_many_periods() {
    test_err("fd 50.4.",
             "Error at pos 0,8: invalid number '50.4.'");
  }

  #[test]
  fn number_no_digits() {
    test_err("rt \n.",
             "Error at pos 1,1: invalid number '.'");
  }

  #[test]
  fn number_float() {
    test_ok("bk 50.5 rt  .5 fd 19.", &[
      Token::Keyword("BK".to_string()),
      Token::Float(50.5),
      Token::Keyword("RT".to_string()),
      Token::Float(0.5),
      Token::Keyword("FD".to_string()),
      Token::Float(19.),
    ]);
  }

  #[test]
  fn number_num() {
    test_ok("repeat \n 50[", &[
      Token::Keyword("REPEAT".to_string()),
      Token::Num(50),
      Token::OpenBracket,
    ]);
  }

  #[test]
  fn unknown_token() {
    test_err("fd 5 `",
             "Error at pos 0,5: unknown char '`'");
  }
}

fn test_lexer(input: &str) {
  println!("{:?}", Lexer::new(input).lex());
}

fn main() {
  #[allow(unused_variables)]
  test_lexer(" rePEat 4[fd 5 rt   90] [lt 5  fd 10 ] ");
  test_lexer(" REPEAT 4[fd 5 Rt   90 [ bK  10 FD 50] ] fd 10");
  test_lexer("fd ");
  test_lexer(" :ASD1 2 3.4 .5a");
  test_lexer("fd 50 :5 :");
  test_lexer("shown? []");
  test_lexer("TO DOE?T :ASD? :BB");
}
