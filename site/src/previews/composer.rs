//! The composer catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn composer_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let owner = cx.entity().downgrade();
        let state = window
            .use_keyed_state("composer.state", cx, |window, cx| {
                let state = cx.new(|cx| {
                    let mut state = ComposerState::new(6, window, cx);
                    state.set_placeholder("Message ~town-square", window, cx);
                    state.set_suggestion_leading(|suggestion, _, _| {
                        if let Some(name) = suggestion.value.strip_prefix('@') {
                            return Some(
                                Avatar::new(gpui_kit::ElementId::Name(
                                    format!("composer.avatar.{name}").into(),
                                ))
                                .size(AvatarSize::Sm)
                                .w(px(20.))
                                .h(px(20.))
                                .fallback(name.chars().take(1).collect::<String>().to_uppercase())
                                .into_any_element(),
                            );
                        }
                        let glyph = composer_emoji()
                            .into_iter()
                            .find(|(name, _)| suggestion.value.trim_matches(':') == *name)?
                            .1;
                        Some(div().w(px(20.)).child(glyph).into_any_element())
                    });
                    state
                });
                let subscription = cx.subscribe_in(
                    &state,
                    window,
                    move |_, state, event, window, cx| match event {
                        ComposerEvent::Autocomplete { trigger, query } => {
                            let suggestions = composer_suggestions(*trigger, query);
                            state.update(cx, |state, cx| state.set_suggestions(suggestions, cx));
                        }
                        ComposerEvent::Submit(text) if !text.trim().is_empty() => {
                            let action = format!("Sent “{}”", text.trim());
                            state.update(cx, |state, cx| state.set_value("", window, cx));
                            let _ = owner.update(cx, |this, cx| {
                                this.demo_action = action;
                                cx.notify();
                            });
                        }
                        _ => {}
                    },
                );
                // Open on a half-typed mention so the popup shows. The editor
                // shapes text with the theme font only after its first layout.
                let prefill = state.clone();
                window.on_next_frame(move |window, cx| {
                    prefill.update(cx, |state, cx| {
                        state.set_value("Thanks for the review @ar", window, cx)
                    });
                });
                (state, subscription)
            })
            .read(cx)
            .0
            .clone();
        let colors = UiTheme::read(cx).colors;
        let tool = |id: &'static str, label: &'static str, icon: LucideIcon| {
            Button::new(id)
                .size(ButtonSize::IconSm)
                .variant(ButtonVariant::Ghost)
                .aria_label(label)
                .child(
                    lucide(icon)
                        .size(px(16.))
                        .text_color(colors.muted_foreground),
                )
        };
        let bold = state.clone();
        let emoji = state.clone();
        let mention = state.clone();
        let send = state.clone();
        div().w_full().max_w(px(480.)).pt(px(220.)).child(
            Composer::new(&state)
                .child(tool(
                    "composer.attach",
                    "Attach files",
                    LucideIcon::Paperclip,
                ))
                .child(tool("composer.bold", "Bold", LucideIcon::Bold).on_click(
                    move |_, window, cx| {
                        bold.update(cx, |state, cx| {
                            state.wrap_selection("**", "**", window, cx);
                            state.focus(window, cx);
                        })
                    },
                ))
                .child(
                    tool("composer.mention", "Mention someone", LucideIcon::AtSign).on_click(
                        move |_, window, cx| {
                            mention.update(cx, |state, cx| {
                                state.insert("@", window, cx);
                                state.focus(window, cx);
                            })
                        },
                    ),
                )
                .child(
                    tool(
                        "composer.emoji",
                        "Add emoji",
                        LucideIcon::FaceSlightlySmilingPlus,
                    )
                    .on_click(move |_, window, cx| {
                        emoji.update(cx, |state, cx| {
                            state.insert(":", window, cx);
                            state.focus(window, cx);
                        })
                    }),
                )
                .child(div().flex_1())
                .child(
                    Button::new("composer.send")
                        .size(ButtonSize::IconSm)
                        .aria_label("Send")
                        .on_click(move |_, window, cx| {
                            send.update(cx, |state, cx| {
                                let text = state.value(cx);
                                if !text.trim().is_empty() {
                                    cx.emit(ComposerEvent::Submit(text));
                                }
                                state.focus(window, cx);
                            })
                        })
                        .child(
                            lucide(LucideIcon::SendHorizontal)
                                .size(px(16.))
                                .text_color(colors.primary_foreground),
                        ),
                ),
        )
    }
}

fn composer_emoji() -> [(&'static str, &'static str); 6] {
    [
        ("tada", "🎉"),
        ("thumbsup", "👍"),
        ("eyes", "👀"),
        ("rocket", "🚀"),
        ("heart", "❤️"),
        ("thinking", "🤔"),
    ]
}

fn composer_suggestions(trigger: char, query: &str) -> Vec<ComposerSuggestion> {
    let query = query.to_lowercase();
    let matches = |text: &str| text.to_lowercase().contains(&query);
    match trigger {
        '@' => [
            ("arya", "Arya Jha"),
            ("aryan", "Aryan Mehta"),
            ("sam", "Sam Rivera"),
            ("lee", "Lee Park"),
        ]
        .into_iter()
        .filter(|(handle, name)| matches(handle) || matches(name))
        .map(|(handle, name)| {
            ComposerSuggestion::new(format!("@{handle}"), name).description(format!("@{handle}"))
        })
        .collect(),
        '~' => [
            ("town-square", "Company-wide announcements"),
            ("gpuicn", "Components for GPUI"),
            ("random", "Off-topic"),
        ]
        .into_iter()
        .filter(|(name, _)| matches(name))
        .map(|(name, purpose)| {
            ComposerSuggestion::new(format!("~{name}"), name)
                .description(purpose)
                .icon(LucideIcon::Hash)
        })
        .collect(),
        // Like Mattermost, emoji search waits for two characters.
        ':' if query.chars().count() >= 2 => composer_emoji()
            .into_iter()
            .filter(|(name, _)| matches(name))
            .map(|(name, _)| ComposerSuggestion::new(format!(":{name}:"), format!(":{name}:")))
            .collect(),
        '/' => [
            ("away", "Set your status to away"),
            ("invite", "Invite someone to the channel"),
            ("status", "Set a custom status"),
        ]
        .into_iter()
        .filter(|(name, _)| matches(name))
        .map(|(name, help)| {
            ComposerSuggestion::new(format!("/{name}"), format!("/{name}"))
                .description(help)
                .icon(LucideIcon::SquareSlash)
        })
        .collect(),
        _ => Vec::new(),
    }
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.composer_preview(window, cx).into_any_element()
}
