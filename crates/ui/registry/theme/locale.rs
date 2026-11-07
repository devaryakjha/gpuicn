//! Layout direction and the built-in strings components draw.
//!
//! Components read the [`Direction`] from [`UiTheme::direction`] and mirror
//! rows, insets, chevrons, popup sides and arrow keys in right-to-left
//! layouts. Change it with [`set_direction`], which also keeps it across
//! theme swaps.
//!
//! Every word a component draws or announces on its own, such as "Close" or
//! "Search emoji", is a [`UiText`]. [`set_strings`] installs the
//! application's translations; anything it leaves out stays English. Text the
//! application passes in, such as a button label, is never translated here.

use std::rc::Rc;

use gpui_kit::{App, Global, SharedString, Styled, TextAlign, px};

use super::UiTheme;

/// The order in which a layout reads: left to right or right to left.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum Direction {
    /// Left to right, as in English.
    #[default]
    Ltr,
    /// Right to left, as in Arabic, Hebrew, Persian and Urdu.
    Rtl,
}

impl Direction {
    /// Whether this is right to left.
    pub fn is_rtl(self) -> bool {
        self == Self::Rtl
    }

    /// The text alignment of the reading start edge.
    pub fn start_align(self) -> TextAlign {
        match self {
            Self::Ltr => TextAlign::Left,
            Self::Rtl => TextAlign::Right,
        }
    }

    /// The keystroke key with `left` and `right` swapped in right-to-left
    /// layouts, so a handler written for left to right moves toward the
    /// reading start on the start-side arrow. Other keys pass through.
    pub fn logical_key(self, key: &str) -> &str {
        match (self, key) {
            (Self::Rtl, "left") => "right",
            (Self::Rtl, "right") => "left",
            _ => key,
        }
    }

    /// The Kit placement mirrored for this direction: Left and Right swap in
    /// right-to-left layouts, Top and Bottom stay.
    pub fn placement(self, placement: gpui_kit::base::Placement) -> gpui_kit::base::Placement {
        use gpui_kit::base::Placement;
        match (self, placement) {
            (Self::Rtl, Placement::Left) => Placement::Right,
            (Self::Rtl, Placement::Right) => Placement::Left,
            _ => placement,
        }
    }

    /// The Kit alignment mirrored for this direction along a horizontal edge:
    /// Start and End swap in right-to-left layouts.
    pub fn align(self, align: gpui_kit::base::Align) -> gpui_kit::base::Align {
        use gpui_kit::base::Align;
        match (self, align) {
            (Self::Rtl, Align::Start) => Align::End,
            (Self::Rtl, Align::End) => Align::Start,
            _ => align,
        }
    }

    /// The popup corner mirrored for this direction: left and right corners
    /// swap in right-to-left layouts.
    pub fn anchor(self, anchor: gpui_kit::Anchor) -> gpui_kit::Anchor {
        use gpui_kit::Anchor;
        if !self.is_rtl() {
            return anchor;
        }
        match anchor {
            Anchor::TopLeft => Anchor::TopRight,
            Anchor::TopRight => Anchor::TopLeft,
            Anchor::BottomLeft => Anchor::BottomRight,
            Anchor::BottomRight => Anchor::BottomLeft,
            Anchor::LeftCenter => Anchor::RightCenter,
            Anchor::RightCenter => Anchor::LeftCenter,
            other => other,
        }
    }

    /// The chevron or arrow pointing toward the reading end (`forward`) or
    /// start, from the left-to-right `right` icon and `left` icon.
    pub fn icon<T>(self, forward: bool, left: T, right: T) -> T {
        if forward == self.is_rtl() {
            left
        } else {
            right
        }
    }
}

/// The key context a right-to-left text input adds around Kit's `Input`, so
/// the arrow keys move toward the visual side they point to.
#[doc(hidden)]
pub const RTL_INPUT_CONTEXT: &str = "GpuicnRtl";

/// Binds the mirrored arrow keys of right-to-left text inputs.
#[doc(hidden)]
pub fn bind_keys(cx: &mut App) {
    use gpui_kit::KeyBinding;
    use gpui_kit::base::actions::{SelectLeft, SelectRight};
    use gpui_kit::base::input::{
        MoveLeft, MoveRight, MoveToNextWord, MoveToPreviousWord, SelectToNextWordEnd,
        SelectToPreviousWordStart,
    };
    let context = Some("GpuicnRtl > Input");
    // Word movement uses Alt on macOS and Ctrl elsewhere, as in Kit.
    let words = if cfg!(target_os = "macos") {
        ["alt-left", "alt-right", "alt-shift-left", "alt-shift-right"]
    } else {
        [
            "ctrl-left",
            "ctrl-right",
            "ctrl-shift-left",
            "ctrl-shift-right",
        ]
    };
    cx.bind_keys([
        KeyBinding::new("left", MoveRight, context),
        KeyBinding::new("right", MoveLeft, context),
        KeyBinding::new("shift-left", SelectRight, context),
        KeyBinding::new("shift-right", SelectLeft, context),
        KeyBinding::new(words[0], MoveToNextWord, context),
        KeyBinding::new(words[1], MoveToPreviousWord, context),
        KeyBinding::new(words[2], SelectToNextWordEnd, context),
        KeyBinding::new(words[3], SelectToPreviousWordStart, context),
    ]);
}

#[derive(Clone, Copy, Default)]
struct DirectionState(Direction);
impl Global for DirectionState {}

/// Sets the layout direction for every window and redraws them.
pub fn set_direction(cx: &mut App, direction: Direction) {
    cx.set_global(DirectionState(direction));
    if cx.has_global::<UiTheme>() {
        let mut theme = UiTheme::read(cx).clone();
        theme.direction = direction;
        UiTheme::set(cx, theme);
    }
}

/// The layout direction, left to right until [`set_direction`] says otherwise.
pub fn direction(cx: &App) -> Direction {
    cx.try_global::<DirectionState>()
        .map(|state| state.0)
        .unwrap_or_default()
}

/// Wraps text the application supplies, such as a person's name, in Unicode
/// first-strong isolates, so a right-to-left name does not reorder the
/// punctuation and words around it, and a left-to-right name inside
/// right-to-left text keeps its own order.
pub fn isolate(text: impl AsRef<str>) -> SharedString {
    let text = text.as_ref();
    if text.is_empty() || text.starts_with('\u{2068}') {
        return SharedString::from(text.to_owned());
    }
    format!("\u{2068}{text}\u{2069}").into()
}

/// Logical inline-direction styles: start and end are left and right in
/// left-to-right layouts and swap in right-to-left ones.
pub trait DirectionalStyled: Styled + Sized {
    /// A flex row whose first child sits at the reading start.
    fn flex_row_in(self, direction: Direction) -> Self {
        let element = self.flex();
        match direction {
            Direction::Ltr => element.flex_row(),
            Direction::Rtl => element.flex_row_reverse(),
        }
    }

    /// Reverses an existing flex row in right-to-left layouts.
    fn mirror_row(self, direction: Direction) -> Self {
        if direction.is_rtl() {
            self.flex_row_reverse()
        } else {
            self
        }
    }

    /// Padding on the reading start edge.
    fn padding_start(mut self, direction: Direction, value: gpui_kit::Pixels) -> Self {
        let padding = &mut self.style().padding;
        match direction {
            Direction::Ltr => padding.left = Some(value.into()),
            Direction::Rtl => padding.right = Some(value.into()),
        }
        self
    }

    /// Padding on the reading end edge.
    fn padding_end(mut self, direction: Direction, value: gpui_kit::Pixels) -> Self {
        let padding = &mut self.style().padding;
        match direction {
            Direction::Ltr => padding.right = Some(value.into()),
            Direction::Rtl => padding.left = Some(value.into()),
        }
        self
    }

    /// Margin on the reading start edge.
    fn margin_start(mut self, direction: Direction, value: gpui_kit::Pixels) -> Self {
        let margin = &mut self.style().margin;
        match direction {
            Direction::Ltr => margin.left = Some(value.into()),
            Direction::Rtl => margin.right = Some(value.into()),
        }
        self
    }

    /// Margin on the reading end edge.
    fn margin_end(mut self, direction: Direction, value: gpui_kit::Pixels) -> Self {
        let margin = &mut self.style().margin;
        match direction {
            Direction::Ltr => margin.right = Some(value.into()),
            Direction::Rtl => margin.left = Some(value.into()),
        }
        self
    }

    /// Distance of a positioned element from the reading start edge.
    fn inset_start(mut self, direction: Direction, value: gpui_kit::Pixels) -> Self {
        let inset = &mut self.style().inset;
        match direction {
            Direction::Ltr => inset.left = Some(value.into()),
            Direction::Rtl => inset.right = Some(value.into()),
        }
        self
    }

    /// Distance of a positioned element from the reading end edge.
    fn inset_end(mut self, direction: Direction, value: gpui_kit::Pixels) -> Self {
        let inset = &mut self.style().inset;
        match direction {
            Direction::Ltr => inset.right = Some(value.into()),
            Direction::Rtl => inset.left = Some(value.into()),
        }
        self
    }

    /// A one-pixel border on the reading start edge.
    fn border_start_1(mut self, direction: Direction) -> Self {
        let border = &mut self.style().border_widths;
        match direction {
            Direction::Ltr => border.left = Some(px(1.).into()),
            Direction::Rtl => border.right = Some(px(1.).into()),
        }
        self
    }

    /// A one-pixel border on the reading end edge.
    fn border_end_1(mut self, direction: Direction) -> Self {
        let border = &mut self.style().border_widths;
        match direction {
            Direction::Ltr => border.right = Some(px(1.).into()),
            Direction::Rtl => border.left = Some(px(1.).into()),
        }
        self
    }

    /// Rounds the two corners on the reading start edge.
    fn rounded_start(mut self, direction: Direction, radius: gpui_kit::Pixels) -> Self {
        let corners = &mut self.style().corner_radii;
        let radius = Some(radius.into());
        match direction {
            Direction::Ltr => (corners.top_left, corners.bottom_left) = (radius, radius),
            Direction::Rtl => (corners.top_right, corners.bottom_right) = (radius, radius),
        }
        self
    }

    /// Rounds the two corners on the reading end edge.
    fn rounded_end(mut self, direction: Direction, radius: gpui_kit::Pixels) -> Self {
        let corners = &mut self.style().corner_radii;
        let radius = Some(radius.into());
        match direction {
            Direction::Ltr => (corners.top_right, corners.bottom_right) = (radius, radius),
            Direction::Rtl => (corners.top_left, corners.bottom_left) = (radius, radius),
        }
        self
    }

    /// A two-pixel border on the reading start edge, as on a highlighted row.
    fn border_start_2(mut self, direction: Direction) -> Self {
        let border = &mut self.style().border_widths;
        match direction {
            Direction::Ltr => border.left = Some(px(2.).into()),
            Direction::Rtl => border.right = Some(px(2.).into()),
        }
        self
    }

    /// Removes the border on the reading start edge.
    fn border_start_0(mut self, direction: Direction) -> Self {
        let border = &mut self.style().border_widths;
        match direction {
            Direction::Ltr => border.left = Some(px(0.).into()),
            Direction::Rtl => border.right = Some(px(0.).into()),
        }
        self
    }

    /// Text aligned to the reading start edge.
    fn text_start(self, direction: Direction) -> Self {
        self.text_align(direction.start_align())
    }
}

impl<T: Styled> DirectionalStyled for T {}

/// A word or sentence a component draws or announces on its own.
///
/// Match on it in [`set_strings`] to translate; [`UiText::key`] gives a stable
/// identifier for message catalogs such as Fluent, and the values a variant
/// carries are the arguments to place.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum UiText {
    /// "Close", on dialogs, drawers, lightboxes and window controls.
    Close,
    /// "Close notification", on toasts.
    CloseNotification,
    /// "Loading", the spinner's accessible name.
    Loading,
    /// "Read only", announced on read-only fields.
    ReadOnly,
    /// "{label} (required)", a required field's accessible name.
    Required(SharedString),
    /// "One-time code", the OTP field's default accessible name.
    OneTimeCode,
    /// "Invalid value", announced on invalid inputs.
    InvalidValue,
    /// "Unavailable", announced on disabled list rows and commands.
    Unavailable,
    /// "No results found.", the command palette's empty state.
    NoResults,
    /// "No items", the virtual list's empty state.
    NoItems,
    /// "Use arrow keys to navigate and Enter to activate.", the virtual list's hint.
    ListInstructions,
    /// The multi-select virtual list's hint.
    MultiSelectInstructions,
    /// "Selection", a select's default accessible name.
    Selection,
    /// "Select…", a select's default placeholder.
    SelectPlaceholder,
    /// "{label} (invalid value)", an invalid select's accessible name.
    InvalidSelection(SharedString),
    /// "Navigation", the sidebar's accessible name.
    Navigation,
    /// "Toggle sidebar".
    ToggleSidebar,
    /// "current destination", read after the selected sidebar item.
    CurrentDestination,
    /// "expanded", read after an open disclosure.
    Expanded,
    /// "collapsed", read after a closed disclosure.
    Collapsed,
    /// "Cut", in the text context menu.
    Cut,
    /// "Copy", in the text context menu.
    Copy,
    /// "Paste", in the text context menu.
    Paste,
    /// "Select All", in the text context menu.
    SelectAll,
    /// "Decrease value", on number fields.
    DecreaseValue,
    /// "Increase value", on number fields.
    IncreaseValue,
    /// "Minimize", a title bar window control.
    Minimize,
    /// "Maximize", a title bar window control.
    Maximize,
    /// "Restore", a title bar window control.
    Restore,
    /// "Uploading… {percent}%", an attachment's progress.
    Uploading(u32),
    /// "Upload failed".
    UploadFailed,
    /// "Remove {name}", on attachments and tags.
    Remove(SharedString),
    /// "Retry uploading {name}".
    RetryUpload(SharedString),
    /// "Previous month", on calendars.
    PreviousMonth,
    /// "Next month", on calendars.
    NextMonth,
    /// A weekday name, 0 for "Sunday" through 6 for "Saturday".
    Weekday(u8),
    /// A two-letter weekday heading, 0 for "Su" through 6 for "Sa".
    WeekdayShort(u8),
    /// A month name, 1 for "January" through 12 for "December".
    Month(u8),
    /// A calendar's heading, such as "March 2026".
    MonthYear {
        /// 1 to 12.
        month: u8,
        /// The year.
        year: i32,
    },
    /// A date read aloud, such as "Saturday, March 14, 2026".
    LongDate {
        /// 0 for Sunday through 6 for Saturday.
        weekday: u8,
        /// 1 to 12.
        month: u8,
        /// 1 to 31.
        day: u8,
        /// The year.
        year: i32,
    },
    /// A compact date, such as "Mar 14, 2026".
    ShortDate {
        /// 1 to 12.
        month: u8,
        /// 1 to 31.
        day: u8,
        /// The year.
        year: i32,
    },
    /// "today", appended to today's date in a calendar.
    Today,
    /// "Pick a date", the date picker's placeholder.
    PickDate,
    /// "Pick a date and time", the date and time picker's placeholder.
    PickDateTime,
    /// "Time, {time}", the time field's accessible name.
    TimeField(SharedString),
    /// "Hour".
    Hour,
    /// "AM", before noon on a 12-hour clock.
    Am,
    /// "PM", after noon on a 12-hour clock.
    Pm,
    /// "AM or PM", the time field's period segment.
    DayPeriod,
    /// "Minute".
    Minute,
    /// "Done", closing the date and time picker.
    Done,
    /// "Carousel", the carousel's default accessible name.
    Carousel,
    /// "Previous slide".
    PreviousSlide,
    /// "Next slide".
    NextSlide,
    /// "Search commands", the command palette's input.
    SearchCommands,
    /// "Commands", the command list and the composer's slash suggestions.
    Commands,
    /// "Command palette".
    CommandPalette,
    /// "Message", the composer's accessible name.
    Message,
    /// "People", the composer's @ suggestions.
    People,
    /// "Channels", the composer's ~ suggestions.
    Channels,
    /// "Emoji", the composer's : suggestions and the emoji grid.
    Emoji,
    /// "Suggestions", the composer's other suggestions.
    Suggestions,
    /// "Load image", on image placeholders.
    LoadImage,
    /// "Load image from {host}".
    LoadImageFrom(SharedString),
    /// "Expand preview", on link embeds.
    ExpandPreview,
    /// "Collapse preview", on link embeds.
    CollapsePreview,
    /// "Remove preview", on link embeds.
    RemovePreview,
    /// "Attachment", an untitled attachment embed.
    Attachment,
    /// "Jump", on quoted messages.
    Jump,
    /// "Jump to the message from {author}".
    JumpToMessageFrom(SharedString),
    /// "Originally posted in {channel}".
    OriginallyPostedIn(SharedString),
    /// "Show more", expanding long quotes.
    ShowMore,
    /// "Show less", collapsing long quotes.
    ShowLess,
    /// "Copy code", on a code block's copy button.
    CopyCode,
    /// "Copied", after a copy button copied.
    Copied,
    /// "Quoted message from {author}".
    QuotedMessageFrom(SharedString),
    /// "Search emoji", the emoji picker's placeholder and input name.
    SearchEmoji,
    /// "Search results", the emoji picker's results heading.
    SearchResults,
    /// "Frequently used", an emoji picker heading.
    FrequentlyUsed,
    /// "Recently used", an emoji picker heading.
    RecentlyUsed,
    /// "Custom", the emoji picker's custom emoji heading.
    CustomEmoji,
    /// "Emoji categories", the emoji picker's tab list.
    EmojiCategories,
    /// "No emoji found.", the emoji picker's empty state.
    NoEmoji,
    /// "Skin tone", the emoji picker's skin tone menu.
    SkinTone,
    /// A skin tone name, 0 for "Default skin tone" through 5 for "Dark skin tone".
    SkinToneName(u8),
    /// "Previous image", in the lightbox.
    PreviousImage,
    /// "Next image", in the lightbox.
    NextImage,
    /// "Image", an untitled image.
    Image,
    /// "Zoom out", in the lightbox.
    ZoomOut,
    /// "Zoom in", in the lightbox.
    ZoomIn,
    /// "Image viewer: {name}", the lightbox's accessible name.
    ImageViewer(SharedString),
    /// "{index} of {count}", the lightbox's position, counting from 1.
    Position {
        /// The current item, from 1.
        index: usize,
        /// How many there are.
        count: usize,
    },
    /// "Zoom {percent}%", the lightbox's zoom status.
    Zoom(u32),
    /// "Message failed to send.".
    MessageFailed,
    /// "Retry", resending a failed message.
    Retry,
    /// "Delete", discarding a failed message.
    Delete,
    /// "Add reaction".
    AddReaction,
    /// "Jump to latest", the message list's default jump label.
    JumpToLatest,
    /// "online", an avatar status.
    StatusOnline,
    /// "away", an avatar status.
    StatusAway,
    /// "do not disturb", an avatar status.
    StatusDnd,
    /// "offline", an avatar status.
    StatusOffline,
    /// A list of names, such as "Ana, Ben and Chen", for avatar stacks.
    Names(Vec<SharedString>),
    /// "Clear selection", on a searchable select.
    ClearSelection,
    /// "Show options", on a searchable select.
    ShowOptions,
    /// "Hue", the Color Picker's hue slider.
    ColorHue,
    /// "Chroma", the Color Picker's colorfulness slider.
    ColorChroma,
    /// "Lightness", the Color Picker's lightness slider.
    ColorLightness,
    /// How to move a sortable row with the keyboard; the keys are already
    /// in reading order.
    SortInstructions {
        /// The key that moves toward the start, such as "Up".
        previous: SharedString,
        /// The key that moves toward the end, such as "Down".
        next: SharedString,
    },
    /// A sortable group header's label with its move keys.
    SortGroup {
        /// The group's name.
        label: SharedString,
        /// The key that moves toward the start.
        previous: SharedString,
        /// The key that moves toward the end.
        next: SharedString,
    },
    /// The announcement after a keyboard move in a sortable list.
    Moved {
        /// The moved row or group.
        label: SharedString,
        /// The new place, counted from 1.
        position: usize,
        /// How many places there are.
        count: usize,
        /// The group the row moved into, if rows are grouped.
        group: Option<SharedString>,
    },
}

const WEEKDAYS: [&str; 7] = [
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
];
const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];
const SKIN_TONES: [&str; 6] = [
    "Default skin tone",
    "Light skin tone",
    "Medium-light skin tone",
    "Medium skin tone",
    "Medium-dark skin tone",
    "Dark skin tone",
];

fn month_english(month: u8) -> &'static str {
    MONTHS[usize::from(month.clamp(1, 12) - 1)]
}

impl UiText {
    /// A stable kebab-case identifier, such as `close` or `emoji-search`,
    /// for looking the string up in a message catalog.
    pub fn key(&self) -> &'static str {
        match self {
            Self::Close => "close",
            Self::CloseNotification => "close-notification",
            Self::Loading => "loading",
            Self::ReadOnly => "read-only",
            Self::Required(_) => "required",
            Self::OneTimeCode => "one-time-code",
            Self::InvalidValue => "invalid-value",
            Self::Unavailable => "unavailable",
            Self::NoResults => "no-results",
            Self::NoItems => "no-items",
            Self::ListInstructions => "list-instructions",
            Self::MultiSelectInstructions => "multi-select-instructions",
            Self::Selection => "selection",
            Self::SelectPlaceholder => "select-placeholder",
            Self::InvalidSelection(_) => "invalid-selection",
            Self::Navigation => "navigation",
            Self::ToggleSidebar => "toggle-sidebar",
            Self::CurrentDestination => "current-destination",
            Self::Expanded => "expanded",
            Self::Collapsed => "collapsed",
            Self::Cut => "cut",
            Self::Copy => "copy",
            Self::Paste => "paste",
            Self::SelectAll => "select-all",
            Self::DecreaseValue => "decrease-value",
            Self::IncreaseValue => "increase-value",
            Self::Minimize => "minimize",
            Self::Maximize => "maximize",
            Self::Restore => "restore",
            Self::Uploading(_) => "uploading",
            Self::UploadFailed => "upload-failed",
            Self::Remove(_) => "remove",
            Self::RetryUpload(_) => "retry-upload",
            Self::PreviousMonth => "previous-month",
            Self::NextMonth => "next-month",
            Self::Weekday(_) => "weekday",
            Self::WeekdayShort(_) => "weekday-short",
            Self::Month(_) => "month",
            Self::MonthYear { .. } => "month-year",
            Self::LongDate { .. } => "long-date",
            Self::ShortDate { .. } => "short-date",
            Self::Today => "today",
            Self::PickDate => "pick-date",
            Self::PickDateTime => "pick-date-time",
            Self::TimeField(_) => "time-field",
            Self::Hour => "hour",
            Self::Am => "am",
            Self::Pm => "pm",
            Self::DayPeriod => "day-period",
            Self::Minute => "minute",
            Self::Done => "done",
            Self::Carousel => "carousel",
            Self::PreviousSlide => "previous-slide",
            Self::NextSlide => "next-slide",
            Self::SearchCommands => "search-commands",
            Self::Commands => "commands",
            Self::CommandPalette => "command-palette",
            Self::Message => "message",
            Self::People => "people",
            Self::Channels => "channels",
            Self::Emoji => "emoji",
            Self::Suggestions => "suggestions",
            Self::LoadImage => "load-image",
            Self::LoadImageFrom(_) => "load-image-from",
            Self::ExpandPreview => "expand-preview",
            Self::CollapsePreview => "collapse-preview",
            Self::RemovePreview => "remove-preview",
            Self::Attachment => "attachment",
            Self::Jump => "jump",
            Self::JumpToMessageFrom(_) => "jump-to-message-from",
            Self::OriginallyPostedIn(_) => "originally-posted-in",
            Self::ShowMore => "show-more",
            Self::ShowLess => "show-less",
            Self::CopyCode => "copy-code",
            Self::Copied => "copied",
            Self::QuotedMessageFrom(_) => "quoted-message-from",
            Self::SearchEmoji => "emoji-search",
            Self::SearchResults => "emoji-search-results",
            Self::FrequentlyUsed => "emoji-frequently-used",
            Self::RecentlyUsed => "emoji-recently-used",
            Self::CustomEmoji => "emoji-custom",
            Self::EmojiCategories => "emoji-categories",
            Self::NoEmoji => "emoji-none",
            Self::SkinTone => "emoji-skin-tone",
            Self::SkinToneName(_) => "emoji-skin-tone-name",
            Self::PreviousImage => "previous-image",
            Self::NextImage => "next-image",
            Self::Image => "image",
            Self::ZoomOut => "zoom-out",
            Self::ZoomIn => "zoom-in",
            Self::ImageViewer(_) => "image-viewer",
            Self::Position { .. } => "position",
            Self::Zoom(_) => "zoom",
            Self::MessageFailed => "message-failed",
            Self::Retry => "retry",
            Self::Delete => "delete",
            Self::AddReaction => "add-reaction",
            Self::JumpToLatest => "jump-to-latest",
            Self::StatusOnline => "status-online",
            Self::StatusAway => "status-away",
            Self::StatusDnd => "status-dnd",
            Self::StatusOffline => "status-offline",
            Self::Names(_) => "names",
            Self::ClearSelection => "clear-selection",
            Self::ShowOptions => "show-options",
            Self::ColorHue => "color-hue",
            Self::ColorChroma => "color-chroma",
            Self::ColorLightness => "color-lightness",
            Self::SortInstructions { .. } => "sort-instructions",
            Self::SortGroup { .. } => "sort-group",
            Self::Moved { .. } => "moved",
        }
    }

    /// The English text, which components show when no translation is set.
    pub fn english(&self) -> SharedString {
        let text: &'static str = match self {
            Self::Uploading(percent) => return format!("Uploading… {percent}%").into(),
            Self::Required(label) => return format!("{label} (required)").into(),
            Self::InvalidSelection(label) => return format!("{label} (invalid value)").into(),
            Self::Remove(name) => return format!("Remove {name}").into(),
            Self::RetryUpload(name) => return format!("Retry uploading {name}").into(),
            Self::Weekday(day) => WEEKDAYS[usize::from(*day) % 7],
            Self::WeekdayShort(day) => &WEEKDAYS[usize::from(*day) % 7][..2],
            Self::Month(month) => month_english(*month),
            Self::TimeField(time) => return format!("Time, {time}").into(),
            Self::MonthYear { month, year } => {
                return format!("{} {year}", month_english(*month)).into();
            }
            Self::LongDate {
                weekday,
                month,
                day,
                year,
            } => {
                let weekday = WEEKDAYS[usize::from(*weekday) % 7];
                return format!("{weekday}, {} {day}, {year}", month_english(*month)).into();
            }
            Self::ShortDate { month, day, year } => {
                return format!("{} {day}, {year}", &month_english(*month)[..3]).into();
            }
            Self::Today => "today",
            Self::LoadImageFrom(host) => return format!("Load image from {host}").into(),
            Self::JumpToMessageFrom(author) => {
                return format!("Jump to the message from {author}").into();
            }
            Self::OriginallyPostedIn(channel) => {
                return format!("Originally posted in {channel}").into();
            }
            Self::QuotedMessageFrom(author) => {
                return format!("Quoted message from {author}").into();
            }
            Self::ImageViewer(name) => return format!("Image viewer: {name}").into(),
            Self::Names(names) => {
                return match names.as_slice() {
                    [] => SharedString::default(),
                    [only] => only.clone(),
                    [rest @ .., last] => format!(
                        "{} and {last}",
                        rest.iter()
                            .map(SharedString::as_ref)
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                    .into(),
                };
            }
            Self::Position { index, count } => return format!("{index} of {count}").into(),
            Self::SortInstructions { previous, next } => {
                return format!(
                    "Press Alt+Shift+{previous} or Alt+Shift+{next} to move the focused row."
                )
                .into();
            }
            Self::SortGroup {
                label,
                previous,
                next,
            } => return format!("{label}, move with Alt+Shift+{previous} or {next}").into(),
            Self::Moved {
                label,
                position,
                count,
                group,
            } => {
                return match group {
                    Some(group) => {
                        format!("{label} moved to position {position} of {count} in {group}")
                    }
                    None => format!("{label} moved to position {position} of {count}"),
                }
                .into();
            }
            Self::ClearSelection => "Clear selection",
            Self::ShowOptions => "Show options",
            Self::ColorHue => "Hue",
            Self::ColorChroma => "Chroma",
            Self::ColorLightness => "Lightness",
            Self::Zoom(percent) => return format!("Zoom {percent}%").into(),
            Self::SkinToneName(tone) => SKIN_TONES[usize::from(*tone).min(SKIN_TONES.len() - 1)],
            Self::Close => "Close",
            Self::CloseNotification => "Close notification",
            Self::Loading => "Loading",
            Self::ReadOnly => "Read only",
            Self::OneTimeCode => "One-time code",
            Self::InvalidValue => "Invalid value",
            Self::Unavailable => "Unavailable",
            Self::NoResults => "No results found.",
            Self::NoItems => "No items",
            Self::ListInstructions => "Use arrow keys to navigate and Enter to activate.",
            Self::MultiSelectInstructions => {
                "Multi-select list. Use arrows to navigate, Shift for a range, Command or Control to toggle."
            }
            Self::Selection => "Selection",
            Self::SelectPlaceholder => "Select…",
            Self::Navigation => "Navigation",
            Self::ToggleSidebar => "Toggle sidebar",
            Self::CurrentDestination => "current destination",
            Self::Expanded => "expanded",
            Self::Collapsed => "collapsed",
            Self::Cut => "Cut",
            Self::Copy => "Copy",
            Self::Paste => "Paste",
            Self::SelectAll => "Select All",
            Self::DecreaseValue => "Decrease value",
            Self::IncreaseValue => "Increase value",
            Self::Minimize => "Minimize",
            Self::Maximize => "Maximize",
            Self::Restore => "Restore",
            Self::UploadFailed => "Upload failed",
            Self::PreviousMonth => "Previous month",
            Self::NextMonth => "Next month",
            Self::PickDate => "Pick a date",
            Self::PickDateTime => "Pick a date and time",
            Self::Hour => "Hour",
            Self::Am => "AM",
            Self::Pm => "PM",
            Self::DayPeriod => "AM or PM",
            Self::Minute => "Minute",
            Self::Done => "Done",
            Self::Carousel => "Carousel",
            Self::PreviousSlide => "Previous slide",
            Self::NextSlide => "Next slide",
            Self::SearchCommands => "Search commands",
            Self::Commands => "Commands",
            Self::CommandPalette => "Command palette",
            Self::Message => "Message",
            Self::People => "People",
            Self::Channels => "Channels",
            Self::Emoji => "Emoji",
            Self::Suggestions => "Suggestions",
            Self::LoadImage => "Load image",
            Self::ExpandPreview => "Expand preview",
            Self::CollapsePreview => "Collapse preview",
            Self::RemovePreview => "Remove preview",
            Self::Attachment => "Attachment",
            Self::Jump => "Jump",
            Self::ShowMore => "Show more",
            Self::ShowLess => "Show less",
            Self::CopyCode => "Copy code",
            Self::Copied => "Copied",
            Self::SearchEmoji => "Search emoji",
            Self::SearchResults => "Search results",
            Self::FrequentlyUsed => "Frequently used",
            Self::RecentlyUsed => "Recently used",
            Self::CustomEmoji => "Custom",
            Self::EmojiCategories => "Emoji categories",
            Self::NoEmoji => "No emoji found.",
            Self::SkinTone => "Skin tone",
            Self::PreviousImage => "Previous image",
            Self::NextImage => "Next image",
            Self::Image => "Image",
            Self::ZoomOut => "Zoom out",
            Self::ZoomIn => "Zoom in",
            Self::MessageFailed => "Message failed to send.",
            Self::Retry => "Retry",
            Self::Delete => "Delete",
            Self::AddReaction => "Add reaction",
            Self::JumpToLatest => "Jump to latest",
            Self::StatusOnline => "online",
            Self::StatusAway => "away",
            Self::StatusDnd => "do not disturb",
            Self::StatusOffline => "offline",
        };
        SharedString::new_static(text)
    }
}

type Translate = Rc<dyn Fn(&UiText) -> Option<SharedString>>;

#[derive(Clone)]
struct Strings(Translate);
impl Global for Strings {}

/// Installs the application's translations of the built-in strings and
/// redraws every window. Return `None` for a string to keep its English.
///
/// ```ignore
/// locale::set_strings(cx, move |text| match text {
///     UiText::Close => Some("إغلاق".into()),
///     UiText::Remove(name) => Some(format!("إزالة {name}").into()),
///     _ => None,
/// });
/// ```
pub fn set_strings(cx: &mut App, translate: impl Fn(&UiText) -> Option<SharedString> + 'static) {
    cx.set_global(Strings(Rc::new(translate)));
    cx.refresh_windows();
}

/// Removes the application's translations, so every built-in string is English.
pub fn clear_strings(cx: &mut App) {
    if cx.has_global::<Strings>() {
        cx.remove_global::<Strings>();
        cx.refresh_windows();
    }
}

/// The application's translation of a built-in string, or its English.
pub fn text(cx: &App, text: UiText) -> SharedString {
    cx.try_global::<Strings>()
        .and_then(|strings| (strings.0)(&text))
        .unwrap_or_else(|| text.english())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn english_covers_arguments_and_tables() {
        assert_eq!(UiText::Remove("a.png".into()).english(), "Remove a.png");
        assert_eq!(UiText::Month(1).english(), "January");
        assert_eq!(UiText::Month(12).english(), "December");
        assert_eq!(UiText::WeekdayShort(0).english(), "Su");
        assert_eq!(UiText::SkinToneName(5).english(), "Dark skin tone");
        assert_eq!(UiText::Uploading(40).english(), "Uploading… 40%");
        let date = UiText::LongDate {
            weekday: 6,
            month: 3,
            day: 14,
            year: 2026,
        };
        assert_eq!(date.english(), "Saturday, March 14, 2026");
        let date = UiText::ShortDate {
            month: 3,
            day: 14,
            year: 2026,
        };
        assert_eq!(date.english(), "Mar 14, 2026");
    }

    #[test]
    fn logical_keys_swap_only_in_rtl() {
        assert_eq!(Direction::Ltr.logical_key("left"), "left");
        assert_eq!(Direction::Rtl.logical_key("left"), "right");
        assert_eq!(Direction::Rtl.logical_key("right"), "left");
        assert_eq!(Direction::Rtl.logical_key("up"), "up");
    }

    #[test]
    fn isolate_wraps_once() {
        let name = isolate("سارة");
        assert_eq!(name.as_ref(), "\u{2068}سارة\u{2069}");
        assert_eq!(isolate(&name), name);
        assert_eq!(isolate(""), "");
    }

    #[gpui_kit::test]
    fn translations_replace_english_and_fall_back(cx: &mut gpui_kit::TestAppContext) {
        cx.update(|cx| {
            assert_eq!(text(cx, UiText::Close), "Close");
            set_strings(cx, |text| match text {
                UiText::Close => Some("إغلاق".into()),
                UiText::Remove(name) => Some(format!("إزالة {name}").into()),
                _ => None,
            });
            assert_eq!(text(cx, UiText::Close), "إغلاق");
            assert_eq!(text(cx, UiText::Remove("a".into())), "إزالة a");
            assert_eq!(text(cx, UiText::Done), "Done");
            clear_strings(cx);
            assert_eq!(text(cx, UiText::Close), "Close");
        });
    }

    #[gpui_kit::test]
    fn direction_survives_theme_swaps(cx: &mut gpui_kit::TestAppContext) {
        cx.update(|cx| {
            super::super::init(cx);
            set_direction(cx, Direction::Rtl);
            assert_eq!(UiTheme::read(cx).direction, Direction::Rtl);
            UiTheme::switch(cx, super::super::ThemeMode::Dark);
            assert_eq!(UiTheme::read(cx).direction, Direction::Rtl);
            UiTheme::set(cx, UiTheme::neutral_light());
            assert_eq!(direction(cx), Direction::Rtl);
            assert_eq!(UiTheme::read(cx).direction, Direction::Rtl);
        });
    }
}
