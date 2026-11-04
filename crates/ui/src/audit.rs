//! Accessible-name audits, available with the `test-support` feature.
//!
//! Every interactive gpuicn component registers with GPUI Kit's element
//! queries, carrying the role and accessible name it gives assistive
//! technology. [`unnamed_controls`] lists the ones with no name, for a
//! headless test; [`A11yAuditOverlay`] outlines them in a running window, for
//! a debug build. Both read the last completed frame.
//!
//! ```ignore
//! // A test: every control in the main window has a name.
//! cx.update(|window, _| assert!(gpuicn::audit::unnamed_controls(window).is_empty()));
//!
//! // A debug build: outline unnamed controls when MATTY_A11Y_AUDIT is set.
//! div().child(main_view).when(std::env::var_os("MATTY_A11Y_AUDIT").is_some(), |root| {
//!     root.child(gpuicn::audit::A11yAuditOverlay::new())
//! })
//! ```

use gpui_kit::{
    App, Bounds, Element, ElementId, GlobalElementId, InspectorElementId, IntoElement, LayoutId,
    Pixels, Role, Window, test::ElementSnapshot, test::TestQueryExt as _,
};

use crate::theme::UiTheme;

/// Roles a person operates rather than reads.
const INTERACTIVE: &[Role] = &[
    Role::Button,
    Role::CheckBox,
    Role::ComboBox,
    Role::Link,
    Role::ListBoxOption,
    Role::MenuItem,
    Role::MenuItemCheckBox,
    Role::MenuItemRadio,
    Role::RadioButton,
    Role::Slider,
    Role::SpinButton,
    Role::Splitter,
    Role::Switch,
    Role::Tab,
    Role::TextInput,
    Role::MultilineTextInput,
    Role::SearchInput,
    Role::TreeItem,
];

/// Whether an observed element is something a person operates.
pub fn is_interactive(element: &ElementSnapshot) -> bool {
    element
        .role()
        .is_some_and(|role| INTERACTIVE.contains(&role))
}

/// The interactive elements of the last frame whose accessible name is
/// missing or blank, in reading order (top to bottom, then left to right).
pub fn unnamed_controls(window: &Window) -> Vec<ElementSnapshot> {
    window
        .elements()
        .into_iter()
        .filter(|element| {
            is_interactive(element) && element.label().is_none_or(|label| label.trim().is_empty())
        })
        .collect()
}

/// A readable path for an element made of the IDs callers chose, such as
/// `nav › general › button`. Views and component type frames are left out.
pub fn element_path(element: &ElementSnapshot) -> String {
    element
        .path()
        .iter()
        .filter_map(|id| match id {
            ElementId::Name(name) if !name.contains("::") => Some(name.to_string()),
            ElementId::NamedChild(_, name) => Some(name.to_string()),
            ElementId::NamedInteger(name, _) => Some(name.to_string()),
            ElementId::Integer(index) => Some(index.to_string()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" › ")
}

/// A debug overlay that outlines every unnamed interactive element in the
/// destructive color, and logs their paths to stderr whenever the set changes.
///
/// Add it as the last child of a window's root. It takes no space and no
/// input; it paints after the rest of the frame.
pub struct A11yAuditOverlay {
    log: bool,
}

impl Default for A11yAuditOverlay {
    fn default() -> Self {
        Self { log: true }
    }
}

impl A11yAuditOverlay {
    /// Creates the overlay, logging on.
    pub fn new() -> Self {
        Self::default()
    }
    /// Turns the stderr log on or off; the outlines always draw.
    pub fn log(mut self, log: bool) -> Self {
        self.log = log;
        self
    }
}

impl IntoElement for A11yAuditOverlay {
    type Element = gpui_kit::Deferred;
    fn into_element(self) -> Self::Element {
        gpui_kit::deferred(Overlay { log: self.log }).with_priority(usize::MAX)
    }
}

struct Overlay {
    log: bool,
}

impl IntoElement for Overlay {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}

impl Element for Overlay {
    type RequestLayoutState = ();
    type PrepaintState = ();
    fn id(&self) -> Option<ElementId> {
        Some("gpuicn-a11y-audit".into())
    }
    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }
    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        let style = gpui_kit::Style {
            position: gpui_kit::Position::Absolute,
            ..Default::default()
        };
        (window.request_layout(style, None, cx), ())
    }
    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        _: &mut Window,
        _: &mut App,
    ) {
    }
    fn paint(
        &mut self,
        id: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        let color = UiTheme::read(cx).colors.destructive;
        let unnamed = unnamed_controls(window);
        for element in &unnamed {
            if element.visible() {
                window.paint_quad(gpui_kit::outline(
                    element.bounds(),
                    color,
                    gpui_kit::BorderStyle::Solid,
                ));
            }
        }
        if self.log
            && let Some(id) = id
        {
            let paths: Vec<String> = unnamed.iter().map(element_path).collect();
            window.with_element_state(id, |last: Option<Vec<String>>, _| {
                if last.as_ref() != Some(&paths) {
                    for path in &paths {
                        eprintln!("gpuicn a11y audit: unnamed control at {path}");
                    }
                }
                ((), paths)
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_kit::{
        Context, IntoElement, ParentElement as _, Render, TestAppContext, div, prelude::*,
    };

    struct Toolbar;
    impl Render for Toolbar {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .flex()
                .child(crate::button::Button::new("save").label("Save"))
                .child(crate::button::Button::new("add").child("+"))
                .child(
                    crate::button::Button::new("blank")
                        .aria_label("  ")
                        .child("?"),
                )
                .child(A11yAuditOverlay::new().log(false))
        }
    }

    #[gpui_kit::test]
    fn unnamed_controls_lists_icon_only_buttons(cx: &mut TestAppContext) {
        cx.update(crate::init);
        let (_, cx) = cx.add_window_view(|_, _| Toolbar);
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.update(|window, _| {
            let unnamed: Vec<_> = unnamed_controls(window).iter().map(element_path).collect();
            assert_eq!(unnamed, ["add", "blank"]);
        });
    }
}
