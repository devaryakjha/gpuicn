//! Select state synchronization checks for edit-existing-record flows.

use gpui_kit::{
    AppContext as _, Context, Entity, IntoElement, Render, SharedString, Styled, TestAppContext,
    Window, px,
};
use gpuicn::select::{SelectItem, SelectState};

#[derive(Clone, Copy)]
enum Presentation {
    Combobox,
    Autocomplete,
}

struct PickerView {
    state: Entity<SelectState>,
    presentation: Presentation,
}

impl Render for PickerView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let picker = match self.presentation {
            Presentation::Combobox => gpuicn::combobox::combobox(&self.state),
            Presentation::Autocomplete => gpuicn::autocomplete::autocomplete(&self.state),
        };
        picker.aria_label("Fruit").w(px(240.))
    }
}

fn picker<'a>(
    cx: &'a mut TestAppContext,
    presentation: Presentation,
    value: &'static str,
) -> (Entity<PickerView>, &'a mut gpui_kit::VisualTestContext) {
    cx.update(gpuicn::init);
    let (view, cx) = cx.add_window_view(move |window, cx| {
        let state = cx.new(|cx| {
            SelectState::new(
                [
                    SelectItem::new("apple", "Apple"),
                    SelectItem::new("pear", "Pear"),
                ],
                window,
                cx,
            )
        });
        state.update(cx, |state, cx| {
            state.set_value(Some(SharedString::from(value.to_owned())), window, cx)
        });
        PickerView {
            state,
            presentation,
        }
    });
    cx.update(|window, cx| {
        window.activate_window();
        window.draw(cx).clear(cx);
    });
    cx.run_until_parked();
    cx.update(|window, cx| window.draw(cx).clear(cx));
    (view, cx)
}

#[gpui_kit::test]
fn combobox_syncs_prefill_programmatic_changes_and_reset(cx: &mut TestAppContext) {
    let (view, cx) = picker(cx, Presentation::Combobox, "pear");
    cx.update(|_, cx| {
        let state = &view.read(cx).state;
        assert_eq!(
            state.read(cx).value().map(|value| value.as_ref()),
            Some("pear")
        );
        assert_eq!(state.read(cx).input().read(cx).value().as_ref(), "Pear");
        assert!(!state.read(cx).is_open());
    });

    cx.update(|window, cx| {
        let state = view.read(cx).state.clone();
        state.update(cx, |state, cx| {
            state.set_value(Some("apple".into()), window, cx)
        });
    });
    cx.update(|_, cx| {
        let state = &view.read(cx).state;
        assert_eq!(
            state.read(cx).value().map(|value| value.as_ref()),
            Some("apple")
        );
        assert_eq!(state.read(cx).input().read(cx).value().as_ref(), "Apple");
        assert!(!state.read(cx).is_open());
    });

    cx.update(|window, cx| {
        let state = view.read(cx).state.clone();
        state.update(cx, |state, cx| state.set_value(None, window, cx));
    });
    cx.update(|_, cx| {
        let state = &view.read(cx).state;
        assert_eq!(state.read(cx).value(), None);
        assert!(state.read(cx).input().read(cx).value().is_empty());
        assert!(!state.read(cx).is_open());
    });

    cx.update(|window, cx| {
        let state = view.read(cx).state.clone();
        state.update(cx, |state, cx| {
            state.set_value(Some("pear".into()), window, cx)
        });
    });
    cx.update(|_, cx| {
        let state = &view.read(cx).state;
        assert_eq!(
            state.read(cx).value().map(|value| value.as_ref()),
            Some("pear")
        );
        assert_eq!(state.read(cx).input().read(cx).value().as_ref(), "Pear");
        assert!(!state.read(cx).is_open());
    });
}

#[gpui_kit::test]
fn autocomplete_preserves_free_text_across_prefill_changes_and_reset(cx: &mut TestAppContext) {
    let (view, cx) = picker(cx, Presentation::Autocomplete, "dragonfruit");
    cx.update(|_, cx| {
        let state = &view.read(cx).state;
        assert_eq!(
            state.read(cx).value().map(|value| value.as_ref()),
            Some("dragonfruit")
        );
        assert_eq!(
            state.read(cx).input().read(cx).value().as_ref(),
            "dragonfruit"
        );
        assert!(!state.read(cx).is_open());
    });

    for value in [Some("lychee"), None, Some("pear")] {
        cx.update(|window, cx| {
            let state = view.read(cx).state.clone();
            state.update(cx, |state, cx| {
                state.set_value(value.map(SharedString::from), window, cx)
            });
        });
    }
    cx.run_until_parked();
    cx.update(|_, cx| {
        let state = &view.read(cx).state;
        assert_eq!(
            state.read(cx).value().map(|value| value.as_ref()),
            Some("pear")
        );
        assert_eq!(state.read(cx).input().read(cx).value().as_ref(), "Pear");
        assert!(!state.read(cx).is_open());
    });
}
