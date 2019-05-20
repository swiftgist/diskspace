extern crate clap;
#[cfg(test)]
use clap::{App, Arg};
mod ds;
#[cfg(feature = "multiple")]
mod ds1;
#[cfg(feature = "multiple")]
mod ds2;
#[cfg(feature = "multiple")]
mod ds3;
#[cfg(feature = "multiple")]
mod ds4;
#[cfg(feature = "multiple")]
mod ds5;
mod report;

pub use ds::*;
pub use report::*;
