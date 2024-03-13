use zen_core::objects::trade::Signal;
use zen_core::CZSC;

pub(crate) trait Processor {
    fn process(&mut self, czsc: &CZSC) -> Vec<Signal>;
}
