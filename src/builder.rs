use alloc::rc::Rc;
use core::{fmt, marker::PhantomData};
use std::sync::Arc;

use crate::{I18nString, NoResolver, Resolver, Translatable, escape::Escaped};

pub struct WantsTemplate;
pub struct WantsArgs;
pub struct Finish(());

pub struct I18nBuilder<'a, S> {
    output: &'a mut I18nString,
    _state: PhantomData<S>,
}

impl<'a> I18nBuilder<'a, WantsTemplate> {
    pub fn new(output: &'a mut I18nString) -> Self {
        Self {
            output,
            _state: PhantomData,
        }
    }
}

impl<'a> I18nBuilder<'a, WantsTemplate> {
    pub fn template(self, template: &str) -> I18nBuilder<'a, WantsArgs> {
        self.output.get_mut().push_str("t!('");
        self.output.get_mut().push_str(&super::escape::escape(template));
        self.output.get_mut().push_str("'");
        I18nBuilder {
            output: self.output,
            _state: PhantomData,
        }
    }
}

impl<'a> I18nBuilder<'a, WantsArgs> {
    pub fn arg_i18n<Arg: I18n + ?Sized>(self, arg: &Arg) -> Self {
        self.output.get_mut().push_str(",");
        arg.build_i18n(I18nBuilder::new(self.output));
        self
    }

    pub fn arg_fmt(self, format_args: fmt::Arguments) -> Self {
        use fmt::Write;

        self.output.get_mut().push_str(",'");
        Escaped::new(self.output.get_mut())
            .write_fmt(format_args)
            .expect("write_fmt failed");
        self.output.get_mut().push_str("'");
        self
    }

    pub fn arg_display<Arg: fmt::Display + ?Sized>(self, arg: &Arg) -> Self {
        self.arg_fmt(format_args!("{}", arg))
    }

    pub fn arg_debug<Arg: fmt::Debug + ?Sized>(self, arg: &Arg) -> Self {
        self.arg_fmt(format_args!("{:?}", arg))
    }

    pub fn arg_fmt_t(self, format_args: fmt::Arguments) -> Self {
        use fmt::Write;

        self.output.get_mut().push_str(",t!('");
        Escaped::new(self.output.get_mut())
            .write_fmt(format_args)
            .expect("write_fmt failed");
        self.output.get_mut().push_str("')");
        self
    }

    pub fn arg_display_t<Arg: fmt::Display + ?Sized>(self, arg: &Arg) -> Self {
        self.arg_fmt_t(format_args!("{}", arg))
    }

    pub fn arg_debug_t<Arg: fmt::Debug + ?Sized>(self, arg: &Arg) -> Self {
        self.arg_fmt_t(format_args!("{:?}", arg))
    }
}

impl<'a> I18nBuilder<'a, WantsArgs> {
    pub fn finish(self) -> Finish {
        self.output.get_mut().push_str(")");
        Finish(())
    }
}

pub trait I18n {
    fn build_i18n(&self, builder: I18nBuilder<WantsTemplate>) -> Finish;
}

macro_rules! impl_delegate {
    ($typ:ty) => {
        impl<I: I18n + ?Sized> I18n for $typ {
            fn build_i18n(&self, builder: I18nBuilder<WantsTemplate>) -> Finish {
                I::build_i18n(&**self, builder)
            }
        }
    };
}

impl_delegate!(&I);
impl_delegate!(&mut I);
impl_delegate!(Box<I>);
impl_delegate!(Arc<I>);
impl_delegate!(Rc<I>);

pub trait I18nExt: I18n {
    fn to_i18n_string(&self) -> I18nString {
        let mut s = I18nString::alloc(64);
        self.build_i18n(I18nBuilder::new(&mut s));
        s
    }

    fn to_localized_string<R: Resolver + ?Sized>(&self, resolver: &R) -> String {
        let mut s = self.to_i18n_string();
        let _ = s.translate_in_place(resolver);
        s.into_string()
    }

    fn to_no_localized_string(&self) -> String {
        self.to_localized_string(&NoResolver)
    }
}

impl<I: I18n + ?Sized> I18nExt for I {}

pub struct FromFn<F>(pub F);

impl<F: Fn(I18nBuilder<WantsTemplate>) -> Finish> I18n for FromFn<F> {
    fn build_i18n(&self, builder: I18nBuilder<WantsTemplate>) -> Finish {
        self.0(builder)
    }
}

pub fn from_fn<F: Fn(I18nBuilder<WantsTemplate>) -> Finish>(f: F) -> FromFn<F> {
    FromFn(f)
}
