/// Defines the accept/reject actions that the milter returns for each step during the processing
/// flow.
pub enum AcceptRejectAction {
    /// Accept the message without further processing
    Accept,
    /// Continue processing the message
    Continue,
    /// Silently discard the message without further processing
    Discard,
    /// Reject the message without further processing
    Reject,
    /// Temporarily fail without further processing
    Tempfail,
    // TODO ReplyCode
}
