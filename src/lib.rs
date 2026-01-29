#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
extern crate core;

pub mod escape;
mod format;
mod parse;
#[cfg(test)]
mod tests;
mod translate;

use alloc::{
    borrow::Cow,
    boxed::Box,
    format,
    rc::Rc,
    string::{String, ToString},
    sync::Arc,
};
use core::{
    fmt::{Debug, Display, Formatter},
    str::FromStr,
};

use compact_str::CompactString;

#[derive(Debug, PartialEq, Eq)]
pub struct InvalidFormat;

impl Display for InvalidFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "invalid format")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for InvalidFormat {}

pub trait Resolver {
    fn resolve<'s>(&'s self, template: &'s str) -> Cow<'s, str>;
}

macro_rules! impl_resolver_delegate {
    ($typ:ty) => {
        impl<T: Resolver> Resolver for $typ {
            fn resolve<'s>(&'s self, template: &'s str) -> Cow<'s, str> {
                Resolver::resolve(&**self, template)
            }
        }
    };
}

impl_resolver_delegate!(&T);
impl_resolver_delegate!(&mut T);
impl_resolver_delegate!(Box<T>);
impl_resolver_delegate!(Arc<T>);
impl_resolver_delegate!(Rc<T>);

pub struct NoResolver;

impl Resolver for NoResolver {
    fn resolve<'s>(&'s self, template: &'s str) -> Cow<'s, str> {
        template.into()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[non_exhaustive]
pub enum I18nString {
    Literal(CompactString),
    Template(CompactString, Box<[I18nString]>),
}

impl I18nString {
    pub fn literal<S: Into<CompactString>>(s: S) -> Self {
        Self::Literal(s.into())
    }

    pub fn template<S: Into<CompactString>, ARGS: IntoIterator<Item = I18nString>>(s: S, args: ARGS) -> Self {
        Self::Template(s.into(), args.into_iter().collect())
    }
}

impl FromStr for I18nString {
    type Err = InvalidFormat;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse::parse(s)
    }
}

impl Display for I18nString {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        format::format_to(f, self)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for I18nString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for I18nString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = I18nString;

            fn expecting(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
                formatter.write_str("a string")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                value.parse().map_err(serde::de::Error::custom)
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}

pub trait I18nStringTranslateExt {
    fn to_no_translate_string(&self) -> String;
}

impl I18nStringTranslateExt for I18nString {
    fn to_no_translate_string(&self) -> String {
        self.translate(NoResolver)
    }
}

pub trait I18nStringBuilderExt {
    fn display<D: Display + ?Sized>(display: &D) -> Self;
    fn debug<D: Debug + ?Sized>(debug: &D) -> Self;
    fn template_display<D: Display + ?Sized>(display: &D) -> Self;
    fn template_debug<D: Debug + ?Sized>(debug: &D) -> Self;
}

impl I18nStringBuilderExt for I18nString {
    fn display<D: Display + ?Sized>(display: &D) -> Self {
        Self::literal(display.to_string())
    }

    fn debug<D: Debug + ?Sized>(debug: &D) -> Self {
        Self::literal(format!("{:?}", debug))
    }

    fn template_display<D: Display + ?Sized>(display: &D) -> Self {
        Self::template(display.to_string(), [])
    }

    fn template_debug<D: Debug + ?Sized>(debug: &D) -> Self {
        Self::template(format!("{:?}", debug), [])
    }
}
