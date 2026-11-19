// SPDX-License-Identifier: MIT OR Apache-2.0
//! The Pixel 2 lab's attended tries. An entry file (tests/attended/*.rs, run by rust-script) reads like a
//! checklist: `use lpoco_attended::prelude::*;` for the runner, `use walleye_tries::touch::Touch;` for
//! this phone's steps. What a step does, parses and retries lives here, tested without a phone.

pub mod display;
pub mod display_boot;
pub mod display_delay;
pub mod host;
pub mod lab;
pub mod phone;
pub mod pmic;
pub mod reader;
pub mod settings;
pub mod touch;
pub mod ufs;
pub mod watch;
