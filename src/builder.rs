use core::{fmt, marker::PhantomData};

use crate::escape::Escaped;

pub trait I18n {
    fn build_i18n<W: fmt::Write>(&self, builder: I18nBuilder<W, WantsTemplate>) -> Result<W, fmt::Error>;
}

pub struct WantsTemplate;
pub struct WantsArgs;

pub struct I18nBuilder<W, S> {
    write: W,
    _state: PhantomData<S>,
}

impl<W> I18nBuilder<W, WantsTemplate> {
    pub fn new(write: W) -> Self {
        Self {
            write,
            _state: PhantomData,
        }
    }
}

impl<W: fmt::Write> I18nBuilder<W, WantsTemplate> {
    pub fn template(mut self, template: &str) -> Result<I18nBuilder<W, WantsArgs>, fmt::Error> {
        self.write.write_str("t!('")?;
        self.write.write_str(&super::escape::escape(template))?;
        self.write.write_str("'")?;
        Ok(I18nBuilder {
            write: self.write,
            _state: PhantomData,
        })
    }
}

impl<W: fmt::Write> I18nBuilder<W, WantsArgs> {
    pub fn arg_i18n<Arg: I18n>(mut self, arg: &Arg) -> Result<Self, fmt::Error> {
        self.write.write_str(",")?;
        arg.build_i18n(I18nBuilder::new(&mut self.write))?;
        Ok(self)
    }

    pub fn arg_fmt(mut self, format_args: fmt::Arguments) -> Result<Self, fmt::Error> {
        use fmt::Write;

        self.write.write_str(",'")?;
        {
            // to avoid recursive type reference
            let mut escaped = Escaped::new(self.write);
            escaped.write_fmt(format_args)?;
            self.write = escaped.into_inner();
        }
        self.write.write_str("'")?;
        Ok(self)
    }

    pub fn arg_display<Arg: fmt::Display>(self, arg: &Arg) -> Result<Self, fmt::Error> {
        self.arg_fmt(format_args!("{}", arg))
    }

    pub fn arg_debug<Arg: fmt::Debug>(self, arg: &Arg) -> Result<Self, fmt::Error> {
        self.arg_fmt(format_args!("{:?}", arg))
    }
}

impl<W: fmt::Write> I18nBuilder<W, WantsArgs> {
    pub fn finish(mut self) -> Result<W, fmt::Error> {
        self.write.write_str(")")?;
        Ok(self.write)
    }
}
