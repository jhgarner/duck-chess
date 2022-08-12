use common::Square;
use egui::{Color32, Image, Rect, Stroke, Ui};

use crate::board::{Images, ImageKey};

pub struct BoardSquare<'a> {
    pub even: bool,
    pub rect: Rect,
    pub active: bool,
    pub actionable: bool,
    pub square: Square,
    pub piece_images: &'a Images,
}

impl<'a> BoardSquare<'a> {
    pub fn new(
        rect: Rect,
        even: bool,
        square: Square,
        piece_images: &'a Images,
    ) -> BoardSquare<'a> {
        BoardSquare {
            even,
            rect,
            active: false,
            actionable: false,
            square,
            piece_images,
        }
    }

    pub fn draw(&self, ui: &mut Ui) {
        let rect = self.rect;
        let painter = ui.painter().with_clip_rect(rect);

        if self.even {
            painter.rect_filled(rect, 0.0, Color32::LIGHT_GRAY)
        } else {
            painter.rect_filled(rect, 0.0, Color32::BROWN)
        }

        if self.active {
            painter.rect_stroke(
                rect,
                0.0,
                Stroke::new(rect.size().x / 10.0, Color32::LIGHT_BLUE),
            );
        }

        match self.square {
            Square::Piece(color, piece) => {
                let texture = self.piece_images.get(&ImageKey::Piece(color, piece)).unwrap();
                let image = Image::new(texture, rect.size());
                image.paint_at(ui, rect);
            }
            Square::Duck => {
                let texture = self.piece_images.get(&ImageKey::Duck).unwrap();
                let image = Image::new(texture, rect.size());
                image.paint_at(ui, rect);
            }
            _ => {}
        }

        if self.actionable {
            painter.circle_filled(
                rect.center(),
                rect.height() * 0.25,
                Color32::GREEN,
            );
        }
    }
}
