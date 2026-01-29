use alloc::{string::String, vec::Vec};

use crate::{I18nString, InvalidFormat};

struct Parser<'s> {
    input: &'s str,
    cursor: usize,
}

impl<'s> Parser<'s> {
    fn new(input: &'s str) -> Self {
        Self { input, cursor: 0 }
    }

    fn peek_char(&self) -> Option<char> {
        self.input[self.cursor..].chars().next()
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek_char() {
            if c.is_whitespace() {
                self.cursor += c.len_utf8();
            } else {
                break;
            }
        }
    }

    fn match_char(&mut self, target: char) -> bool {
        if let Some(c) = self.peek_char() {
            if c == target {
                self.cursor += c.len_utf8();
                return true;
            }
        }
        false
    }

    fn expect_char(&mut self, target: char) -> Result<(), InvalidFormat> {
        if self.match_char(target) { Ok(()) } else { Err(InvalidFormat) }
    }

    fn expect_str(&mut self, target: &str) -> Result<(), InvalidFormat> {
        if self.input[self.cursor..].starts_with(target) {
            self.cursor += target.len();
            Ok(())
        } else {
            Err(InvalidFormat)
        }
    }

    fn parse_literal(&mut self) -> Result<String, InvalidFormat> {
        self.expect_char('\'')?;

        let mut escaping = false;

        let mut ret = String::with_capacity(32);
        loop {
            let c = self.input[self.cursor..].chars().next().ok_or(InvalidFormat)?;
            self.cursor += c.len_utf8();
            if escaping {
                escaping = false;

                match c {
                    '\'' => ret.push('\''),
                    'n' => ret.push('\n'),
                    't' => ret.push('\t'),
                    '\\' => ret.push('\\'),
                    _ => ret.push(c),
                }
            } else {
                if c == '\'' {
                    break;
                } else if c == '\\' {
                    escaping = true;
                } else {
                    ret.push(c);
                }
            }
        }

        Ok(ret)
    }

    fn parse_macro(&mut self) -> Result<(String, Vec<I18nString>), InvalidFormat> {
        self.expect_str("t!(")?;
        self.skip_whitespace();

        let template = self.parse_literal()?;
        let mut args: Vec<I18nString> = Vec::with_capacity(3);
        loop {
            self.skip_whitespace();
            if self.match_char(')') {
                break;
            } else if self.match_char(',') {
                self.skip_whitespace();
                args.push(self.parse()?);
            } else {
                return Err(InvalidFormat);
            }
        }

        Ok((template, args))
    }

    fn parse(&mut self) -> Result<I18nString, InvalidFormat> {
        if self.peek_char() == Some('\'') {
            let text = self.parse_literal()?;
            Ok(I18nString::Literal(text.into()))
        } else if self.input[self.cursor..].starts_with("t!(") {
            let (template, args) = self.parse_macro()?;
            Ok(I18nString::Template(template.into(), args.into_boxed_slice()))
        } else {
            Err(InvalidFormat)
        }
    }
}

pub fn parse(input: &str) -> Result<I18nString, InvalidFormat> {
    let mut parser = Parser::new(input);
    parser.parse()
}
