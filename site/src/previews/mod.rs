//! One file per catalog preview. Adding a component means adding its file
//! here and one line to `previews!`; `cargo xtask new-component` does both.

pub(crate) mod prelude {
    //! Everything a preview file needs besides `crate::*`.
    #![allow(unused_imports)]
    pub(crate) use gpui_icons::{LucideAssetSource, LucideIcon, lucide};
    pub(crate) use gpui_kit::{
        App, AppContext as _, Application, Bounds, Context, Entity, InteractiveElement as _,
        IntoElement, ParentElement as _, Render, SharedString, StatefulInteractiveElement as _,
        Styled as _, Window, WindowBounds, WindowOptions, div, prelude::FluentBuilder as _, px,
        size,
    };
    pub(crate) use gpuicn::context_menu::context_menu;
    pub(crate) use gpuicn::{
        Button, ButtonSize, ButtonVariant, Checkbox, ThemeMode, UiTheme,
        accordion::*,
        alert_dialog::*,
        attachment::{Attachment, AttachmentStatus, attachment_group},
        avatar::{Avatar, AvatarSize, AvatarStatus},
        badge::{Badge, BadgeVariant},
        carousel::*,
        checkbox_group::*,
        collapsible::*,
        command::{CommandEvent, CommandState, command_dialog_popup},
        composer::{Composer, ComposerEvent, ComposerState, ComposerSuggestion},
        dialog::*,
        drawer::{
            Drawer, DrawerContent, DrawerSide, drawer_body, drawer_close, drawer_description,
            drawer_footer, drawer_header, drawer_title, drawer_trigger,
        },
        emoji_picker::{Emoji, EmojiCategory, EmojiPicker, EmojiPickerEvent, EmojiPickerState},
        empty::*,
        field::*,
        fieldset::*,
        form::form,
        input::{Input, InputEvent, InputState},
        input_group::*,
        kbd::{Kbd, kbd_group},
        lightbox::{LightboxImage, LightboxState, lightbox},
        markdown::{CustomEmoji, EntityLinks, MarkdownImages, selection_root},
        menu::{Menu, MenuItem, MenuState},
        menubar::Menubar,
        message::{
            Message, MessageDensity, MessageHighlight, MessageStatus, Reaction, day_divider,
            message_action, message_actions, new_messages_divider,
        },
        message_list::{MessageList, MessageListState},
        meter::Meter,
        navigation_menu::navigation_menu,
        number_field::NumberField,
        otp_field::{OtpField, OtpState},
        popover::*,
        preview_card::*,
        progress::Progress,
        radio_group::*,
        scroll_area::ScrollArea,
        select::{Select, SelectItem, SelectState},
        separator::Separator,
        skeleton::Skeleton,
        slider::Slider,
        sortable::{Sortable, SortableGroup, SortableRow},
        spinner::Spinner,
        switch::Switch,
        tabs::*,
        tag::{Tag, TagVariant},
        textarea::*,
        title_bar::TitleBar,
        toast::{Notice, ToastState},
        toggle::{Toggle, ToggleVariant},
        toggle_group::*,
        toolbar::{Toolbar, ToolbarButton},
        tooltip::*,
    };
    pub(crate) use std::borrow::Cow;
}

pub(crate) mod button;

use crate::Showcase;
use gpui_kit::{Context, IntoElement, Window};

macro_rules! previews {
    ($($module:ident => $variant:ident,)*) => {
        $(pub(crate) mod $module;)*

        /// The catalog entry the showcase renders.
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
        pub(crate) enum Demo {
            Icons,
            #[default]
            Button,
            $($variant,)*
        }

        impl Demo {
            /// Every preview, in catalog order.
            pub(crate) const ALL: &[Demo] = &[Demo::Icons, Demo::Button, $(Demo::$variant,)*];

            /// The registry name, such as `alert-dialog`.
            pub(crate) fn slug(self) -> &'static str {
                match self {
                    Demo::Icons => "icons",
                    Demo::Button => "button",
                    $(Demo::$variant => stringify!($module),)*
                }
            }

            /// Parses a registry name (`alert-dialog`) or module name (`alert_dialog`).
            pub(crate) fn parse(value: &str) -> Option<Self> {
                let value = value.replace('-', "_");
                Self::ALL.iter().copied().find(|demo| demo.slug() == value)
            }
        }

        impl Showcase {
            pub(crate) fn preview(&self, window: &mut Window, cx: &mut Context<Self>) -> gpui_kit::AnyElement {
                match self.demo {
                    Demo::Icons => self.icons_preview(cx).into_any_element(),
                    Demo::Button => button::preview(self, window, cx),
                    $(Demo::$variant => $module::preview(self, window, cx),)*
                }
            }
        }
    };
}

impl Demo {
    /// Previews that need a 960×600 window instead of 640×288.
    pub(crate) fn is_wide(self) -> bool {
        matches!(self, Demo::Sidebar | Demo::VirtualList | Demo::MessageList)
    }

    /// Previews that fill the window without the 16px frame.
    pub(crate) fn is_edge_to_edge(self) -> bool {
        self.is_wide() || self == Demo::Message
    }
}

previews! {
    accordion => Accordion,
    alert => Alert,
    alert_dialog => AlertDialog,
    app => App,
    attachment => Attachment,
    autocomplete => Autocomplete,
    avatar => Avatar,
    avatar_group => AvatarGroup,
    badge => Badge,
    calendar => Calendar,
    carousel => Carousel,
    checkbox => Checkbox,
    checkbox_group => CheckboxGroup,
    collapsible => Collapsible,
    color_picker => ColorPicker,
    combobox => Combobox,
    command => Command,
    composer => Composer,
    context_menu => ContextMenu,
    date_picker => DatePicker,
    date_time_picker => DateTimePicker,
    dialog => Dialog,
    drawer => Drawer,
    embed => Embed,
    emoji_picker => EmojiPicker,
    empty => Empty,
    field => Field,
    fieldset => Fieldset,
    form => Form,
    input => Input,
    input_group => InputGroup,
    kbd => Kbd,
    lightbox => Lightbox,
    markdown => Markdown,
    menu => Menu,
    menubar => Menubar,
    message => Message,
    message_list => MessageList,
    meter => Meter,
    navigation_menu => NavigationMenu,
    number_field => NumberField,
    otp_field => OtpField,
    popover => Popover,
    preview_card => PreviewCard,
    progress => Progress,
    radio_group => RadioGroup,
    resizable => Resizable,
    scroll_area => ScrollArea,
    select => Select,
    separator => Separator,
    sidebar => Sidebar,
    skeleton => Skeleton,
    slider => Slider,
    sortable => Sortable,
    spinner => Spinner,
    switch => Switch,
    tabs => Tabs,
    tag => Tag,
    textarea => Textarea,
    title_bar => TitleBar,
    toast => Toast,
    toggle => Toggle,
    toggle_group => ToggleGroup,
    toolbar => Toolbar,
    tooltip => Tooltip,
    virtual_list => VirtualList,
}
