use crate::element::enums::Direction;
use crate::element::event::Signal;
use crate::element::chan::{Bar};
use crate::analyze::CZSC;


// 通过中枢、笔，计算浪型，或者结构饱满程度
// 五浪过后，有背驰容易反转
fn wave_zs_count(
    czsc: &mut CZSC,
    parent_czsc: &mut CZSC,
    is_new: bool,
    start: Option<(Bar, Direction)>,
) -> Vec<Signal> {
    vec![]
}

// 时钟方向，一点、五点方向很难长时间保持
// 斜率角度太大，不容易维持
// 出现分型时计算
