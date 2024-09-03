use anstyle::{AnsiColor, Style};
use clap::builder::Styles;

pub mod argument_from_cli;
pub mod install_mode;
pub mod parsers;
pub mod subnet_selection_opt;

pub fn style() -> Styles {
    let green = Style::new().fg_color(Some(AnsiColor::Green.into()));
    let yellow = Style::new().fg_color(Some(AnsiColor::Yellow.into()));
    let red = Style::new()
        .fg_color(Some(AnsiColor::BrightRed.into()))
        .bold();
    Styles::styled()
        .literal(green)
        .placeholder(green)
        .error(red)
        .header(yellow)
        .invalid(yellow)
        .valid(green)
}
