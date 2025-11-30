#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod builder;
pub mod escape;
#[cfg(test)]
mod tests;
mod transform;

use alloc::{borrow::Cow, string::String};
use core::{fmt::Display, ops::Deref};

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

#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct I18nString(String);

impl I18nString {
    pub fn new(s: String) -> Self {
        Self(s)
    }

    pub fn into_string(self) -> String {
        self.0
    }

    pub fn get_ref(&self) -> &String {
        &self.0
    }

    pub fn get_mut(&mut self) -> &mut String {
        &mut self.0
    }
}

impl I18nString {
    pub fn alloc(capacity: usize) -> Self {
        Self(String::with_capacity(capacity))
    }
}

impl Deref for I18nString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for I18nString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for I18nString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for I18nString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Self(s))
    }
}

impl Translatable for I18nString {
    fn translate_in_place<R>(&mut self, resolver: &R) -> Result<(), InvalidFormat>
    where
        R: Resolver,
    {
        match transform(&*self, resolver)? {
            Cow::Owned(s) => self.0 = s,
            _ => {}
        }
        Ok(())
    }
}
