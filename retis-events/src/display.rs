use std::{
    fmt::{self, Write},
    result,
};

use log::warn;

use super::TimeSpec;

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub enum DisplayFormatFlavor {
    SingleLine,
    #[default]
    MultiLine,
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub enum TimeFormat {
    #[default]
    MonotonicTimestamp,
    UtcDate,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct DisplayFormat {
    pub flavor: DisplayFormatFlavor,
    pub time_format: TimeFormat,
    pub show_metadata: bool,
    pub monotonic_offset: Option<TimeSpec>,
}

impl DisplayFormat {
    pub fn new(flavor: DisplayFormatFlavor) -> Self {
        Self {
            flavor,
            ..Default::default()
        }
    }

    pub fn show_metadata(&mut self) {
        self.show_metadata = true
    }

    pub fn set_time_format(&mut self, format: TimeFormat) {
        self.time_format = format;
    }

    pub fn set_monotonic_offset(&mut self, offset: TimeSpec) {
        self.monotonic_offset = Some(offset);
    }
}

/// `Formatter` implements `std::fmt::Write` and controls how events are being
/// displayed. The main advantage is formatting is done on the fly without the
/// need for extra String allocations. This is similar to `std::fmt::Formatter`
/// but with our own constraints.
///
/// It supports the following capabilities: indentation, itemization and
/// delimitation. Each of those are always context-based: the capabilities and
/// their configuration can change over time and might end based on input (eg.
/// delimitation).
pub struct Formatter<'a, 'inner> {
    inner: &'a mut fmt::Formatter<'inner>,
    pub conf: FormatterConf,
    level: usize,
    start: bool,
    buf: String,
}

impl<'a, 'inner> Formatter<'a, 'inner> {
    fn new(inner: &'a mut fmt::Formatter<'inner>, conf: FormatterConf) -> Formatter<'a, 'inner> {
        let level = conf.level;

        Self {
            inner,
            conf,
            level,
            start: true,
            buf: String::with_capacity(4096usize),
        }
    }

    /// Directly implement write_fmt to avoid the need of an explicit
    /// `use fmt::Write` by every user. See the `std::write` documentation.
    #[inline]
    pub fn write_fmt(&mut self, args: fmt::Arguments<'_>) -> result::Result<(), fmt::Error> {
        <Self as fmt::Write>::write_fmt(self, args)
    }

    pub fn flush_buf(&mut self) -> result::Result<(), fmt::Error> {
        let mut lines = self.buf.split('\n');
        let prefix = " ".repeat(self.level);

        if let Some(line) = lines.next() {
            if self.start {
                self.start = false;
                self.inner.write_str(&prefix)?;
            }
            self.inner.write_str(line)?;
        }

        lines.try_for_each(|line| {
            self.inner.write_char('\n')?;
            self.inner.write_str(&prefix)?;
            self.inner.write_str(line)
        })?;

        if self.buf.ends_with('\n') {
            self.inner.write_char('\n')?;
            self.start = true;
        }

        self.buf.clear();
        Ok(())
    }
}

impl fmt::Write for Formatter<'_, '_> {
    fn write_str(&mut self, s: &str) -> result::Result<(), fmt::Error> {
        if self.conf.level != self.level {
            if !self.buf.is_empty() {
                self.flush_buf()?;
            }
            self.level = self.conf.level;
        }

        self.buf.push_str(s);
        Ok(())
    }
}

impl Drop for Formatter<'_, '_> {
    fn drop(&mut self) {
        if !self.buf.is_empty() {
            self.flush_buf().expect("Could not flush Formatter buffer");
        }
    }
}

#[derive(Clone, Default)]
pub struct FormatterConf {
    level: usize,
    saved_levels: Vec<usize>,
}

impl FormatterConf {
    pub fn new() -> Self {
        Self::with_level(0)
    }

    pub fn with_level(level: usize) -> Self {
        Self {
            level,
            ..Default::default()
        }
    }

    /// Increase the indentation level by `diff`.
    pub fn inc_level(&mut self, diff: usize) {
        self.saved_levels.push(self.level);
        self.level += diff;
    }

    /// Reset the indentation level to its previous value.
    pub fn reset_level(&mut self) {
        match self.saved_levels.pop() {
            Some(level) => {
                self.level = level;
            }
            None => warn!("Cannot reset the indentation level"),
        }
    }
}

/// Trait controlling how an event or an event section (or any custom type
/// inside it) is displayed. It works by providing an helper returning an
/// implementation of the std::fmt::Display trait, which can be used later to
/// provide different formats. It is also interesting as those helpers can take
/// arguments, unlike a plain std::fmt::Display implementation.
pub trait EventDisplay<'a>: EventFmt {
    /// Display the event using the default event format.
    fn display(
        &'a self,
        format: &'a DisplayFormat,
        conf: FormatterConf,
    ) -> Box<dyn fmt::Display + 'a>;
}

/// Trait controlling how an event or an event section (or any custom type
/// inside it) is formatted.
///
/// Splitting this from EventDisplay allows to 1) not implement boilerplate for
/// all event sections and custom types thanks to the following generic
/// implementation and 2) access `self` directly allowing to access its private
/// members if any.
pub trait EventFmt {
    /// Default formatting of an event.
    fn event_fmt(&self, f: &mut Formatter, format: &DisplayFormat) -> fmt::Result;
}

impl<'a, T> EventDisplay<'a> for T
where
    T: EventFmt,
{
    fn display(
        &'a self,
        format: &'a DisplayFormat,
        conf: FormatterConf,
    ) -> Box<dyn fmt::Display + 'a> {
        struct DefaultDisplay<'a, U> {
            myself: &'a U,
            format: &'a DisplayFormat,
            conf: FormatterConf,
        }
        impl<U: EventFmt> fmt::Display for DefaultDisplay<'_, U> {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                self.myself
                    .event_fmt(&mut Formatter::new(f, self.conf.clone()), self.format)
            }
        }
        Box::new(DefaultDisplay {
            myself: self,
            format,
            conf,
        })
    }
}

/// DelimWriter is a simple helper that prints a character delimiter (e.g: ',' or ' ') only if it's
/// not the first time write() is called. This helps print lists of optional fields.
///
/// # Example:
///
/// ```
/// use std::fmt;
/// use retis_events::DelimWriter;
///
/// struct Flags {
///     opt1: bool,
///     opt2: bool,
/// }
/// impl fmt::Display for Flags {
///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         write!(f, "flags")?;
///         let mut space = DelimWriter::new(' ');
///         if self.opt1 {
///             space.write(f)?;
///             write!(f, "opt1");
///          }
///         if self.opt2 {
///             space.write(f)?;
///             write!(f, "opt2")?;
///          }
///          Ok(())
///     }
/// }
/// ```
pub struct DelimWriter {
    delim: char,
    first: bool,
}

impl DelimWriter {
    /// Create a new DelimWriter
    pub fn new(delim: char) -> Self {
        DelimWriter { delim, first: true }
    }

    /// If it's not the first time it's called, write the delimiter.
    pub fn write(&mut self, f: &mut Formatter) -> fmt::Result {
        match self.first {
            true => self.first = false,
            false => write!(f, "{}", self.delim)?,
        }
        Ok(())
    }
}
