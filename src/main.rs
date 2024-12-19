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
                    .flex()
                    .flex_col()
                    .child(
                        // Header
                        div()
                            .w(px(300.))
                            .h(px(24.))
                            .bg(rgb(0x2D3142))
                            .rounded_t_md()
                            .flex()
                            .items_center()
                            .px_2()
                            .text_color(rgb(0xFFFFFF))
                            .text_sm()
                            .child("Notes"),
                    )
                    .child(
                        // Text box
                        div()
                            .w(px(300.))
                            .h(px(40.))
                            .bg(white())
                            .rounded_b_md()
                            .shadow_md()
                            .p_2()
                            .flex()
                            .items_center()
                            .child(self.text.clone()),
                    ),
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
