use lazy_static::lazy_static;
use tuirealm::props::{Color, TextSpan};

use crate::display::tui::types::TextSpans;

lazy_static! {
    pub static ref UNRATED_COLOR: Color = Color::Rgb(0x00, 0x00, 0x00);
    pub static ref NEWBIE_COLOR: Color = Color::Rgb(0x80, 0x80, 0x80);
    pub static ref PUPIL_COLOR: Color = Color::Rgb(0x00, 0x80, 0x00);
    pub static ref SPECIALIST_COLOR: Color = Color::Rgb(0x03, 0xa8, 0x9e);
    pub static ref EXPERT_COLOR: Color = Color::Rgb(0x00, 0x00, 0xff);
    pub static ref CANDIDATE_MASTER_COLOR: Color = Color::Rgb(0xaa, 0x00, 0xaa);
    pub static ref MASTER_COLOR: Color = Color::Rgb(0xff, 0x8c, 0x00);
    pub static ref INTERNATIONAL_MASTER_COLOR: Color = Color::Rgb(0xff, 0x8c, 0x00);
    pub static ref GRANDMASTER_COLOR: Color = Color::Rgb(0xff, 0x00, 0x00);
    pub static ref INTERNATIONAL_GRANDMASTER_COLOR: Color = Color::Rgb(0xff, 0x00, 0x00);
    pub static ref LEGENDARY_GRANDMASTER_COLOR: Color = Color::Rgb(0x00, 0x00, 0x00);
}

pub fn colorful_handle(handle: String, rating: i32) -> TextSpans {
    if rating >= 0 && rating < 1200 {
        TextSpans::from(vec![TextSpan::new(handle).fg(*NEWBIE_COLOR)])
    } else if rating >= 1200 && rating < 1400 {
        TextSpans::from(vec![TextSpan::new(handle).fg(*PUPIL_COLOR)])
    } else if rating >= 1400 && rating < 1600 {
        TextSpans::from(vec![TextSpan::new(handle).fg(*SPECIALIST_COLOR)])
    } else if rating >= 1600 && rating < 1900 {
        TextSpans::from(vec![TextSpan::new(handle).fg(*EXPERT_COLOR)])
    } else if rating >= 1900 && rating < 2100 {
        TextSpans::from(vec![TextSpan::new(handle).fg(*CANDIDATE_MASTER_COLOR)])
    } else if rating >= 2100 && rating < 2300 {
        TextSpans::from(vec![TextSpan::new(handle).fg(*MASTER_COLOR)])
    } else if rating >= 2300 && rating < 2400 {
        TextSpans::from(vec![TextSpan::new(handle).fg(*INTERNATIONAL_MASTER_COLOR)])
    } else if rating >= 2400 && rating < 2600 {
        TextSpans::from(vec![TextSpan::new(handle).fg(*GRANDMASTER_COLOR)])
    } else if rating >= 2600 && rating < 3000 {
        TextSpans::from(vec![
            TextSpan::new(handle).fg(*INTERNATIONAL_GRANDMASTER_COLOR)
        ])
    } else if rating > 3000 {
        let (first, rest) = handle.split_at(1);
        TextSpans::from(vec![
            TextSpan::new(first).fg(*LEGENDARY_GRANDMASTER_COLOR),
            TextSpan::new(rest).fg(*GRANDMASTER_COLOR),
        ])
    } else {
        TextSpans::from(vec![TextSpan::new(handle).fg(*UNRATED_COLOR)])
    }
}
