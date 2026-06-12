#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

#[cfg(feature = "cli")]
pub mod cli;
pub mod error;
pub mod template;
pub mod vcard;
