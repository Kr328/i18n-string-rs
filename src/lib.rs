#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod builder;
pub mod escape;
#[cfg(test)]
mod tests;
mod transform;

use alloc::{borrow::Cow, string::String};
use core::fmt::Display;

#[derive(Debug, PartialEq, Eq)]
pub struct InvalidFormat;

impl Display for InvalidFormat {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "invalid format")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for InvalidFormat {}

pub trait Resolver {
    fn resolve<'s>(&'s self, template: Cow<'s, str>) -> Cow<'s, str>;
}

pub struct NoResolver;

impl Resolver for NoResolver {
    fn resolve<'s>(&'s self, template: Cow<'s, str>) -> Cow<'s, str> {
        template
    }
}

pub fn transform<'s, R>(input: &'s str, resolver: &R) -> Result<Cow<'s, str>, InvalidFormat>
where
    R: Resolver,
{
    transform::transform(input, resolver)
}

pub trait Translatable {
    fn translate_in_place<R>(&mut self, resolver: &R) -> Result<(), InvalidFormat>
    where
        R: Resolver;
}

impl Translatable for String {
    fn translate_in_place<R>(&mut self, resolver: &R) -> Result<(), InvalidFormat>
    where
        R: Resolver,
    {
        match transform(self.as_str(), resolver)? {
            Cow::Owned(s) => *self = s,
            _ => {}
        }
        Ok(())
    }
}
