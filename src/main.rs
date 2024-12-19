use gpui::*;

struct SimpleTextBox {
    text: SharedString,
}

impl SimpleTextBox {
    fn new(_cx: &mut ViewContext<Self>) -> Self {
        Self {
            text: "Hello World".into(),
        }
    }
}

impl Render for SimpleTextBox {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .size_full()
            .bg(rgb(0xEEEEEE))
            .flex()
            .justify_center()
            .items_center()
            .child(
                div()
                    .w(px(300.))
                    .h(px(40.))
                    .bg(white())
                    .rounded_md()
                    .shadow_md()
                    .p_2()
                    .flex()
                    .items_center()
                    .child(self.text.clone()),
            )
    }
}

fn main() {
    App::new().run(|cx: &mut AppContext| {
        let bounds = Bounds::centered(None, size(px(800.), px(600.)), cx);

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |cx| cx.new_view(SimpleTextBox::new),
        )
        .unwrap();
    });
}
