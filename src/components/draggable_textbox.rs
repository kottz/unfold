use super::text_field::TextField;
use gpui::*;

#[derive(Clone)]
pub struct DraggableTextBox {
    pub textfield: Entity<TextField>,
    pub position: Point<Pixels>,
    pub size: Size<Pixels>,
    pub is_dragging: bool,
    pub index: usize,
}

impl DraggableTextBox {
    pub fn new(
        initial_text: SharedString,
        position: Point<Pixels>,
        size: Size<Pixels>,
        index: usize,
        cx: &mut Context<Self>,
    ) -> Self {
        let textfield = cx.new(|cx| TextField::new(initial_text, cx));
        Self {
            textfield,
            position,
            size,
            is_dragging: false,
            index,
        }
    }

    pub fn bounds(&self) -> Bounds<Pixels> {
        Bounds::new(self.position, self.size)
    }

    pub fn overlaps(&self, other: &DraggableTextBox) -> bool {
        let b1 = self.bounds();
        let b2 = other.bounds();

        b1.origin.x < (b2.origin.x + b2.size.width)
            && (b1.origin.x + b1.size.width) > b2.origin.x
            && b1.origin.y < (b2.origin.y + b2.size.height)
            && (b1.origin.y + b1.size.height) > b2.origin.y
    }
}

impl Render for DraggableTextBox {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
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
            .child(self.textfield.clone())
    }
}
