use alloc::string::String;
use core::{fmt, marker::PhantomData};

use crate::{I18nString, NoResolver, Resolver, Translatable, escape::Escaped};

pub trait I18n {
    fn build_i18n(&self, builder: I18nBuilder<WantsTemplate>) -> Finish;

    fn to_i18n_string(&self) -> I18nString {
        let mut s = I18nString::alloc(64);
        self.build_i18n(I18nBuilder::new(&mut s));
        s
    }

    fn to_localized_string<R: Resolver>(&self, resolver: &R) -> String {
        let mut s = self.to_i18n_string();
        let _ = s.translate_in_place(resolver);
        s.into_string()
    }

    fn to_no_localized_string(&self) -> String {
        self.to_localized_string(&NoResolver)
    }
}

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
    pub fn arg_i18n<Arg: I18n>(self, arg: &Arg) -> Self {
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

    pub fn arg_display<Arg: fmt::Display>(self, arg: &Arg) -> Self {
        self.arg_fmt(format_args!("{}", arg))
    }

    pub fn arg_debug<Arg: fmt::Debug>(self, arg: &Arg) -> Self {
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

    pub fn arg_display_t<Arg: fmt::Display>(self, arg: &Arg) -> Self {
        self.arg_fmt_t(format_args!("{}", arg))
    }

    pub fn arg_debug_t<Arg: fmt::Debug>(self, arg: &Arg) -> Self {
        self.arg_fmt_t(format_args!("{:?}", arg))
    }
}

impl<'a> I18nBuilder<'a, WantsArgs> {
    pub fn finish(self) -> Finish {
        self.output.get_mut().push_str(")");
        Finish(())
    }
}
