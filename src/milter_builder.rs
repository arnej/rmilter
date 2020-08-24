use crate::message_handler::MessageHandler;
use crate::milter::Milter;

pub struct MilterBuilder {}

impl Default for MilterBuilder {
    fn default() -> Self {
        Self {}
    }
}

impl<'a> MilterBuilder {
    pub fn set_message_handler(
        self,
        message_handler: &'a mut impl MessageHandler,
    ) -> MilterBuilderWithHandler<'a> {
        MilterBuilderWithHandler { message_handler }
    }
}

pub struct MilterBuilderWithHandler<'a> {
    message_handler: &'a mut dyn MessageHandler,
}

impl<'a> MilterBuilderWithHandler<'a> {
    pub fn build(self) -> Milter<'a> {
        Milter::new(self.message_handler)
    }
}
