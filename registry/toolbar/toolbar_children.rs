// Toolbar composition adapted from Base GPUI 64b22337.
// MIT License
//
// Copyright (c) 2026 Luke Tandjung
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use gpui::{AnyElement, IntoElement};

use super::{ToolbarButton, ToolbarGroup, ToolbarInput, ToolbarLink, ToolbarSeparator};

/// Typed direct children of `ToolbarRoot`. Buttons, links, and inputs occupy
/// roving slots; groups and separators do not. Designed to gain
/// `Toggle(...)` and `ToggleGroup(...)` variants later: a nested ToggleGroup
/// contributes its child toggles as individual flattened toolbar items (one
/// roving slot per toggle, none for the group container). No `AnyElement`
/// escape hatch for now; trigger-hosting wrappers are deferred until
/// menu/select trigger hosting is designed.
pub enum ToolbarChild {
    Button(ToolbarButton),
    Link(ToolbarLink),
    Input(ToolbarInput),
    Group(ToolbarGroup),
    Separator(ToolbarSeparator),
}

impl IntoElement for ToolbarChild {
    type Element = AnyElement;

    fn into_element(self) -> Self::Element {
        match self {
            Self::Button(button) => button.into_any_element(),
            Self::Link(link) => link.into_any_element(),
            Self::Input(input) => input.into_any_element(),
            Self::Group(group) => group.into_any_element(),
            Self::Separator(separator) => separator.into_any_element(),
        }
    }
}

impl From<ToolbarButton> for ToolbarChild {
    fn from(value: ToolbarButton) -> Self {
        Self::Button(value)
    }
}

impl From<ToolbarLink> for ToolbarChild {
    fn from(value: ToolbarLink) -> Self {
        Self::Link(value)
    }
}

impl From<ToolbarInput> for ToolbarChild {
    fn from(value: ToolbarInput) -> Self {
        Self::Input(value)
    }
}

impl From<ToolbarGroup> for ToolbarChild {
    fn from(value: ToolbarGroup) -> Self {
        Self::Group(value)
    }
}

impl From<ToolbarSeparator> for ToolbarChild {
    fn from(value: ToolbarSeparator) -> Self {
        Self::Separator(value)
    }
}

/// Typed children of `ToolbarGroup`. Group children register directly as
/// toolbar items with flattened indices; the group container occupies no
/// roving slot. Designed to gain a `Toggle(...)` variant later.
pub enum ToolbarGroupChild {
    Button(ToolbarButton),
    Link(ToolbarLink),
    Input(ToolbarInput),
}

impl IntoElement for ToolbarGroupChild {
    type Element = AnyElement;

    fn into_element(self) -> Self::Element {
        match self {
            Self::Button(button) => button.into_any_element(),
            Self::Link(link) => link.into_any_element(),
            Self::Input(input) => input.into_any_element(),
        }
    }
}

impl From<ToolbarButton> for ToolbarGroupChild {
    fn from(value: ToolbarButton) -> Self {
        Self::Button(value)
    }
}

impl From<ToolbarLink> for ToolbarGroupChild {
    fn from(value: ToolbarLink) -> Self {
        Self::Link(value)
    }
}

impl From<ToolbarInput> for ToolbarGroupChild {
    fn from(value: ToolbarInput) -> Self {
        Self::Input(value)
    }
}
