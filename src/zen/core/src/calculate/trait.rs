use crate::element::enums::Direction;
use crate::element::event::Signal;
use crate::element::chan::{Bar};
use crate::analyze::CZSC;

pub(crate) trait Processor {
    fn process(
        &mut self,
        czsc: &CZSC,
        is_new: bool,
        start: Option<(Bar, Direction)>,
    ) -> Vec<Signal>;
}
