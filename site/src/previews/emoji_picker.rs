//! The emoji picker catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn emoji_picker_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let owner = cx.entity().downgrade();
        // Shown like a reaction popover: picks keep it open, and Escape
        // closes it. Recents picked while open show from the next open.
        let shown = window
            .use_keyed_state("emoji-picker.shown", cx, |_, _| true)
            .clone();
        let hide = shown.clone();
        let avatar = "https://raw.githubusercontent.com/devaryakjha/devaryakjha/6526e3d7415b2fb573ba3da4523b5c6948aa5d08/avatar.png";
        let state = window
            .use_keyed_state("emoji-picker.state", cx, |window, cx| {
                let state = cx.new(|cx| {
                    let mut state = EmojiPickerState::new(emoji_catalog(), window, cx);
                    state.set_custom([("arya".into(), avatar.into())], cx);
                    state.set_frequent(["thumbsup", "tada", "rocket", "eyes", "100"], cx);
                    state.set_recents(["tada", "thumbsup", "eyes", "arya"], cx);
                    state
                });
                let recents = std::rc::Rc::new(std::cell::RefCell::new(vec![
                    SharedString::from("tada"),
                    "thumbsup".into(),
                    "eyes".into(),
                    "arya".into(),
                ]));
                let subscription =
                    cx.subscribe_in(&state, window, move |_, state, event, _, cx| {
                        let action = match event {
                            EmojiPickerEvent::Pick(name) => {
                                // The application owns recents: move the pick to the front.
                                let mut list = recents.borrow_mut();
                                list.retain(|recent| recent != name);
                                list.insert(0, name.clone());
                                list.truncate(18);
                                let list = list.clone();
                                state.update(cx, |state, cx| state.set_recents(list, cx));
                                format!("Picked :{name}:")
                            }
                            EmojiPickerEvent::Cancel => {
                                hide.update(cx, |shown, cx| {
                                    *shown = false;
                                    cx.notify();
                                });
                                "Closed".into()
                            }
                            EmojiPickerEvent::SkinTone(tone) => match tone {
                                Some(tone) => format!("Skin tone {}", tone + 1),
                                None => "Default skin tone".into(),
                            },
                        };
                        let _ = owner.update(cx, |this, cx| {
                            this.demo_action = action;
                            cx.notify();
                        });
                    });
                (state, subscription)
            })
            .read(cx)
            .0
            .clone();
        let open = *shown.read(cx);
        let toggle = state.clone();
        div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(12.))
            .child(
                Button::new("emoji-picker.toggle")
                    .variant(ButtonVariant::Outline)
                    .on_click(move |_, window, cx| {
                        let open = !*shown.read(cx);
                        shown.update(cx, |shown, cx| {
                            *shown = open;
                            cx.notify();
                        });
                        if open {
                            toggle.update(cx, |state, cx| state.open(window, cx));
                        }
                    })
                    .child(if open { "Close picker" } else { "Add reaction" }),
            )
            .when(open, |preview| preview.child(EmojiPicker::new(&state)))
    }
}

/// A small sample: applications supply the full Unicode set and its shortcodes.
fn emoji_catalog() -> Vec<EmojiCategory> {
    // Hands come in five skin tones, named like Mattermost's `_tone1` variants.
    let toned = [
        "wave",
        "raised_hand",
        "ok_hand",
        "v",
        "crossed_fingers",
        "point_up",
        "thumbsup",
        "thumbsdown",
        "clap",
        "raised_hands",
        "pray",
        "muscle",
        "writing_hand",
    ];
    let category = |name: &'static str, icon, emoji: &[(&'static str, &'static str)]| {
        EmojiCategory::new(
            name,
            icon,
            emoji.iter().map(|(name, glyph)| {
                let emoji = Emoji::new(*name, *glyph);
                if !toned.contains(name) {
                    return emoji;
                }
                let base = glyph.trim_end_matches('\u{fe0f}');
                emoji.skin_tones(["🏻", "🏼", "🏽", "🏾", "🏿"].iter().enumerate().map(
                    |(tone, modifier)| {
                        Emoji::new(
                            format!("{name}_tone{}", tone + 1),
                            format!("{base}{modifier}"),
                        )
                    },
                ))
            }),
        )
    };
    vec![
        category(
            "Smileys & Emotion",
            LucideIcon::FaceSlightlySmiling,
            &[
                ("grinning", "😀"),
                ("smiley", "😃"),
                ("smile", "😄"),
                ("grin", "😁"),
                ("laughing", "😆"),
                ("sweat_smile", "😅"),
                ("joy", "😂"),
                ("rofl", "🤣"),
                ("slightly_smiling_face", "🙂"),
                ("upside_down_face", "🙃"),
                ("wink", "😉"),
                ("blush", "😊"),
                ("innocent", "😇"),
                ("heart_eyes", "😍"),
                ("star_struck", "🤩"),
                ("kissing_heart", "😘"),
                ("yum", "😋"),
                ("stuck_out_tongue", "😛"),
                ("thinking", "🤔"),
                ("neutral_face", "😐"),
                ("expressionless", "😑"),
                ("smirk", "😏"),
                ("unamused", "😒"),
                ("roll_eyes", "🙄"),
                ("grimacing", "😬"),
                ("relieved", "😌"),
                ("pensive", "😔"),
                ("sleepy", "😪"),
                ("sleeping", "😴"),
                ("sunglasses", "😎"),
                ("nerd_face", "🤓"),
                ("confused", "😕"),
                ("worried", "😟"),
                ("open_mouth", "😮"),
                ("astonished", "😲"),
                ("flushed", "😳"),
                ("cry", "😢"),
                ("sob", "😭"),
                ("scream", "😱"),
                ("rage", "😡"),
                ("skull", "💀"),
                ("heart", "❤️"),
                ("broken_heart", "💔"),
                ("sparkling_heart", "💖"),
                ("100", "💯"),
            ],
        ),
        category(
            "People & Body",
            LucideIcon::Hand,
            &[
                ("wave", "👋"),
                ("raised_hand", "✋"),
                ("ok_hand", "👌"),
                ("v", "✌️"),
                ("crossed_fingers", "🤞"),
                ("point_up", "☝️"),
                ("thumbsup", "👍"),
                ("thumbsdown", "👎"),
                ("clap", "👏"),
                ("raised_hands", "🙌"),
                ("pray", "🙏"),
                ("muscle", "💪"),
                ("eyes", "👀"),
                ("brain", "🧠"),
                ("handshake", "🤝"),
                ("writing_hand", "✍️"),
                ("shrug", "🤷"),
                ("facepalm", "🤦"),
            ],
        ),
        category(
            "Animals & Nature",
            LucideIcon::Leaf,
            &[
                ("dog", "🐶"),
                ("cat", "🐱"),
                ("fox_face", "🦊"),
                ("panda_face", "🐼"),
                ("penguin", "🐧"),
                ("owl", "🦉"),
                ("turtle", "🐢"),
                ("octopus", "🐙"),
                ("crab", "🦀"),
                ("bee", "🐝"),
                ("seedling", "🌱"),
                ("evergreen_tree", "🌲"),
                ("cactus", "🌵"),
                ("sunflower", "🌻"),
                ("fire", "🔥"),
                ("star", "⭐"),
                ("zap", "⚡"),
                ("rainbow", "🌈"),
            ],
        ),
        category(
            "Food & Drink",
            LucideIcon::Coffee,
            &[
                ("apple", "🍎"),
                ("avocado", "🥑"),
                ("pizza", "🍕"),
                ("taco", "🌮"),
                ("ramen", "🍜"),
                ("sushi", "🍣"),
                ("doughnut", "🍩"),
                ("cookie", "🍪"),
                ("cake", "🍰"),
                ("coffee", "☕"),
                ("tea", "🍵"),
                ("beers", "🍻"),
            ],
        ),
        category(
            "Activities",
            LucideIcon::PartyPopper,
            &[
                ("tada", "🎉"),
                ("confetti_ball", "🎊"),
                ("balloon", "🎈"),
                ("gift", "🎁"),
                ("trophy", "🏆"),
                ("medal", "🏅"),
                ("soccer", "⚽"),
                ("basketball", "🏀"),
                ("video_game", "🎮"),
                ("dart", "🎯"),
                ("chess_pawn", "♟️"),
                ("art", "🎨"),
            ],
        ),
        category(
            "Objects",
            LucideIcon::Lightbulb,
            &[
                ("rocket", "🚀"),
                ("bulb", "💡"),
                ("computer", "💻"),
                ("keyboard", "⌨️"),
                ("hammer_and_wrench", "🛠️"),
                ("gear", "⚙️"),
                ("lock", "🔒"),
                ("key", "🔑"),
                ("memo", "📝"),
                ("pushpin", "📌"),
                ("bell", "🔔"),
                ("calendar", "📅"),
                ("white_check_mark", "✅"),
                ("x", "❌"),
                ("warning", "⚠️"),
                ("question", "❓"),
            ],
        ),
    ]
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.emoji_picker_preview(window, cx).into_any_element()
}
