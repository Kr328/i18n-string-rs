use alloc::{borrow::Cow, string::String, vec::Vec};

pub fn escape(input: &str) -> Cow<'_, str> {
    if !input.contains(['\'', '\\']) {
        return Cow::Borrowed(input);
    }

    let mut output = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '\'' => output.push_str("\\'"),
            '\\' => output.push_str("\\\\"),
            _ => output.push(c),
        }
    }
    Cow::Owned(output)
}

pub fn escape_bytes(input: &[u8]) -> Cow<'_, [u8]> {
    if !input.contains(&b'\'') && !input.contains(&b'\\') {
        return Cow::Borrowed(input);
    }

    let mut output = Vec::with_capacity(input.len());
    for &c in input {
        match c {
            b'\'' => output.extend_from_slice(b"\\'"),
            b'\\' => output.extend_from_slice(b"\\\\"),
            _ => output.push(c),
        }
    }
    Cow::Owned(output)
}

pub struct Escaped<W>(pub W);

impl<W> Escaped<W> {
    pub fn new(write: W) -> Self {
        Self(write)
    }

    pub fn into_inner(self) -> W {
        self.0
    }
}

impl<W: core::fmt::Write> core::fmt::Write for Escaped<W> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.0.write_str(escape(s).as_ref())
    }

    fn write_char(&mut self, c: char) -> core::fmt::Result {
        match c {
            '\'' => self.0.write_str("\\'"),
            '\\' => self.0.write_str("\\\\"),
            _ => self.0.write_char(c),
        }
    }
}

#[cfg(feature = "std")]
impl<W: std::io::Write> std::io::Write for Escaped<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.write(escape_bytes(buf).as_ref())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.0.flush()
    }
}
