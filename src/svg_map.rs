use gpui::{black, svg, Length, Styled};

pub enum IconName {
    QuestionMark,
}

impl IconName {
    pub fn path(&self) -> &'static str {
        match self {
            Self::QuestionMark => "assets/svg/Q.svg",
        }
    }
}

pub struct Icon {}

impl Icon {
    pub fn new(icon: IconName) -> gpui::Svg {
        svg()
            .text_color(black())
            .w(Length::Definite(gpui::DefiniteLength::Fraction(
                200.0.into(),
            )))
            .h_12()
            .path(icon.path())
    }
}
