//! rmilter
//! =======
//!
//! **rmilter** is a Rust crate that allows to connect to MTA services like sendmail or postfix using the milter protocol.
//!
//! This library uses pure safe Rust code and doesn't require external libraries like libmilter.
//!
//! Features
//! --------
//!
//! - Connect to MTA services using the milter protocol (IPv4/IPv6 only for now)
//! - Define which messages should be transferred
//! - Automatically decode `base64` and `quoted-printable` values
//! - Uses Rust's type system to prevent misusing the milter protocol
//!
//! Usage
//! -----
//!
//! This crate is [on crates.io](https://crates.io/crates/rmilter) and can be used by adding `rmilter` to your dependencies in your project's `Cargo.toml`.
//!
//! ```toml
//! [dependencies]
//! rmilter = "0.1"
//! ```
//!
//! Example
//! -------
//!
//!  ```
//! use rmilter::accept_reject_action::AcceptRejectAction;
//! use rmilter::message_handler::MessageHandler;
//! use rmilter::milter_message::MilterProtocol;
//! use rmilter::milter_builder::MilterBuilder;
//!
//! struct MyMessageHandler {}
//!
//! impl MessageHandler for MyMessageHandler {
//!     fn header(&mut self, name: &str, value: &str) -> AcceptRejectAction {
//!         println!("name: {}, value: {}", name, value);
//!         AcceptRejectAction::Continue
//!     }
//! }
//!
//! fn main() {
//!     let mut handler = MyMessageHandler {};
//!     let protocol = MilterProtocol::new(false, false, false, false, false, false, false);
//!     let mut milter = MilterBuilder::new(&mut handler)
//!         .set_protocol(protocol)
//!         .build();
//!
//!     // Uncomment this to run the milter (not done here due to doc tests)
//!     //milter
//!     //    .run("127.0.0.1:31337")
//!     //    .expect("Failed to start milter");
//! }
//! ```
//!
//! Status
//! ------
//!
//! **rmilter** can be used to connect to MTA services and receive messages. It is also possible to easily accept or reject a mail (using AcceptRejectAction).
//!
//! Currently, functionality for manipulating the mail (add header, recipients and so on) is not yet supported, but will be in a future release.
#[macro_use]
extern crate lazy_static;

pub mod accept_reject_action;
pub mod message_handler;
pub mod milter;
pub mod milter_builder;
pub mod milter_error;
pub mod milter_message;
