#![allow(dead_code)]

mod ui;

use gpui_icons::LucideAssetSource;
use ui::{
    button::Button,
    checkbox::Checkbox,
    dialog::dialog_root,
    theme::UiTheme,
};

fn main() {
    let _assets = LucideAssetSource;
    let _theme = UiTheme::neutral_light();
    let _button = Button::new("fixture.button");
    let _checkbox = Checkbox::new("fixture.checkbox");
    let _dialog = dialog_root("fixture.dialog");
}
