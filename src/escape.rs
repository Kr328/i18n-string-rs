use alloc::{borrow::Cow, string::String, vec::Vec};

/// Escape special characters in a string to fit I18nString format.
pub fn escape(input: &str) -> Cow<'_, str> {
    if !input.contains(['\'', '\\', '\n', '\t']) {
        return Cow::Borrowed(input);
    }

    let mut output = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '\'' => output.push_str("\\'"),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\t' => output.push_str("\\t"),
            _ => output.push(c),
        }
    }
    Cow::Owned(output)
}

/// Escape special characters in a byte slice to fit I18nString format.
pub fn escape_bytes(input: &[u8]) -> Cow<'_, [u8]> {
    if !input.iter().any(|&c| c == b'\'' || c == b'\\' || c == b'\n' || c == b'\t') {
        return Cow::Borrowed(input);
    }

    let mut output = Vec::with_capacity(input.len());
    for &c in input {
        match c {
            b'\'' => output.extend_from_slice(b"\\'"),
            b'\\' => output.extend_from_slice(b"\\\\"),
            b'\n' => output.extend_from_slice(b"\\n"),
            b'\t' => output.extend_from_slice(b"\\t"),
            _ => output.push(c),
        }
    }
    Cow::Owned(output)
}

/// A wrapper for a formatter that escapes special characters in I18nString format.
pub struct Escaped<W>(pub W);

impl<W> Escaped<W> {
    /// Create a new `Escaped` formatter.
    pub fn new(write: W) -> Self {
        Self(write)
    }

    /// Get the inner formatter.
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
            '\n' => self.0.write_str("\\n"),
            '\t' => self.0.write_str("\\t"),
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
