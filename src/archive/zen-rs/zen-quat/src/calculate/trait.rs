use zen_core::objects::enums::Direction;
use zen_core::objects::trade::Signal;
use zen_core::{Bar, CZSC};

pub(crate) trait Processor {
    fn process(
        &mut self,
        czsc: &CZSC,
        is_new: bool,
        start: Option<(Bar, Direction)>,
    ) -> Vec<Signal>;
}
