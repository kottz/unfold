use gpui::*;

struct SimpleTextBox {
    text: SharedString,
    is_dragging: bool,
    start_position: Option<Point<Pixels>>,
}

impl SimpleTextBox {
    fn new(_cx: &mut ViewContext<Self>) -> Self {
        Self {
            text: "Hello World".into(),
            is_dragging: false,
            start_position: None,
        }
    }
}

impl Render for SimpleTextBox {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
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
                            .cursor(CursorStyle::OpenHand)
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, ev: &MouseDownEvent, cx| {
                                    this.is_dragging = true;
                                    this.start_position = Some(ev.position);
                                    cx.start_window_move();
                                    cx.notify();
                                }),
                            )
                            .on_mouse_move(cx.listener(|this, ev: &MouseMoveEvent, cx| {
                                if this.is_dragging {
                                    if let Some(start) = this.start_position {
                                        if ev.position != start {
                                            cx.start_window_move();
                                        }
                                    }
                                }
                            }))
                            .on_mouse_up(
                                MouseButton::Left,
                                cx.listener(|this, _ev: &MouseUpEvent, cx| {
                                    this.is_dragging = false;
                                    this.start_position = None;
                                    cx.notify();
                                }),
                            )
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
                titlebar: None,
                window_decorations: Some(WindowDecorations::Client),
                is_movable: true,
                ..Default::default()
            },
            |cx| cx.new_view(SimpleTextBox::new),
        )
        .unwrap();
        cx.activate(true);
    });
}
