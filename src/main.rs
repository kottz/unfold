use gpui::*;

#[derive(Clone)]
struct TextBoxData {
    text: SharedString,
    position: Point<Pixels>,
}

struct SimpleTextBox {
    textboxes: Vec<TextBoxData>,
    is_dragging: Option<usize>, // Index of the textbox being dragged
    drag_offset: Option<Point<Pixels>>,
}

impl SimpleTextBox {
    fn new(_cx: &mut ViewContext<Self>) -> Self {
        Self {
            textboxes: vec![TextBoxData {
                text: "Hello World".into(),
                position: point(px(250.), px(250.)),
            }],
            is_dragging: None,
            drag_offset: None,
        }
    }

    fn spawn_new_textbox(&mut self, base_position: Point<Pixels>) {
        self.textboxes.push(TextBoxData {
            text: "New Note".into(),
            position: point(base_position.x + px(320.), base_position.y), // 300px width + 20px gap
        });
    }
}

impl Render for SimpleTextBox {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .size_full()
            .bg(rgb(0xEEEEEE))
            .flex()
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, cx| {
                if let Some(drag_idx) = this.is_dragging {
                    if let Some(offset) = this.drag_offset {
                        if let Some(textbox) = this.textboxes.get_mut(drag_idx) {
                            textbox.position =
                                point(event.position.x - offset.x, event.position.y - offset.y);
                            cx.notify();
                        }
                    }
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _: &MouseUpEvent, _cx| {
                    this.is_dragging = None;
                    this.drag_offset = None;
                }),
            )
            .children(self.textboxes.iter().enumerate().map(|(idx, textbox)| {
                div()
                    .absolute()
                    .left(textbox.position.x)
                    .top(textbox.position.y)
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
                            .justify_between()
                            .items_center()
                            .px_2()
                            .text_color(rgb(0xFFFFFF))
                            .text_sm()
                            .cursor(if self.is_dragging == Some(idx) {
                                CursorStyle::ClosedHand
                            } else {
                                CursorStyle::OpenHand
                            })
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, event: &MouseDownEvent, _cx| {
                                    this.is_dragging = Some(idx);
                                    if let Some(textbox) = this.textboxes.get(idx) {
                                        this.drag_offset = Some(point(
                                            event.position.x - textbox.position.x,
                                            event.position.y - textbox.position.y,
                                        ));
                                    }
                                }),
                            )
                            .child(
                                // Title and add button container
                                div()
                                    .flex()
                                    .justify_between()
                                    .items_center()
                                    .w_full()
                                    .child("Notes")
                                    .child(
                                        // Add button
                                        div()
                                            .w(px(16.))
                                            .h(px(16.))
                                            .bg(rgb(0x4CAF50)) // Green color
                                            .rounded_full()
                                            .cursor(CursorStyle::PointingHand)
                                            .flex()
                                            .justify_center()
                                            .items_center()
                                            .text_color(rgb(0xFFFFFF))
                                            .child("+")
                                            .on_mouse_down(
                                                MouseButton::Left,
                                                cx.listener(move |this, _: &MouseDownEvent, cx| {
                                                    if let Some(textbox) = this.textboxes.get(idx) {
                                                        this.spawn_new_textbox(textbox.position);
                                                        cx.notify();
                                                    }
                                                }),
                                            ),
                                    ),
                            ),
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
                            .child(textbox.text.clone()),
                    )
            }))
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
