use core::fmt::{Formatter, Write};

use crate::{I18nString, escape::Escaped};

pub fn format_to(f: &mut Formatter<'_>, s: &I18nString) -> core::fmt::Result {
    match s {
        I18nString::Literal(s) => {
            f.write_str("'")?;
            Escaped::new(&mut *f).write_str(s)?;
            f.write_str("'")
        }
        I18nString::Template(template, args) => {
            f.write_str("t!('")?;
            Escaped::new(&mut *f).write_str(template)?;
            f.write_str("'")?;
            for arg in args {
                f.write_str(",")?;
                format_to(f, arg)?;
            }
            f.write_str(")")
        }
    }
}
