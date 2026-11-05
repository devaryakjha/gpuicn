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

    /// The chevron or arrow pointing toward the reading end (`forward`) or
    /// start, from the left-to-right `right` icon and `left` icon.
    pub fn icon<T>(self, forward: bool, left: T, right: T) -> T {
        if forward == self.is_rtl() { left } else { right }
    }
}

/// The key context a right-to-left text input adds around Kit's `Input`, so
/// the arrow keys move toward the visual side they point to.
#[doc(hidden)]
pub const RTL_INPUT_CONTEXT: &str = "GpuicnRtl";

/// Binds the mirrored arrow keys of right-to-left text inputs.
pub(super) fn bind_keys(cx: &mut App) {
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
        ["ctrl-left", "ctrl-right", "ctrl-shift-left", "ctrl-shift-right"]
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
    /// "Selection", a select's default accessible name.
    Selection,
    /// "Navigation", the sidebar's accessible name.
    Navigation,
    /// "Toggle sidebar".
    ToggleSidebar,
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
    /// "Pick a date", the date picker's placeholder.
    PickDate,
    /// "Pick a date and time", the date and time picker's placeholder.
    PickDateTime,
    /// "Time, {time}", the time field's accessible name.
    TimeField(SharedString),
    /// "Hour".
    Hour,
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

impl UiText {
    /// A stable kebab-case identifier, such as `close` or `emoji-search`,
    /// for looking the string up in a message catalog.
    pub fn key(&self) -> &'static str {
        match self {
            Self::Close => "close",
            Self::CloseNotification => "close-notification",
            Self::Loading => "loading",
            Self::ReadOnly => "read-only",
            Self::InvalidValue => "invalid-value",
            Self::Unavailable => "unavailable",
            Self::NoResults => "no-results",
            Self::NoItems => "no-items",
            Self::ListInstructions => "list-instructions",
            Self::Selection => "selection",
            Self::Navigation => "navigation",
            Self::ToggleSidebar => "toggle-sidebar",
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
            Self::PickDate => "pick-date",
            Self::PickDateTime => "pick-date-time",
            Self::TimeField(_) => "time-field",
            Self::Hour => "hour",
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
            Self::MessageFailed => "message-failed",
            Self::Retry => "retry",
            Self::Delete => "delete",
            Self::AddReaction => "add-reaction",
            Self::JumpToLatest => "jump-to-latest",
            Self::StatusOnline => "status-online",
            Self::StatusAway => "status-away",
            Self::StatusDnd => "status-dnd",
            Self::StatusOffline => "status-offline",
        }
    }

    /// The English text, which components show when no translation is set.
    pub fn english(&self) -> SharedString {
        let text: &'static str = match self {
            Self::Uploading(percent) => return format!("Uploading… {percent}%").into(),
            Self::Remove(name) => return format!("Remove {name}").into(),
            Self::RetryUpload(name) => return format!("Retry uploading {name}").into(),
            Self::Weekday(day) => WEEKDAYS[usize::from(*day) % 7],
            Self::WeekdayShort(day) => &WEEKDAYS[usize::from(*day) % 7][..2],
            Self::Month(month) => MONTHS[usize::from(month.clamp(&1, &12) - 1)],
            Self::TimeField(time) => return format!("Time, {time}").into(),
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
            Self::SkinToneName(tone) => SKIN_TONES[usize::from(*tone).min(SKIN_TONES.len() - 1)],
            Self::Close => "Close",
            Self::CloseNotification => "Close notification",
            Self::Loading => "Loading",
            Self::ReadOnly => "Read only",
            Self::InvalidValue => "Invalid value",
            Self::Unavailable => "Unavailable",
            Self::NoResults => "No results found.",
            Self::NoItems => "No items",
            Self::ListInstructions => "Use arrow keys to navigate and Enter to activate.",
            Self::Selection => "Selection",
            Self::Navigation => "Navigation",
            Self::ToggleSidebar => "Toggle sidebar",
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
