use gpui::*;

#[derive(Clone, Debug)]
pub struct DragState;

#[derive(Clone, Copy)]
pub struct Viewport {
    pub zoom: f32,
    pub center: Point<Pixels>,
}

impl Viewport {
    pub fn new() -> Self {
        Self {
            zoom: 1.0,
            center: point(px(0.0), px(0.0)),
        }
    }

    pub fn transform_point(&self, p: Point<Pixels>) -> Point<Pixels> {
        point(
            (p.x - self.center.x) * self.zoom,
            (p.y - self.center.y) * self.zoom,
        )
    }

    pub fn inverse_transform_point(&self, p: Point<Pixels>) -> Point<Pixels> {
        point(
            (p.x / self.zoom) + self.center.x,
            (p.y / self.zoom) + self.center.y,
        )
    }

    pub fn transform_size(&self, s: Size<Pixels>) -> Size<Pixels> {
        size(s.width * self.zoom, s.height * self.zoom)
    }
}
