use crate::message_handler::MessageHandler;
use crate::milter::Milter;
use crate::milter_message::MilterProtocol;

/// Used to build a Milter.
///
/// This crate tries to make it as easy as possible to use milter functionality in rust, while at
/// the same time make it as difficult as possible to violate the milter protocol. The builder
/// pattern is used here to provide an API that achieves these requirements.
///
/// One example of this is that the user doesn't need to implement OPTNEG messages manually and
/// instead only defines what should be enabled by using the `set_protocol` method and rmilter uses
/// this information during option negotiation with the MTA.
///
/// If the `set_protocol` method is not used, all functionality is enabled by default during option
/// negotiation.
///
/// # Example
/// ```
/// use rmilter::milter_builder::MilterBuilder;
/// use rmilter::message_handler::MessageHandler;
///
/// struct MyHandler;
/// impl MessageHandler for MyHandler {}
///
/// let mut handler = MyHandler {};
///
/// let mut milter = MilterBuilder::new(&mut handler)
///     .build();
/// ```
pub struct MilterBuilder<'a> {
    message_handler: &'a mut dyn MessageHandler,
    protocol: Option<MilterProtocol>,
}

impl<'a> MilterBuilder<'a> {
    /// Creates a Milter from the MilterBuilder configuration.
    ///
    /// # Example
    /// ```
    /// use rmilter::milter_builder::MilterBuilder;
    /// use rmilter::message_handler::MessageHandler;
    ///
    /// struct MyHandler;
    /// impl MessageHandler for MyHandler {}
    ///
    /// let mut handler = MyHandler {};
    ///
    /// let mut milter = MilterBuilder::new(&mut handler)
    ///     .build();
    /// ```
    pub fn build(self) -> Milter<'a> {
        Milter::new(self.message_handler, self.protocol)
    }

    /// Creates a new MilterBuilder with a given MessageHandler.
    ///
    /// The MessageHandler is passed as a mutable borrow to allow the user of the milter to store
    /// and use state inside the MessageHandler.
    ///
    /// # Example
    /// ```
    /// use rmilter::milter_builder::MilterBuilder;
    /// use rmilter::message_handler::MessageHandler;
    ///
    /// struct MyHandler;
    /// impl MessageHandler for MyHandler {}
    ///
    /// let mut handler = MyHandler {};
    ///
    /// let mut milter = MilterBuilder::new(&mut handler)
    ///     .build();
    /// ```
    pub fn new(message_handler: &'a mut impl MessageHandler) -> Self {
        Self {
            message_handler,
            protocol: None,
        }
    }

    /// Used to define the protocol for communicating with the MTA.
    ///
    /// # Example
    /// ```
    /// use rmilter::milter_builder::MilterBuilder;
    /// use rmilter::message_handler::MessageHandler;
    /// use rmilter::milter_message::MilterProtocol;
    ///
    /// struct MyHandler;
    /// impl MessageHandler for MyHandler {}
    ///
    /// let mut handler = MyHandler {};
    /// let protocol = MilterProtocol::default();
    ///
    /// let mut milter = MilterBuilder::new(&mut handler)
    ///     .set_protocol(protocol)
    ///     .build();
    /// ```
    pub fn set_protocol(self, protocol: MilterProtocol) -> Self {
        Self {
            protocol: Some(protocol),
            ..self
        }
    }
}
