pub mod colors {
    use ratatui::style::{
        palette::tailwind::{RED, BLUE, LIME},
        Color,
    };

    pub const STATUS_BAR_INTERACTION_COLOR: Color = RED.c300;
    pub const STATUS_BAR_NAVIGATION_COLOR: Color = BLUE.c300;
    pub const STATUS_BAR_NAVIGATION_INPUT_COLOR: Color = LIME.c300;
}
