use gpui::*;

#[derive(Clone, Debug)]
struct DragState;

#[derive(Clone, Copy)]
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

    fn transform_point(&self, p: Point<Pixels>) -> Point<Pixels> {
        point(
            (p.x - self.center.x) * self.zoom,
            (p.y - self.center.y) * self.zoom,
        )
    }

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

// TextBox Component
#[derive(Clone)]
struct DraggableTextBox {
    text: SharedString,
    position: Point<Pixels>,
    size: Size<Pixels>,
    is_dragging: bool,
    index: usize,
}

impl DraggableTextBox {
    fn new(text: SharedString, position: Point<Pixels>, size: Size<Pixels>, index: usize) -> Self {
        Self {
            text,
            position,
            size,
            is_dragging: false,
            index,
        }
    }

    fn bounds(&self) -> Bounds<Pixels> {
        Bounds::new(self.position, self.size)
    }

    fn overlaps(&self, other: &DraggableTextBox) -> bool {
        let b1 = self.bounds();
        let b2 = other.bounds();

        b1.origin.x < (b2.origin.x + b2.size.width)
            && (b1.origin.x + b1.size.width) > b2.origin.x
            && b1.origin.y < (b2.origin.y + b2.size.height)
            && (b1.origin.y + b1.size.height) > b2.origin.y
    }
}

impl Render for DraggableTextBox {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .absolute()
            .left(self.position.x)
            .top(self.position.y)
            .w(self.size.width)
            .h(self.size.height)
            .bg(rgb(0x2D3142))
            .text_color(rgb(0xFFFFFF))
            .cursor(if self.is_dragging {
                CursorStyle::ClosedHand
            } else {
                CursorStyle::OpenHand
            })
            .id(("textbox", self.index))
            .on_drag(DragState, move |_this, offset, _window, cx| {
                println!("Textbox dragged with offset: {:?}", offset);
                cx.new(|_| EmptyView {})
            })
            .child(self.text.clone())
    }
}

// Main ViewportApp
#[derive(Clone)]
struct ViewportApp {
    textboxes: Vec<Entity<DraggableTextBox>>,
    viewport: Viewport,
    is_dragging: Option<usize>,
    drag_offset: Option<Point<Pixels>>,
    is_panning: bool,
    last_mouse_pos: Option<Point<Pixels>>,
    focus_handle: FocusHandle,
    last_move_direction: Option<Point<Pixels>>,
}

impl ViewportApp {
    fn new(cx: &mut Context<Self>) -> Self {
        let textbox1 = cx.new(|_| {
            DraggableTextBox::new(
                "Hello World".into(),
                point(px(100.0), px(100.0)),
                size(px(200.0), px(100.0)),
                0,
            )
        });

        let textbox2 = cx.new(|_| {
            DraggableTextBox::new(
                "Second Box".into(),
                point(px(400.0), px(300.0)),
                size(px(200.0), px(100.0)),
                1,
            )
        });

        Self {
            textboxes: vec![textbox1, textbox2],
            viewport: Viewport::new(),
            is_dragging: None,
            drag_offset: None,
            is_panning: false,
            last_mouse_pos: None,
            focus_handle: cx.focus_handle(),
            last_move_direction: None,
        }
    }

    fn update_textbox_position(
        &mut self,
        index: usize,
        new_position: Point<Pixels>,
        cx: &mut Context<Self>,
    ) {
        if let Some(textbox) = self.textboxes.get(index) {
            textbox.update(cx, |textbox, cx| {
                let old_position = textbox.position;
                textbox.position = new_position;
                cx.notify();

                let move_delta = point(
                    new_position.x - old_position.x,
                    new_position.y - old_position.y,
                );
                self.last_move_direction = Some(move_delta);
            });

            self.handle_collisions(index, cx);
        }
    }

    fn handle_collisions(&mut self, moving_idx: usize, cx: &mut Context<Self>) {
        if let Some(move_delta) = self.last_move_direction {
            let mut boxes_to_move = Vec::new();

            if let Some(moving_box) = self.textboxes.get(moving_idx) {
                let moving_box_data = moving_box.read(cx);

                for (idx, other_box) in self.textboxes.iter().enumerate() {
                    if idx != moving_idx {
                        let other_box_data = other_box.read(cx);
                        if moving_box_data.overlaps(&other_box_data) {
                            boxes_to_move.push(idx);
                        }
                    }
                }
            }

            for idx in boxes_to_move {
                if let Some(box_to_move) = self.textboxes.get(idx) {
                    box_to_move.update(cx, |textbox, cx| {
                        textbox.position = point(
                            textbox.position.x + move_delta.x,
                            textbox.position.y + move_delta.y,
                        );
                        cx.notify();
                    });
                    self.handle_collisions(idx, cx);
                }
            }
        }
    }
}

impl Focusable for ViewportApp {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ViewportApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let textbox_data: Vec<_> = self
            .textboxes
            .iter()
            .enumerate()
            .map(|(idx, textbox)| {
                let tb = textbox.read(cx);
                (
                    idx,
                    tb.text.clone(),
                    self.viewport.transform_point(tb.position),
                    self.viewport.transform_size(tb.size),
                )
            })
            .collect();

        let viewport = self.viewport;

        div()
            .size_full()
            .bg(rgb(0xEEEEEE))
            .flex()
            .key_context("viewport_app")
            .track_focus(&self.focus_handle(cx))
            .id("viewport_app")
            .on_drag(DragState, move |_this, offset, _window, cx| {
                println!("Canvas dragged with offset: {:?}", offset);
                cx.new(|_| EmptyView {})
            })
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, event: &MouseDownEvent, _window, cx| {
                    if this.is_dragging.is_none() {
                        this.is_panning = true;
                        this.last_mouse_pos = Some(event.position);
                        cx.notify();
                    }
                }),
            )
            .on_drag_move(
                cx.listener(|this, event: &DragMoveEvent<DragState>, _window, cx| {
                    if let Some(drag_idx) = this.is_dragging {
                        if let Some(offset) = this.drag_offset {
                            let screen_pos = event.event.position;
                            let new_screen_pos =
                                point(screen_pos.x - offset.x, screen_pos.y - offset.y);
                            let new_position =
                                this.viewport.inverse_transform_point(new_screen_pos);
                            this.update_textbox_position(drag_idx, new_position, cx);
                        }
                    } else if this.is_panning {
                        if let Some(last_pos) = this.last_mouse_pos {
                            let dx = event.event.position.x - last_pos.x;
                            let dy = event.event.position.y - last_pos.y;

                            this.viewport.center.x -= dx / this.viewport.zoom;
                            this.viewport.center.y -= dy / this.viewport.zoom;
                        }
                        this.last_mouse_pos = Some(event.event.position);
                        cx.notify();
                    }
                }),
            )
            .on_scroll_wheel(cx.listener(|this, event: &ScrollWheelEvent, _window, cx| {
                let old_mouse_world = this.viewport.inverse_transform_point(event.position);

                match event.delta {
                    ScrollDelta::Lines(delta) => {
                        let zoom_delta = if delta.y < 0.0 { 0.9 } else { 1.1 };
                        this.viewport.zoom = (this.viewport.zoom * zoom_delta).max(0.1).min(5.0);
                    }
                    ScrollDelta::Pixels(delta) => {
                        let zoom_delta = if delta.y < px(0.0) { 0.9 } else { 1.1 };
                        this.viewport.zoom = (this.viewport.zoom * zoom_delta).max(0.1).min(5.0);
                    }
                }

                let new_mouse_world = this.viewport.inverse_transform_point(event.position);
                this.viewport.center.x += old_mouse_world.x - new_mouse_world.x;
                this.viewport.center.y += old_mouse_world.y - new_mouse_world.y;

                cx.notify();
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _: &MouseUpEvent, _window, cx| {
                    this.is_dragging = None;
                    this.drag_offset = None;
                    this.is_panning = false;
                    this.last_mouse_pos = None;
                    this.last_move_direction = None;
                    cx.notify();
                }),
            )
            .children(textbox_data.into_iter().map(move |(idx, text, pos, size)| {
                div()
                    .absolute()
                    .left(pos.x)
                    .top(pos.y)
                    .w(size.width)
                    .h(size.height)
                    .bg(rgb(0x2D3142))
                    .text_color(rgb(0xFFFFFF))
                    .text_size(px(16.0 * viewport.zoom))
                    .cursor(CursorStyle::OpenHand)
                    .id(("textbox", idx))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, event: &MouseDownEvent, _window, cx| {
                            this.is_dragging = Some(idx);
                            this.drag_offset =
                                Some(point(event.position.x - pos.x, event.position.y - pos.y));
                            cx.notify();
                        }),
                    )
                    .child(text)
            }))
    }
}

fn main() {
    Application::new().run(|app: &mut App| {
        let bounds = Bounds::centered(None, size(px(800.), px(600.)), app);
        app.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(|cx| ViewportApp::new(cx)),
        )
        .unwrap();
    });
}
