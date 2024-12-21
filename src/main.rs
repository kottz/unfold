use gpui::*;

#[derive(Clone)]
struct TextBoxData {
    text: SharedString,
    position: Point<Pixels>,
    size: Size<Pixels>,
}

impl TextBoxData {
    fn bounds(&self) -> Bounds<Pixels> {
        Bounds::new(self.position, self.size)
    }

    fn overlaps(&self, other: &TextBoxData) -> bool {
        let b1 = self.bounds();
        let b2 = other.bounds();

        b1.origin.x < (b2.origin.x + b2.size.width)
            && (b1.origin.x + b1.size.width) > b2.origin.x
            && b1.origin.y < (b2.origin.y + b2.size.height)
            && (b1.origin.y + b1.size.height) > b2.origin.y
    }
}

struct SimpleTextBox {
    textboxes: Vec<TextBoxData>,
    is_dragging: Option<usize>,
    drag_offset: Option<Point<Pixels>>,
    last_move_direction: Option<Point<Pixels>>,
}

impl SimpleTextBox {
    fn new(_cx: &mut ViewContext<Self>) -> Self {
        Self {
            textboxes: vec![],
            is_dragging: None,
            drag_offset: None,
            last_move_direction: None,
        }
    }

    fn spawn_new_textbox(&mut self, position: Point<Pixels>) {
        self.textboxes.push(TextBoxData {
            text: "New Note".into(),
            position,
            size: size(px(300.), px(64.)),
        });
    }

    fn remove_textbox(&mut self, index: usize) {
        self.textboxes.remove(index);
    }

    fn handle_collision(&mut self, moving_idx: usize, move_delta: Point<Pixels>) {
        let mut boxes_to_move = vec![];
        let moving_box = &self.textboxes[moving_idx];

        for (idx, other_box) in self.textboxes.iter().enumerate() {
            if idx != moving_idx && moving_box.overlaps(other_box) {
                boxes_to_move.push(idx);
            }
        }

        for idx in boxes_to_move {
            if let Some(box_to_move) = self.textboxes.get_mut(idx) {
                box_to_move.position = point(
                    box_to_move.position.x + move_delta.x,
                    box_to_move.position.y + move_delta.y,
                );
                self.handle_collision(idx, move_delta);
            }
        }
    }
}

impl Render for SimpleTextBox {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let window_size = cx.window_bounds().get_bounds().size;

        div()
            .size_full()
            .bg(rgb(0xEEEEEE))
            .flex()
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, cx| {
                if let Some(drag_idx) = this.is_dragging {
                    if let Some(offset) = this.drag_offset {
                        if let Some(textbox) = this.textboxes.get_mut(drag_idx) {
                            let old_position = textbox.position;
                            let new_position =
                                point(event.position.x - offset.x, event.position.y - offset.y);

                            let move_delta = point(
                                new_position.x - old_position.x,
                                new_position.y - old_position.y,
                            );

                            textbox.position = new_position;
                            this.last_move_direction = Some(move_delta);

                            this.handle_collision(drag_idx, move_delta);

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
                    this.last_move_direction = None;
                }),
            )
            .children({
                let mut elements = Vec::new();

                // Add textboxes
                elements.extend(self.textboxes.iter().enumerate().map(|(idx, textbox)| {
                    div()
                        .absolute()
                        .left(textbox.position.x)
                        .top(textbox.position.y)
                        .flex()
                        .flex_col()
                        .child(
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
                                    div()
                                        .flex()
                                        .justify_between()
                                        .items_center()
                                        .w_full()
                                        .child("Notes")
                                        .child(div().flex().gap_2().children(vec![
                                                // Close button
                                                div()
                                                    .w(px(16.))
                                                    .h(px(16.))
                                                    .bg(rgb(0xFF5252))
                                                    .rounded_full()
                                                    .cursor(CursorStyle::PointingHand)
                                                    .flex()
                                                    .justify_center()
                                                    .items_center()
                                                    .text_color(rgb(0xFFFFFF))
                                                    .child("×")
                                                    .on_mouse_down(
                                                        MouseButton::Left,
                                                        cx.listener(
                                                            move |this, _: &MouseDownEvent, cx| {
                                                                this.remove_textbox(idx);
                                                                cx.notify();
                                                            },
                                                        ),
                                                    ),
                                                // Add button
                                                div()
                                                    .w(px(16.))
                                                    .h(px(16.))
                                                    .bg(rgb(0x4CAF50))
                                                    .rounded_full()
                                                    .cursor(CursorStyle::PointingHand)
                                                    .flex()
                                                    .justify_center()
                                                    .items_center()
                                                    .text_color(rgb(0xFFFFFF))
                                                    .child("+")
                                                    .on_mouse_down(
                                                        MouseButton::Left,
                                                        cx.listener(
                                                            move |this, _: &MouseDownEvent, cx| {
                                                                if let Some(textbox) =
                                                                    this.textboxes.get(idx)
                                                                {
                                                                    this.spawn_new_textbox(point(
                                                                        textbox.position.x
                                                                            + px(320.),
                                                                        textbox.position.y,
                                                                    ));
                                                                    cx.notify();
                                                                }
                                                            },
                                                        ),
                                                    ),
                                            ])),
                                ),
                        )
                        .child(
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
                }));

                // Add circular add button when no textboxes exist
                if self.textboxes.is_empty() {
                    elements.push(
                        div()
                            .absolute()
                            .left(window_size.width / 2.0 - px(25.))
                            .top(window_size.height / 2.0 - px(25.))
                            .w(px(50.))
                            .h(px(50.))
                            .bg(rgb(0x4CAF50))
                            .rounded_full()
                            .cursor(CursorStyle::PointingHand)
                            .flex()
                            .justify_center()
                            .items_center()
                            .text_color(rgb(0xFFFFFF))
                            .text_xl()
                            .child("+")
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, event: &MouseDownEvent, cx| {
                                    this.spawn_new_textbox(point(
                                        event.position.x - px(150.), // Center the textbox
                                        event.position.y - px(32.),  // Center the textbox
                                    ));
                                    cx.notify();
                                }),
                            ),
                    );
                }

                elements
            })
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
