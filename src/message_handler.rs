use crate::accept_reject_action::AcceptRejectAction;
use crate::milter_message::{MilterMacro, ProtocolFamily};

/// Implement this trait to define the behavior of your milter application.
///
/// All methods have a default implementation which returns AcceptRejectAction::Continue. Overwrite
/// any of these methods to implement the desired behavior.
pub trait MessageHandler {
    /// Milter checks for the current message have been aborted (SMFIC_ABORT).
    ///
    /// # Example:
    /// ```
    /// use rmilter::message_handler::MessageHandler;
    /// use rmilter::milter_message::MilterMacro;
    ///
    /// struct MyMessageHandler {}
    ///
    /// impl MessageHandler for MyMessageHandler {
    ///     fn abort_filter_checks(&mut self) {
    ///         println!("Aborted filter checks");
    ///     }
    /// }
    /// ```
    fn abort_filter_checks(&mut self) {}

    /// A body chunk of the incoming email (SMFIC_BODY).
    ///
    /// - `value` contains the value of the body chunk.
    ///
    /// # Example:
    /// ```
    /// use rmilter::accept_reject_action::AcceptRejectAction;
    /// use rmilter::message_handler::MessageHandler;
    ///
    /// struct MyMessageHandler {}
    ///
    /// impl MessageHandler for MyMessageHandler {
    ///     fn body_chunk(&mut self, value: &str) -> AcceptRejectAction {
    ///         println!("value: {}", value);
    ///         AcceptRejectAction::Continue
    ///     }
    /// }
    /// ```
    #[allow(unused_variables)]
    fn body_chunk(&mut self, value: &str) -> AcceptRejectAction {
        AcceptRejectAction::Continue
    }

    /// Provides information about the connection to the MTA (SMFIC_CONNECT).
    ///
    /// - `hostname` The hostname of the machine running the MTA.
    /// - `family` The protocol family used.
    /// - `port` The used port (Inet4 and Inet6 only).
    /// - `address` The IP address or socket path used.
    ///
    /// # Example:
    /// ```
    /// use rmilter::accept_reject_action::AcceptRejectAction;
    /// use rmilter::message_handler::MessageHandler;
    /// use rmilter::milter_message::ProtocolFamily;
    ///
    /// struct MyMessageHandler {}
    ///
    /// impl MessageHandler for MyMessageHandler {
    ///     fn connection(
    ///     &mut self,
    ///     hostname: &str,
    ///     family: &ProtocolFamily,
    ///     port: &u16,
    ///     address: &str,
    ///     ) -> AcceptRejectAction {
    ///         println!(
    ///             "hostname: {}, family: {:?}, port: {}, address: {}",
    ///             hostname, family, port, address
    ///         );
    ///         AcceptRejectAction::Continue
    ///     }
    /// }
    /// ```
    #[allow(unused_variables)]
    fn connection(
        &mut self,
        hostname: &str,
        family: &ProtocolFamily,
        port: &u16,
        address: &str,
    ) -> AcceptRejectAction {
        AcceptRejectAction::Continue
    }

    /// A set of macros defined by the MTA (SMFIC_MACRO).
    ///
    /// - `cmdcode` represents the command for which the macros are defined.
    /// - `macros` contains the defined macros.
    ///
    /// # Example:
    /// ```
    /// use rmilter::message_handler::MessageHandler;
    /// use rmilter::milter_message::MilterMacro;
    ///
    /// struct MyMessageHandler {}
    ///
    /// impl MessageHandler for MyMessageHandler {
    ///     fn define_macros(&mut self, cmdcode: &char, macros: Vec<MilterMacro>) {
    ///         println!("cmdcode: {}", cmdcode);
    ///     }
    /// }
    /// ```
    #[allow(unused_variables)]
    fn define_macros(&mut self, cmdcode: &char, macros: Vec<MilterMacro>) {}

    /// The MTA informs that all body chunks of the message are sent (SMFIC_BODYEOB).
    ///
    /// # Example:
    /// ```
    /// use rmilter::accept_reject_action::AcceptRejectAction;
    /// use rmilter::message_handler::MessageHandler;
    ///
    /// struct MyMessageHandler {}
    ///
    /// impl MessageHandler for MyMessageHandler {
    ///     fn end_of_body(&mut self) -> AcceptRejectAction {
    ///         println!("End of body");
    ///         AcceptRejectAction::Continue
    ///     }
    /// }
    /// ```
    fn end_of_body(&mut self) -> AcceptRejectAction {
        // TODO: Add support for modifying here (header, body, recipients, etc.)
        AcceptRejectAction::Continue
    }

    /// The MTA informs that all header chunks of the message are sent (SMFIC_EOH).
    ///
    /// # Example:
    /// ```
    /// use rmilter::accept_reject_action::AcceptRejectAction;
    /// use rmilter::message_handler::MessageHandler;
    ///
    /// struct MyMessageHandler {}
    ///
    /// impl MessageHandler for MyMessageHandler {
    ///     fn end_of_header(&mut self) -> AcceptRejectAction {
    ///         println!("End of header");
    ///         AcceptRejectAction::Continue
    ///     }
    /// }
    /// ```
    fn end_of_header(&mut self) -> AcceptRejectAction {
        AcceptRejectAction::Continue
    }

    /// A header chunk (SMFIC_HEADER).
    ///
    /// - `name` defines the name of the provided value.
    /// - `value` contains the actual value.
    ///
    /// # Example:
    /// ```
    /// use rmilter::accept_reject_action::AcceptRejectAction;
    /// use rmilter::message_handler::MessageHandler;
    ///
    /// struct MyMessageHandler {}
    ///
    /// impl MessageHandler for MyMessageHandler {
    ///     fn header(&mut self, name: &str, value: &str) -> AcceptRejectAction {
    ///         println!("name: {}, value: {}", name, value);
    ///         AcceptRejectAction::Continue
    ///     }
    /// }
    /// ```
    #[allow(unused_variables)]
    fn header(&mut self, name: &str, value: &str) -> AcceptRejectAction {
        AcceptRejectAction::Continue
    }

    /// A helo message (SMFIC_HELO).
    ///
    /// - `msg` contains the sent helo message.
    ///
    /// # Example:
    /// ```
    /// use rmilter::accept_reject_action::AcceptRejectAction;
    /// use rmilter::message_handler::MessageHandler;
    ///
    /// struct MyMessageHandler {}
    ///
    /// impl MessageHandler for MyMessageHandler {
    ///     fn helo(&mut self, msg: &str) -> AcceptRejectAction {
    ///         println!("msg: {}", msg);
    ///         AcceptRejectAction::Continue
    ///     }
    /// }
    /// ```
    #[allow(unused_variables)]
    fn helo(&mut self, msg: &str) -> AcceptRejectAction {
        AcceptRejectAction::Continue
    }

    /// A mail from message (SMFIC_MAIL).
    ///
    /// - `address` contains the address of the sender.
    /// - `args` contains optional arguments.
    ///
    /// # Example:
    /// ```
    /// use rmilter::accept_reject_action::AcceptRejectAction;
    /// use rmilter::message_handler::MessageHandler;
    ///
    /// struct MyMessageHandler {}
    ///
    /// impl MessageHandler for MyMessageHandler {
    ///     fn mail_from(&mut self, address: &str, args: &[String]) -> AcceptRejectAction {
    ///         println!("address: {}, args: {:?}", address, args);
    ///         AcceptRejectAction::Continue
    ///     }
    /// }
    /// ```
    #[allow(unused_variables)]
    fn mail_from(&mut self, address: &str, args: &[String]) -> AcceptRejectAction {
        AcceptRejectAction::Continue
    }

    /// Recipient information (SMFIC_RCPT).
    ///
    /// - `recipient` contains the recipient of the message.
    /// - `args` contains optional arguments.
    ///
    /// # Example:
    /// ```
    /// use rmilter::accept_reject_action::AcceptRejectAction;
    /// use rmilter::message_handler::MessageHandler;
    ///
    /// struct MyMessageHandler {}
    ///
    /// impl MessageHandler for MyMessageHandler {
    ///     fn recipient(&mut self, recipient: &str, args: &[String]) -> AcceptRejectAction {
    ///         println!("recipient: {}, args: {:?}", recipient, args);
    ///         AcceptRejectAction::Continue
    ///     }
    /// }
    /// ```
    #[allow(unused_variables)]
    fn recipient(&mut self, recipient: &str, args: &[String]) -> AcceptRejectAction {
        AcceptRejectAction::Continue
    }
}
