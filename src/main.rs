use gpui::*;

actions!(simple_text_box, [ResetZoom, Backspace]);

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

#[derive(Clone)]
struct Viewport {
    zoom: f32,
    center: Point<Pixels>,
}

impl Viewport {
    fn new() -> Self {
        Self {
            zoom: 1.0,
            center: point(px(0.0), px(0.0)),
        }
    }

    // Screen <- transform <- World
    fn transform_point(&self, p: Point<Pixels>) -> Point<Pixels> {
        point(
            (p.x - self.center.x) * self.zoom,
            (p.y - self.center.y) * self.zoom,
        )
    }

    // World <- inverse transform <- Screen
    fn inverse_transform_point(&self, p: Point<Pixels>) -> Point<Pixels> {
        point(
            (p.x / self.zoom) + self.center.x,
            (p.y / self.zoom) + self.center.y,
        )
    }

    fn transform_size(&self, s: Size<Pixels>) -> Size<Pixels> {
        size(s.width * self.zoom, s.height * self.zoom)
    }
}

struct SimpleTextBox {
    textboxes: Vec<TextBoxData>,
    is_dragging: Option<usize>,
    drag_offset: Option<Point<Pixels>>,
    last_move_direction: Option<Point<Pixels>>,
    viewport: Viewport,
    focus_handle: FocusHandle,

    is_panning: bool,
    last_mouse_pos: Option<Point<Pixels>>,
}

impl SimpleTextBox {
    fn new(cx: &mut ViewContext<Self>) -> Self {
        Self {
            textboxes: vec![],
            is_dragging: None,
            drag_offset: None,
            last_move_direction: None,
            viewport: Viewport::new(),
            focus_handle: cx.focus_handle(),
            is_panning: false,
            last_mouse_pos: None,
        }
    }

    fn backspace(&mut self, _: &Backspace, cx: &mut ViewContext<Self>) {
        println!("Backspace pressed!");
        cx.notify();
    }

    fn reset_zoom(&mut self, _: &ResetZoom, cx: &mut ViewContext<Self>) {
        // 1. If there are no textboxes, choose a default
        if self.textboxes.is_empty() {
            self.viewport.zoom = 1.0;
            self.viewport.center = point(px(0.0), px(0.0));
            cx.notify();
            return;
        }

        // 2. Compute the bounding box for all textboxes
        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        let mut min_y = f32::MAX;
        let mut max_y = f32::MIN;

        for tb in &self.textboxes {
            let x1 = tb.position.x.0;
            let x2 = tb.position.x.0 + tb.size.width.0;
            let y1 = tb.position.y.0;
            let y2 = tb.position.y.0 + tb.size.height.0;

            min_x = min_x.min(x1);
            max_x = max_x.max(x2);
            min_y = min_y.min(y1);
            max_y = max_y.max(y2);
        }

        let box_width = (max_x - min_x).max(1.0);
        let box_height = (max_y - min_y).max(1.0);

        // 3. Current window size
        let window_size = cx.window_bounds().get_bounds().size;
        let win_w = window_size.width.0;
        let win_h = window_size.height.0;

        // 4. Center of the bounding box
        let box_center_x = (min_x + max_x) / 2.0;
        let box_center_y = (min_y + max_y) / 2.0;

        // 5. Differentiate single-box vs multiple-box logic
        let count = self.textboxes.len();

        if count == 1 {
            //
            // For a single box, try to scale it so that it fills ~60% of the window.
            // If the box is large, we clamp further.
            //
            let fill_ratio = 0.60; // 60% of window
            let fit_zoom_x = (win_w * fill_ratio) / box_width;
            let fit_zoom_y = (win_h * fill_ratio) / box_height;

            let best_zoom = fit_zoom_x.min(fit_zoom_y);
            // Final clamp so we don’t overshoot
            let new_zoom = best_zoom.clamp(0.1, 3.0);

            self.viewport.zoom = new_zoom;
            // Shift so bounding box center is at window center
            self.viewport.center = point(
                px(box_center_x - (win_w / 2.0) / new_zoom),
                px(box_center_y - (win_h / 2.0) / new_zoom),
            );
        } else {
            //
            // For multiple boxes, do a straightforward "zoom to fit" approach:
            // A small margin ensures there’s some space around them.
            //
            let margin_ratio = 0.10; // 10% margin on each side
            let margin_factor = 1.0 / (1.0 - 2.0 * margin_ratio);

            let inflated_width = box_width * margin_factor;
            let inflated_height = box_height * margin_factor;

            let fit_zoom_x = win_w / inflated_width;
            let fit_zoom_y = win_h / inflated_height;
            let best_zoom = fit_zoom_x.min(fit_zoom_y);
            let new_zoom = best_zoom.clamp(0.1, 3.0);

            self.viewport.zoom = new_zoom;
            self.viewport.center = point(
                px(box_center_x - (win_w / 2.0) / new_zoom),
                px(box_center_y - (win_h / 2.0) / new_zoom),
            );
        }

        cx.notify();
    }

    fn spawn_new_textbox(&mut self, position: Point<Pixels>) {
        self.textboxes.push(TextBoxData {
            text: "New Note".into(),
            position,
            size: size(px(300.0), px(64.0)),
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

impl FocusableView for SimpleTextBox {
    fn focus_handle(&self, _: &AppContext) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for SimpleTextBox {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .size_full()
            .bg(rgb(0xEEEEEE))
            .flex()
            .key_context("simple_text_box")
            .track_focus(&self.focus_handle(cx))
            .on_action(cx.listener(Self::reset_zoom))
            .on_action(cx.listener(Self::backspace))
            // Initiate panning on blank canvas
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, event: &MouseDownEvent, _cx| {
                    if this.is_dragging.is_none() {
                        this.is_panning = true;
                        this.last_mouse_pos = Some(event.position);
                    }
                }),
            )
            // Handle dragging textboxes or panning
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, cx| {
                if let Some(drag_idx) = this.is_dragging {
                    if let Some(offset) = this.drag_offset {
                        if let Some(textbox) = this.textboxes.get_mut(drag_idx) {
                            let old_position = textbox.position;
                            let event_pos = this.viewport.inverse_transform_point(event.position);
                            let new_position =
                                point(event_pos.x - offset.x, event_pos.y - offset.y);

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
                } else if this.is_panning {
                    if let Some(last_pos) = this.last_mouse_pos {
                        let dx = event.position.x - last_pos.x;
                        let dy = event.position.y - last_pos.y;

                        this.viewport.center.x -= dx / this.viewport.zoom;
                        this.viewport.center.y -= dy / this.viewport.zoom;
                    }
                    this.last_mouse_pos = Some(event.position);
                    cx.notify();
                }
            }))
            // Zoom at mouse cursor
            .on_scroll_wheel(cx.listener(|this, event: &ScrollWheelEvent, cx| {
                match event.delta {
                    ScrollDelta::Lines(delta) => {
                        // Pin the mouse location in world space before zoom
                        let old_mouse_world = this.viewport.inverse_transform_point(event.position);

                        let zoom_delta = if delta.y < 0.0 { 0.9 } else { 1.1 };
                        this.viewport.zoom = (this.viewport.zoom * zoom_delta).max(0.1).min(5.0);

                        // Pin the mouse location in world space after zoom
                        let new_mouse_world = this.viewport.inverse_transform_point(event.position);

                        // Shift the center so that these two coincide
                        this.viewport.center.x += old_mouse_world.x - new_mouse_world.x;
                        this.viewport.center.y += old_mouse_world.y - new_mouse_world.y;

                        cx.notify();
                    }
                    ScrollDelta::Pixels(delta) => {
                        let old_mouse_world = this.viewport.inverse_transform_point(event.position);

                        let zoom_delta = if delta.y < px(0.0) { 0.9 } else { 1.1 };
                        this.viewport.zoom = (this.viewport.zoom * zoom_delta).max(0.1).min(5.0);

                        let new_mouse_world = this.viewport.inverse_transform_point(event.position);

                        this.viewport.center.x += old_mouse_world.x - new_mouse_world.x;
                        this.viewport.center.y += old_mouse_world.y - new_mouse_world.y;

                        cx.notify();
                    }
                }
            }))
            // Release drag/panning on mouse up
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _: &MouseUpEvent, _cx| {
                    this.is_dragging = None;
                    this.drag_offset = None;
                    this.last_move_direction = None;
                    this.is_panning = false;
                    this.last_mouse_pos = None;
                }),
            )
            .children({
                let mut elements = Vec::new();

                // Textbox elements
                elements.extend(self.textboxes.iter().enumerate().map(|(idx, textbox)| {
                    let transformed_pos = self.viewport.transform_point(textbox.position);
                    let transformed_size = self.viewport.transform_size(textbox.size);

                    div()
                        .absolute()
                        .left(transformed_pos.x)
                        .top(transformed_pos.y)
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .w(transformed_size.width)
                                .h(px(24.0 * self.viewport.zoom))
                                .bg(rgb(0x2D3142))
                                .rounded_t_md()
                                .flex()
                                .justify_between()
                                .items_center()
                                .px(px(8.0 * self.viewport.zoom))
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
                                            let event_pos = this
                                                .viewport
                                                .inverse_transform_point(event.position);
                                            this.drag_offset = Some(point(
                                                event_pos.x - textbox.position.x,
                                                event_pos.y - textbox.position.y,
                                            ));
                                        }
                                    }),
                                )
                                .child(
                                    div()
                                        .text_size(px(14.0 * self.viewport.zoom))
                                        .flex()
                                        .justify_between()
                                        .items_center()
                                        .w_full()
                                        .child("Notes")
                                        .child(div().flex().gap_2().children(vec![
                                                // Close button
                                                div()
                                                    .text_size(px(16.0 * self.viewport.zoom))
                                                    .w(px(16.0 * self.viewport.zoom))
                                                    .h(px(16.0 * self.viewport.zoom))
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
                                                    .text_size(px(16.0 * self.viewport.zoom))
                                                    .w(px(16.0 * self.viewport.zoom))
                                                    .h(px(16.0 * self.viewport.zoom))
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
                                .text_size(px(16.0 * self.viewport.zoom))
                                .w(transformed_size.width)
                                .h(px(40.0 * self.viewport.zoom))
                                .bg(white())
                                .rounded_b_md()
                                .shadow_md()
                                .px(px(8.0 * self.viewport.zoom))
                                .flex()
                                .items_center()
                                .child(textbox.text.clone()),
                        )
                }));

                // "Add" button in the center if there are no textboxes
                if self.textboxes.is_empty() {
                    elements.push(
                        div()
                            .relative()
                            .size_full()
                            .flex()
                            .justify_center()
                            .items_center()
                            .child(
                                div()
                                    .w(px(50.0 * self.viewport.zoom))
                                    .h(px(50.0 * self.viewport.zoom))
                                    .bg(rgb(0x4CAF50))
                                    .rounded_full()
                                    .cursor(CursorStyle::PointingHand)
                                    .flex()
                                    .justify_center()
                                    .items_center()
                                    .text_color(rgb(0xFFFFFF))
                                    .text_size(px(20.0 * self.viewport.zoom))
                                    .child("+")
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, event: &MouseDownEvent, cx| {
                                            let pos = this
                                                .viewport
                                                .inverse_transform_point(event.position);
                                            this.spawn_new_textbox(point(
                                                pos.x - px(150.),
                                                pos.y - px(32.),
                                            ));
                                            cx.notify();
                                        }),
                                    ),
                            ),
                    );
                }

                elements
            })
    }
}

fn main() {
    App::new().run(|cx: &mut AppContext| {
        cx.bind_keys([
            KeyBinding::new("backspace", Backspace, None),
            KeyBinding::new("ctrl-0", ResetZoom, None),
        ]);

        let bounds = Bounds::centered(None, size(px(800.), px(600.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: None,
                window_decorations: Some(WindowDecorations::Client),
                is_movable: true,
                ..Default::default()
            },
            |cx| {
                let view = cx.new_view(|cx| SimpleTextBox::new(cx));
                cx.focus_view(&view);
                view
            },
        )
        .unwrap();

        cx.activate(true);
    });
}
