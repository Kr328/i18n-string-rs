use core::{fmt, marker::PhantomData};

use crate::{I18nString, escape::Escaped};

pub trait I18n {
    fn build_i18n(&self, builder: I18nBuilder<WantsTemplate>) -> Result<(), fmt::Error>;
}

pub struct WantsTemplate;
pub struct WantsArgs;

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
    pub fn template(self, template: &str) -> Result<I18nBuilder<'a, WantsArgs>, fmt::Error> {
        self.output.get_mut().push_str("t!('");
        self.output.get_mut().push_str(&super::escape::escape(template));
        self.output.get_mut().push_str("'");
        Ok(I18nBuilder {
            output: self.output,
            _state: PhantomData,
        })
    }
}

impl<'a> I18nBuilder<'a, WantsArgs> {
    pub fn arg_i18n<Arg: I18n>(self, arg: &Arg) -> Result<I18nBuilder<'a, WantsArgs>, fmt::Error> {
        self.output.get_mut().push_str(",");
        arg.build_i18n(I18nBuilder::new(self.output))?;
        Ok(self)
    }

    pub fn arg_fmt(self, format_args: fmt::Arguments) -> Result<Self, fmt::Error> {
        use fmt::Write;

        self.output.get_mut().push_str(",'");
        Escaped::new(self.output.get_mut()).write_fmt(format_args)?;
        self.output.get_mut().push_str("'");
        Ok(self)
    }

    pub fn arg_display<Arg: fmt::Display>(self, arg: &Arg) -> Result<Self, fmt::Error> {
        self.arg_fmt(format_args!("{}", arg))
    }

    pub fn arg_debug<Arg: fmt::Debug>(self, arg: &Arg) -> Result<Self, fmt::Error> {
        self.arg_fmt(format_args!("{:?}", arg))
    }
}

impl<'a> I18nBuilder<'a, WantsArgs> {
    pub fn finish(self) -> Result<(), fmt::Error> {
        self.output.get_mut().push_str(")");
        Ok(())
    }
}
