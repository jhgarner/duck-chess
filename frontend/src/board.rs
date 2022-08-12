use std::collections::HashMap;

use common::*;
use egui::{Pos2, Rect, Sense, TextureHandle, Ui, Vec2};
use rust_embed::RustEmbed;

use crate::{app::{Message, MessageChannel}, square::BoardSquare, svg::load_svg};

#[derive(RustEmbed)]
#[folder = "assets/"]
struct PieceImages;

#[derive(Debug, Hash, Copy, Clone, PartialEq, Eq)]
pub enum ImageKey {
    Duck,
    Piece(Color, Piece),
}

pub type Images = HashMap<ImageKey, TextureHandle>;

pub struct BoardDrawer {
    piece_images: Images,
}

pub struct BoardSquareGrid<'a>(Vec<Vec<BoardSquare<'a>>>);

impl<'a> BoardSquareGrid<'a> {
    pub fn get_mut(&mut self, loc: Loc) -> &mut BoardSquare<'a> {
        &mut self.0[loc.down][loc.right]
    }

    pub fn iter(&'a self) -> impl Iterator<Item = &BoardSquare<'a>> {
        self.0.iter().flat_map(|row| row.iter())
    }

    pub fn iter_mut<'b>(&'b mut self) -> impl Iterator<Item = &'b mut BoardSquare<'a>>
    where
        'a: 'b,
    {
        self.0.iter_mut().flat_map(|row| row.iter_mut())
    }
}

impl BoardDrawer {
    pub fn new(cc: &eframe::CreationContext<'_>) -> BoardDrawer {
        let mut piece_images = Images::default();

        for piece in Piece::all() {
            for color in [Color::Black, Color::White] {
                let name = format!("{}{}.svg", color.short_name(), piece.short_name());
                let svg = PieceImages::get(&name).unwrap();
                let color_image = load_svg(&svg.data).unwrap();
                let handle = cc.egui_ctx.load_texture(name, color_image.clone());
                piece_images.insert(ImageKey::Piece(color, piece), handle);
            }
        }

        let svg = PieceImages::get("duck.svg").unwrap();
        let color_image = load_svg(&svg.data).unwrap();
        let handle = cc.egui_ctx.load_texture("duck", color_image.clone());
        piece_images.insert(ImageKey::Duck, handle);

        BoardDrawer { piece_images }
    }

    // The board needs to have the following properties:
    // 1. Adapt to any screen size
    // 2. Draw without artifacts caused by f32->pixel conversions
    // 3. Detect interactions at every cell
    pub fn layout<'a, const N: usize, const M: usize>(
        &'a self,
        ui: &mut Ui,
        game: &[[Square; N]; M],
        message: &MessageChannel,
    ) -> BoardSquareGrid<'a> {
        let width = ui.available_width();
        let height = ui.available_height();
        let board_width = M as f32;
        let board_height = N as f32;
        let board_ratio = board_width / board_height;
        let view_ratio = width / height;
        let (size, unit) = if board_ratio > view_ratio {
            (width, width / board_width)
        } else {
            (height, height / board_height)
        };
        let start = ui.next_widget_position();
        let rect = Rect {
            min: start,
            max: start + Vec2::new(size, size),
        };
        let x = rect.left_top().x;
        let y = rect.left_top().y;

        let mut squares: Vec<Vec<BoardSquare<'a>>> = Vec::new();

        let mut min = Pos2::new(x, y);
        min = ui.painter().round_pos_to_pixels(min);

        for (down, row) in game.iter().enumerate() {
            let mut board_row: Vec<BoardSquare<'a>> = Vec::new();
            for (right, square) in row.iter().enumerate() {
                let painter = ui.painter();

                let max = Pos2::new(min.x + unit, min.y + unit);
                let max = painter.round_pos_to_pixels(max);

                let rect = Rect { min, max };
                let even = (down + right) % 2 == 0;
                let board_square = BoardSquare::new(rect, even, *square, &self.piece_images);

                board_row.push(board_square);

                let response = ui.allocate_rect(rect, Sense::click());
                if response.clicked() {
                    message.write(Message::SpaceClicked(Loc::new(right, down)));
                }

                min = Pos2::new(max.x, min.y);
                min = ui.painter().round_pos_to_pixels(min);
            }
            squares.push(board_row);

            min = Pos2::new(x, min.y + unit);
            min = ui.painter().round_pos_to_pixels(min);
        }

        BoardSquareGrid(squares)
    }
}
