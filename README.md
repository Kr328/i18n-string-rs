# i18n-string

A lightweight and flexible Rust library for handling internationalization strings with template support.

## Getting Started

### Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
i18n-string = "2.0.0"
```

### Basic Usage

```rust
use std::{
    borrow::Cow,
    str::FromStr,
};

use i18n_string::{I18nString, Resolver, I18nStringTranslateExt};

struct SimpleResolver;

impl Resolver for SimpleResolver {
    fn resolve<'s>(&'s self, template: &'s str) -> Cow<'s, str> {
        match template {
            "world" => Cow::Borrowed("<translated world>"),
            _ => template.into(),
        }
    }
}

fn main() {
    let s = I18nString::template(
        "hello {0}, you are {1}",
        [
            I18nString::template("world", []),
            I18nString::literal("123")
        ]
    );
    // or create directly from template
    let sd = I18nString::from_str("t!('hello {0}, you are {1}', t!('world'), '123')").unwrap();
    assert_eq!(s, sd);

    println!("Original: {}", s.to_no_translate_string());
    println!("Translated: {}", s.translate(&SimpleResolver));
}
```
