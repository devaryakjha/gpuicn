//! Open-code shadcn visual ports for GPUI.
//!
//! Every installed component root takes a caller-owned [`gpui::ElementId`].
//! Keep IDs stable across renders and unique among siblings. Use semantic paths
//! such as `settings.save`; never derive an ID from visible text, list position,
//! a pointer, or a random value. Changing an ID resets GPUI's keyed state.

#[path = "../../../registry/accordion/accordion.rs"]
pub mod accordion;
#[path = "../../../registry/alert_dialog/alert_dialog.rs"]
pub mod alert_dialog;
#[path = "../../../registry/autocomplete/autocomplete.rs"]
pub mod autocomplete;
#[path = "../../../registry/avatar/avatar.rs"]
pub mod avatar;
#[path = "../../../registry/button/button.rs"]
pub mod button;
#[path = "../../../registry/checkbox/checkbox.rs"]
pub mod checkbox;
#[path = "../../../registry/checkbox_group/checkbox_group.rs"]
pub mod checkbox_group;
#[path = "../../../registry/collapsible/collapsible.rs"]
pub mod collapsible;
#[path = "../../../registry/combobox/combobox.rs"]
pub mod combobox;
#[path = "../../../registry/context_menu/context_menu.rs"]
pub mod context_menu;
#[path = "../../../registry/dialog/dialog.rs"]
pub mod dialog;
#[path = "../../../registry/drawer/drawer.rs"]
pub mod drawer;
#[path = "../../../registry/field/field.rs"]
pub mod field;
#[path = "../../../registry/fieldset/fieldset.rs"]
pub mod fieldset;
#[path = "../../../registry/form/form.rs"]
pub mod form;
#[path = "../../../registry/input/input.rs"]
pub mod input;
#[path = "../../../registry/menu/menu.rs"]
pub mod menu;
#[path = "../../../registry/menubar/menubar.rs"]
pub mod menubar;
#[path = "../../../registry/meter/meter.rs"]
pub mod meter;
#[path = "../../../registry/navigation_menu/navigation_menu.rs"]
pub mod navigation_menu;
#[path = "../../../registry/number_field/number_field.rs"]
pub mod number_field;
#[path = "../../../registry/otp_field/otp_field.rs"]
pub mod otp_field;
#[path = "../../../registry/popover/popover.rs"]
pub mod popover;
#[path = "../../../registry/preview_card/preview_card.rs"]
pub mod preview_card;
#[path = "../../../registry/progress/progress.rs"]
pub mod progress;
#[path = "../../../registry/radio_group/radio_group.rs"]
pub mod radio_group;
#[path = "../../../registry/scroll_area/scroll_area.rs"]
pub mod scroll_area;
#[path = "../../../registry/select/select.rs"]
pub mod select;
#[path = "../../../registry/separator/separator.rs"]
pub mod separator;
#[path = "../../../registry/slider/slider.rs"]
pub mod slider;
#[path = "../../../registry/switch/switch.rs"]
pub mod switch;
#[path = "../../../registry/tabs/tabs.rs"]
pub mod tabs;
#[path = "../../../registry/theme/theme.rs"]
pub mod theme;
#[path = "../../../registry/toast/toast.rs"]
pub mod toast;
#[path = "../../../registry/toggle/toggle.rs"]
pub mod toggle;
#[path = "../../../registry/toggle_group/toggle_group.rs"]
pub mod toggle_group;
#[path = "../../../registry/toolbar/toolbar.rs"]
pub mod toolbar;
#[path = "../../../registry/tooltip/tooltip.rs"]
pub mod tooltip;

pub use button::{Button, ButtonSize, ButtonVariant};
pub use checkbox::Checkbox;
pub use theme::{ThemeMode, UiColors, UiFonts, UiRadius, UiSpacing, UiTheme};

/// Registers every Base GPUI action used by the component catalog.
pub fn init(cx: &mut gpui::App) {
    base_gpui::init(cx);
}
