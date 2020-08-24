use crate::accept_reject_action::AcceptRejectAction;
use crate::milter_message::{MilterMacro, ProtocolFamily};

pub trait MessageHandler {
    fn abort_filter_checks(&mut self) {}

    fn body_chunk(&mut self, _value: &str) -> AcceptRejectAction {
        AcceptRejectAction::Continue
    }

    fn connection(
        &mut self,
        _hostname: &str,
        _family: &ProtocolFamily,
        _port: &u16,
        _address: &str,
    ) -> AcceptRejectAction {
        AcceptRejectAction::Continue
    }

    fn define_macros(&mut self, _cmdcode: &char, _macros: Vec<MilterMacro>) {
    }

    fn end_of_body(&mut self) -> AcceptRejectAction {
        // TODO: Add support for modifying here (header, body, recipients, etc.)
        AcceptRejectAction::Continue
    }

    fn end_of_header(&mut self) -> AcceptRejectAction {
        AcceptRejectAction::Continue
    }

    fn header(&mut self, _name: &str, _value: &str) -> AcceptRejectAction {
        AcceptRejectAction::Continue
    }

    fn helo(&mut self, _msg: &str) -> AcceptRejectAction {
        AcceptRejectAction::Continue
    }

    fn mail_from(&mut self, _address: &str, _args: &[String]) -> AcceptRejectAction {
        AcceptRejectAction::Continue
    }

    fn recipient(&mut self, _recipient: &str, _args: &[String]) -> AcceptRejectAction {
        AcceptRejectAction::Continue
    }
}
