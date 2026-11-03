//! The form catalog preview.

use crate::*;

impl Showcase {
    pub(crate) fn form_preview(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let validation = window.use_keyed_state("subscribe.validation", cx, |_, _| {
            (false, false, false, false)
        });
        let fruit = window.use_keyed_state("subscribe.fruit", cx, |window, cx| {
            SelectState::new(
                [
                    SelectItem::new("apple", "Apple"),
                    SelectItem::new("pear", "Pear"),
                ],
                window,
                cx,
            )
        });
        let quantity = window.use_keyed_state("subscribe.quantity", cx, |window, cx| {
            InputState::new(window, cx)
                .default_value("0")
                .min(0.)
                .max(20.)
                .step(1.)
        });
        let enter_validation = validation.clone();
        let enter_fruit = fruit.clone();
        let enter_quantity = quantity.clone();
        let email = live_input(
            "subscribe.email",
            "",
            "you@example.com",
            window,
            cx,
            move |this, input, event, cx| {
                if matches!(event, InputEvent::PressEnter { .. }) {
                    this.submit_form(
                        input.read(cx).value(),
                        enter_fruit.read(cx).value().cloned(),
                        enter_quantity.read(cx).value(),
                        &enter_validation,
                        cx,
                    );
                }
            },
        );
        let result = *validation.read(cx);
        let submit_email = email.clone();
        let submit_fruit = fruit.clone();
        let submit_quantity = quantity.clone();
        form("subscribe", cx)
            .w(px(320.))
            .aria_label("Fruit order")
            .child(
                Field::new("subscribe-email", &email)
                    .label("Email")
                    .required(true)
                    .when(result.0, |f| f.error("Enter a valid email address.")),
            )
            .child(
                Field::from_control(
                    "subscribe-fruit",
                    Select::new(&fruit).placeholder("Select a fruit…"),
                )
                .label("Fruit")
                .required(true)
                .when(result.1, |f| f.error("Select a fruit.")),
            )
            .child(
                Field::from_control("subscribe-quantity", NumberField::new(&quantity))
                    .label("Quantity")
                    .required(true)
                    .when(result.2, |f| f.error("Enter a quantity from 1 to 20.")),
            )
            .child(
                Button::new("subscribe-submit")
                    .label("Place order")
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.submit_form(
                            submit_email.read(cx).value(),
                            submit_fruit.read(cx).value().cloned(),
                            submit_quantity.read(cx).value(),
                            &validation,
                            cx,
                        );
                    })),
            )
            .when(result.3, |f| f.child("Order submitted."))
    }

    fn submit_form(
        &mut self,
        email: SharedString,
        fruit: Option<SharedString>,
        quantity: SharedString,
        validation: &Entity<(bool, bool, bool, bool)>,
        cx: &mut Context<Self>,
    ) {
        let email_invalid = !email
            .split_once('@')
            .is_some_and(|(name, host)| !name.is_empty() && host.contains('.'));
        let fruit_invalid = fruit.is_none();
        let quantity_invalid = !quantity
            .parse::<u32>()
            .is_ok_and(|quantity| (1..=20).contains(&quantity));
        let submitted = !(email_invalid || fruit_invalid || quantity_invalid);
        validation.update(cx, |validation, cx| {
            *validation = (email_invalid, fruit_invalid, quantity_invalid, submitted);
            cx.notify();
        });
        cx.notify();
    }
}

pub(crate) fn preview(view: &Showcase, window: &mut Window, cx: &mut Context<Showcase>) -> gpui_kit::AnyElement {
    view.form_preview(window, cx).into_any_element()
}
