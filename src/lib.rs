//! A crate for providing a custom clap value parser for `--opt key1=val1:key2=val2` syntax.
//!
//! This allows options to have sub-options.
//!
//! # Derive usage
//!
//! ```
//! use clap::Parser;
//! use clap_subopt_parser::{SubOpt, SubOptError, SubOptParser};
//!
//! #[derive(clap::Parser, Debug)]
//! struct Args {
//!     /// Add a buffer with a given offset and source.
//!     ///
//!     /// Example: --buf source=0:offset=1000
//!     #[clap(long="buf", value_parser=SubOptParser::<Buf>::default())]
//!     buf: Vec<Buf>,
//! }
//!
//! #[derive(Debug, Default, Clone)]
//! struct Buf {
//!     source: usize,
//!     offset: usize,
//! }
//!
//! impl SubOpt for Buf {
//!     fn update_from_value(&mut self, k: &str) -> Result<(), SubOptError> {
//!         Err(match k {
//!             "source" | "offset" => SubOptError::MissingValueForKey(k.into()),
//!             _ => SubOptError::UnknownKey(k.into()),
//!         })
//!     }
//!     fn update_from_kvpair(&mut self, k: &str, v: &str) -> Result<(), SubOptError> {
//!         match k {
//!             "source" => {
//!                 self.source = v
//!                     .parse::<usize>()
//!                     .map_err(|e| SubOptError::Custom(e.to_string()))?
//!             }
//!             "offset" => {
//!                 self.offset = v
//!                     .parse::<usize>()
//!                     .map_err(|e| SubOptError::Custom(e.to_string()))?
//!             }
//!             k => return Err(SubOptError::UnknownKey(k.into())),
//!         }
//!         Ok(())
//!     }
//! }
//!
//!
//! eprintln!("{:#?}", Args::parse());
//! ```
//!

#![warn(missing_docs)]

use clap::builder::TypedValueParser;
use std::marker::PhantomData;

/// The [`TypedValueParser`] implementation
#[derive(Default)]
pub struct SubOptParser<T> {
    _opt: PhantomData<T>,
}

impl<T> Clone for SubOptParser<T> {
    fn clone(&self) -> Self {
        Self { _opt: PhantomData }
    }
}

impl<T: SubOpt> TypedValueParser for SubOptParser<T> {
    type Value = T;

    fn parse_ref(
        &self,
        _cmd: &clap::Command,
        _arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        let s = value
            .to_str()
            .expect("SubOptParser requires arguments to be UTF-8");
        let mut val = T::default();
        let opts = s.split(':');
        for opt in opts {
            match opt.split_once('=') {
                Some((k, v)) => val.update_from_kvpair(k, v)?,
                None => val.update_from_value(opt)?,
            }
        }
        Ok(val)
    }
}

/// An argument that has sub-options.
///
/// The implementor must also implement [`std::default::Default`] with sensible defaults.
///
/// It will then be built up from the sub-options given as arguments to its methods.
pub trait SubOpt: Default + Send + Sync + 'static {
    /// Update from a single value, like in the example `--foo value1:value2:value3`.
    ///
    /// Each sub-option is a value without a key in the above example.
    fn update_from_value(&mut self, k: &str) -> Result<(), SubOptError>;
    /// Update from a key-value pair, like in the example `--foo key1=value1:key2=value2`
    ///
    /// Each sub-option is a key-value pair in the above example.
    fn update_from_kvpair(&mut self, k: &str, v: &str) -> Result<(), SubOptError>;
}

/// An error that can happen when parsing a sub-option.
#[derive(Debug)]
pub enum SubOptError {
    /// Unknown key
    UnknownKey(String),
    /// Missing value for key
    MissingValueForKey(String),
    /// Custom error, for example parse errors. Convert these errors into a string.
    Custom(String),
}

impl From<SubOptError> for clap::Error {
    fn from(sub: SubOptError) -> Self {
        match sub {
            SubOptError::UnknownKey(k) => clap::Error::raw(
                clap::ErrorKind::UnknownArgument,
                format!("Unknown key: {}\n", k),
            ),
            SubOptError::MissingValueForKey(k) => clap::Error::raw(
                clap::ErrorKind::EmptyValue,
                format!("Missing value for key '{}'\n", k),
            ),
            SubOptError::Custom(s) => clap::Error::raw(
                clap::ErrorKind::InvalidValue,
                format!("Custom error: {}\n", s),
            ),
        }
    }
}
