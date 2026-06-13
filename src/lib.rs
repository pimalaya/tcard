#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

extern crate alloc;
#[cfg(feature = "cli")]
extern crate std;

#[cfg(feature = "cli")]
pub mod cli;
pub mod error;
pub mod template;
pub mod vcard;
