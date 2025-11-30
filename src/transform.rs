use alloc::{borrow::Cow, string::String, vec::Vec};

use crate::{InvalidFormat, Resolver};

struct Transformer<'a, R: Resolver> {
    resolver: &'a R,
    input: &'a str,
    cursor: usize,
}

impl<'a, R: Resolver> Transformer<'a, R> {
    fn new(resolver: &'a R, input: &'a str) -> Self {
        Self {
            resolver,
            input,
            cursor: 0,
        }
    }

    fn transform(mut self) -> Result<String, InvalidFormat> {
        let mut result = String::with_capacity(self.input.len());

        while self.cursor < self.input.len() {
            match self.input[self.cursor..].find("t!(") {
                Some(idx) => {
                    let start = self.cursor;
                    let end = self.cursor + idx;
                    result.push_str(&self.input[start..end]);
                    self.cursor = end;

                    result.push_str(&self.parse_macro()?);
                }
                None => {
                    result.push_str(&self.input[self.cursor..]);
                    break;
                }
            }
        }

        Ok(result)
    }

    fn parse_macro(&mut self) -> Result<Cow<'a, str>, InvalidFormat> {
        self.expect_str("t!(")?;
        self.skip_whitespace();

        let template = self.resolver.resolve(self.parse_string_literal()?);
        let mut args: Vec<Cow<'a, str>> = Vec::with_capacity(3);
        loop {
            self.skip_whitespace();
            if self.match_char(')') {
                break;
            } else if self.match_char(',') {
                self.skip_whitespace();
                args.push(self.parse_arg()?);
            } else {
                return Err(InvalidFormat);
            }
        }

        if args.is_empty() {
            Ok(template)
        } else {
            self.format_message(template, &args)
        }
    }

    fn parse_arg(&mut self) -> Result<Cow<'a, str>, InvalidFormat> {
        if self.peek_char() == Some('\'') {
            self.parse_string_literal()
        } else if self.input[self.cursor..].starts_with("t!(") {
            self.parse_macro()
        } else {
            Err(InvalidFormat)
        }
    }

    fn parse_string_literal(&mut self) -> Result<Cow<'a, str>, InvalidFormat> {
        self.expect_char('\'')?;

        let start = self.cursor;
        let mut has_escape = false;

        let end = 'find_end: {
            let mut escaping = false;
            for (idx, c) in self.input[self.cursor..].char_indices() {
                if escaping {
                    escaping = false;
                } else if c == '\\' {
                    has_escape = true;
                    escaping = true;
                } else if c == '\'' {
                    break 'find_end start + idx;
                }
            }
            return Err(InvalidFormat);
        };

        self.cursor = end + 1;

        if !has_escape {
            Ok(Cow::Borrowed(&self.input[start..end]))
        } else {
            let mut result = String::with_capacity(end - start);
            let mut escaping = false;
            for c in self.input[start..end].chars() {
                if escaping {
                    escaping = false;
                    result.push(c);
                } else if c == '\\' {
                    escaping = true;
                } else {
                    result.push(c);
                }
            }
            Ok(Cow::Owned(result))
        }
    }

    fn format_message(&self, template: Cow<'a, str>, args: &[Cow<'a, str>]) -> Result<Cow<'a, str>, InvalidFormat> {
        enum ParseState {
            Normal,
            HitLeftBrace { pos: usize },
            HitRightBrace,
        }

        if !template.contains(['{', '}']) {
            return Ok(template);
        }

        let mut output = String::with_capacity(template.len() + 16);
        let mut state = ParseState::Normal;

        for (idx, c) in template.char_indices() {
            match state {
                ParseState::Normal => {
                    if c == '{' {
                        state = ParseState::HitLeftBrace { pos: idx };
                    } else if c == '}' {
                        state = ParseState::HitRightBrace;
                    } else {
                        output.push(c);
                    }
                }
                ParseState::HitLeftBrace { pos } => {
                    if c == '}' {
                        let arg_idx_str = &template[pos + 1..idx];
                        let arg_idx = arg_idx_str.parse::<usize>().map_err(|_| InvalidFormat)?;
                        if let Some(arg) = args.get(arg_idx) {
                            output.push_str(arg);
                        } else {
                            output.push('{');
                            output.push_str(arg_idx_str);
                            output.push('}');
                        }
                        state = ParseState::Normal;
                    } else if c == '{' {
                        output.push(c);
                        state = ParseState::Normal;
                    }
                }
                ParseState::HitRightBrace => {
                    if c == '}' {
                        output.push(c);
                        state = ParseState::Normal;
                    } else {
                        return Err(InvalidFormat);
                    }
                }
            }
        }

        Ok(Cow::Owned(output))
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

    fn peek_char(&self) -> Option<char> {
        self.input[self.cursor..].chars().next()
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
}

pub fn transform<'s, R: Resolver>(input: &'s str, resolver: &R) -> Result<Cow<'s, str>, InvalidFormat> {
    if !input.contains("t!") {
        return Ok(Cow::Borrowed(input));
    }

    Transformer::new(resolver, input).transform().map(Cow::Owned)
}
