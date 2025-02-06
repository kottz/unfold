use gpui::*;

#[derive(Clone, Debug)]
struct DragState;

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

#[derive(Clone)]
struct TextBox {
    text: SharedString,
    position: Point<Pixels>,
    size: Size<Pixels>,
}

#[derive(Clone)]
struct ViewportApp {
    textboxes: Vec<TextBox>,
    viewport: Viewport,
    is_dragging: Option<usize>,
    drag_offset: Option<Point<Pixels>>,
    is_panning: bool,
    last_mouse_pos: Option<Point<Pixels>>,
    focus_handle: FocusHandle,
}

impl ViewportApp {
    fn new(cx: &mut ViewContext<Self>) -> Self {
        Self {
            textboxes: vec![
                TextBox {
                    text: "Hello World".into(),
                    position: point(px(100.0), px(100.0)),
                    size: size(px(200.0), px(100.0)),
                },
                TextBox {
                    text: "Second Box".into(),
                    position: point(px(400.0), px(300.0)),
                    size: size(px(200.0), px(100.0)),
                },
            ],
            viewport: Viewport::new(),
            is_dragging: None,
            drag_offset: None,
            is_panning: false,
            last_mouse_pos: None,
            focus_handle: cx.focus_handle(),
        }
    }
}

impl FocusableView for ViewportApp {
    fn focus_handle(&self, _: &AppContext) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ViewportApp {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .size_full()
            .bg(rgb(0xEEEEEE))
            .flex()
            .key_context("viewport_app")
            .track_focus(&self.focus_handle(cx))
            .id("viewport_app")
            .on_drag(DragState, move |_this, _, cx| {
                println!("Dragged!");
                cx.new_view(|_| EmptyView {})
            })
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, event: &MouseDownEvent, _cx| {
                    if this.is_dragging.is_none() {
                        this.is_panning = true;
                        this.last_mouse_pos = Some(event.position);
                    }
                }),
            )
            .on_drag_move(cx.listener(|this, event: &DragMoveEvent<DragState>, cx| {
                if let Some(drag_idx) = this.is_dragging {
                    if let Some(offset) = this.drag_offset {
                        if let Some(textbox) = this.textboxes.get_mut(drag_idx) {
                            let event_pos = this.viewport.inverse_transform_point(event.event.position);
                            textbox.position = point(
                                event_pos.x - offset.x,
                                event_pos.y - offset.y,
                            );
                            cx.notify();
                        }
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
            }))
            .on_scroll_wheel(cx.listener(|this, event: &ScrollWheelEvent, cx| {
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
            .on_mouse_up(MouseButton::Left, cx.listener(|this, _: &MouseUpEvent, _cx| {
                this.is_dragging = None;
                this.drag_offset = None;
                this.is_panning = false;
                this.last_mouse_pos = None;
            }))
            .children(
                self.textboxes
                    .iter()
                    .enumerate()
                    .map(|(idx, textbox)| {
                        let transformed_pos = self.viewport.transform_point(textbox.position);
                        let transformed_size = self.viewport.transform_size(textbox.size);
                        
                        div()
                            .absolute()
                            .left(transformed_pos.x)
                            .top(transformed_pos.y)
                            .w(transformed_size.width)
                            .h(transformed_size.height)
                            .bg(rgb(0x2D3142))
                            .text_color(rgb(0xFFFFFF))
                            .cursor(if self.is_dragging == Some(idx) {
                                CursorStyle::ClosedHand
                            } else {
                                CursorStyle::OpenHand
                            })
                            .id("textboxhaha")
                            .on_drag(DragState, move |_this, _, cx| {
                                println!("Textbox dragged!");
                                cx.new_view(|_| EmptyView {})
                            })
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, event: &MouseDownEvent, _cx| {
                                    this.is_dragging = Some(idx);
                                    if let Some(textbox) = this.textboxes.get(idx) {
                                        let event_pos = this.viewport.inverse_transform_point(event.position);
                                        this.drag_offset = Some(point(
                                            event_pos.x - textbox.position.x,
                                            event_pos.y - textbox.position.y,
                                        ));
                                    }
                                }),
                            )
                            .child(textbox.text.clone())
                    })
                    .collect::<Vec<_>>(),
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
            |cx| cx.new_view(|cx| ViewportApp::new(cx)),
        )
        .unwrap();
    });
}
