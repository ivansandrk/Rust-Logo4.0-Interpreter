// - implement Iter(able) on Lexer
// - forgo Strings, use &str everywhere
// keywords go in parser, or maybe even evaluator

// Need to import used modules.  If you use things like "std::str::Chars"
// then you need to import std (use std).
// use std;
// use std::*;
use std::iter::Peekable;
use std::str::Chars;
use std::collections::HashSet;
use std::collections::HashMap;
use std::iter::FromIterator;
use std::mem::replace;

// GRAPHICS
// ...
const KEYWORDS: &str = "
  FD FORWARD BK BACK LT LEFT RT RIGHT
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
  Line, // \n
  LineCont, // \\n
  Escape, // \ without a following newline
  Whitespace, // A block (1+ chars) of non-newline whitespace.

  // Used for assignment (MAKE "A 5) or as a word.
  Word(String),
  // Can be builtin or user defined.
  Function(String),
  // TO FOO :A :B, or FD :A
  Var(String),
  // Numbers (I think all numbers are float in PC Logo 4.0 actually).
  Num(i32),
  Float(f32),

  // Arithmetic operators.
  Plus,
  Minus,
  Multiply,
  Divide,
  Modulo,

  // Unary.
  Negation,

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

  // Not known yet.
  Unknown,
  None,
}

pub struct Lexer<'a> {
  iter: Peekable<Chars<'a>>,
  tokens: Vec<Token>,
  row: u32,
  col: u32,
  keyword_set: HashSet<&'static str>,
}

impl<'a> Lexer<'a> {
  pub fn new(input: &'a str) -> Self {
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
        Some(c @ 'a' ... 'z') |
        Some(c @ 'A' ... 'Z') |
        Some(c @ '0' ... '9') |
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

  pub fn process(&mut self) -> Result<Vec<Token>, String> {
    loop {
      // Skip whitespace, but collect the token as it might be needed in the parser.
      if self.skip_whitespace() {
        self.tokens.push(Token::Whitespace);
      }

      // No more input, we're done.
      let c: char;
      match self.peek_char() {
        None => { break; },
        Some(x) => { c = x; },
      }

      let token: Token;
      match c {
        '\n' => { self.next_char(); token = Token::Line; },
        '\\' => {
          self.next_char();
          if self.peek_char() == Some('\n') {
            self.next_char();
            token = Token::LineCont;
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

    let tokens = replace(&mut self.tokens, Vec::new());
    Ok(tokens)
  }
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
    test_ok("TO FOO :A\nFD :A\nEND", &[
      Token::Function("TO".to_string()),
      Token::Function("FOO".to_string()),
      Token::Var("A".to_string()),
      Token::Line,
      Token::Function("FD".to_string()),
      Token::Var("A".to_string()),
      Token::Line,
      Token::Function("END".to_string()),
    ]);
  }

  #[test]
  fn word() {
    test_ok("MAKE \"ASD \"SOMETHING", &[
      Token::Function("MAKE".to_string()),
      Token::Word("ASD".to_string()),
      Token::Word("SOMETHING".to_string()),
    ]);
  }

  #[test]
  fn function() {
    test_ok("shown? []", &[
      Token::Function("SHOWN?".to_string()),
      Token::LBracket,
      Token::RBracket,
    ]);
  }

  #[test]
  fn number_float() {
    test_ok("bk 50.5 rt  .5 fd 19.", &[
      Token::Function("BK".to_string()),
      Token::Float(50.5),
      Token::Function("RT".to_string()),
      Token::Float(0.5),
      Token::Function("FD".to_string()),
      Token::Float(19.),
    ]);
  }

  #[test]
  fn number_num() {
    test_ok("repeat \n 50[", &[
      Token::Function("REPEAT".to_string()),
      Token::Line,
      Token::Num(50),
      Token::LBracket,
    ]);
  }

  #[test]
  fn line_cont() {
    test_ok ("REPEAT 4 [FD 40\\\nRT 90]fd 50\n", &[
      Token::Function("REPEAT".to_string()),
      Token::Num(4),
      Token::LBracket,
      Token::Function("FD".to_string()),
      Token::Num(40),
      Token::LineCont,
      Token::Line,
      Token::Function("RT".to_string()),
      Token::Num(90),
      Token::RBracket,
      Token::Function("FD".to_string()),
      Token::Num(50),
      Token::Line,
    ]);
  }
}

// fn test_lexer(input: &str) {
//   println!("{:?}", Lexer::new(input).process());
// }

// fn main() {
//   #[allow(unused_variables)]
//   test_lexer(" rePEat 4[fd 5 rt   90] [lt 5  fd 10 ] ");
//   test_lexer(" REPEAT 4[fd 5 Rt   90 [ bK  10 FD 50] ] fd 10");
//   test_lexer("fd ");
//   test_lexer(" :ASD1 2 3.4 .5a");
//   test_lexer("fd 50 :5 :");
//   test_lexer("shown? []");
//   test_lexer("TO DOE?T :ASD? :BB");
// }
