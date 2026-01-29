use alloc::string::String;
use core::str::FromStr;

use crate::{I18nString, Resolver};

fn translate_to<R: Resolver + ?Sized>(input: &I18nString, output: &mut String, resolver: &R) {
    match input {
        I18nString::Literal(s) => {
            output.push_str(s);
        }
        I18nString::Template(template, args) => {
            enum ParseState {
                Normal,
                HitLeftBrace { pos: usize },
                HitRightBrace,
            }

            let template = resolver.resolve(template);

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
                            match usize::from_str(&template[pos + 1..idx]).ok().and_then(|idx| args.get(idx)) {
                                Some(arg) => {
                                    translate_to(arg, output, resolver);
                                }
                                None => {
                                    // ignore invalid format or no arg
                                    output.push('{');
                                    output.push_str(&template[pos + 1..idx]);
                                    output.push('}');
                                }
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
                            // ignore invalid format
                            output.push('}');
                            output.push(c);
                            state = ParseState::Normal;
                        }
                    }
                }
            }

            match state {
                ParseState::Normal => {}
                ParseState::HitLeftBrace { pos } => {
                    // ignore unclosed left brace
                    output.push('{');
                    output.push_str(&template[pos + 1..]);
                }
                ParseState::HitRightBrace => {
                    // ignore unclosed right brace
                    output.push('}');
                }
            }
        }
    }
}

impl I18nString {
    pub fn translate<R: Resolver>(&self, resolver: R) -> String {
        let mut res = String::with_capacity(32);
        translate_to(self, &mut res, &resolver);
        res
    }
}
