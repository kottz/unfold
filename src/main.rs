use gpui::*;

struct SimpleTextBox {
    text: SharedString,
    position: Point<Pixels>,
    is_dragging: bool,
    drag_offset: Option<Point<Pixels>>,
}

impl SimpleTextBox {
    fn new(_cx: &mut ViewContext<Self>) -> Self {
        Self {
            text: "Hello World".into(),
            position: point(px(250.), px(250.)), // Initial position
            is_dragging: false,
            drag_offset: None,
        }
    }
}

impl Render for SimpleTextBox {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .size_full()
            .bg(rgb(0xEEEEEE))
            .flex()
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, cx| {
                if this.is_dragging {
                    if let Some(offset) = this.drag_offset {
                        this.position =
                            point(event.position.x - offset.x, event.position.y - offset.y);
                        cx.notify();
                    }
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _: &MouseUpEvent, _cx| {
                    this.is_dragging = false;
                    this.drag_offset = None;
                }),
            )
            .child(
                div()
                    .absolute()
                    .left(self.position.x)
                    .top(self.position.y)
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
                            .cursor(if self.is_dragging {
                                CursorStyle::ClosedHand
                            } else {
                                CursorStyle::OpenHand
                            })
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, event: &MouseDownEvent, _cx| {
                                    this.is_dragging = true;
                                    this.drag_offset = Some(point(
                                        event.position.x - this.position.x,
                                        event.position.y - this.position.y,
                                    ));
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
