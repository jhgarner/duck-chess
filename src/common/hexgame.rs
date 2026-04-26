// use crate::{
//     game::GameRaw,
//     hexboard::{Coord, Hexboard},
//     Square,
// };
//
// type Hexgame = GameRaw<Hexboard<Square>>;
//
// struct MoveBuilder {
//     taking_moves: Vec<Dir>,
//     nontaking_moves: Vec<Dir>,
// }
//
// impl MoveBuilder {
//     pub fn and(mut self, rhs: impl Into<MoveBuilder>) -> MoveBuilder {
//         let mut rhs: MoveBuilder = rhs.into();
//         self.taking_moves.append(&mut rhs.taking_moves);
//         self.nontaking_moves.append(&mut rhs.nontaking_moves);
//         self
//     }
//
//     pub fn nontaking(mut self) -> MoveBuilder {
//         self.nontaking_moves.append(&mut self.taking_moves);
//         self
//     }
//
//     pub fn from(self, start: Coord, game: Hexgame) -> Vec<Action> {
//         let mut actions = vec![];
//         for dir in self.nontaking_moves {
//             for (coord, square) in board.iter(from, dir) {
//                 if let Piece {color, ..} = square {
//                     if game.turn() != color {
//                         actions.push(Action::Move())
//                     }
//                 }
//             }
//         }
//     }
// }
//
// impl From<Dir> for MoveBuilder {
//     fn from(value: Dir) -> Self {
//         MoveBuilder {
//             taking_moves: vec![value],
//             nontaking_moves: vec![],
//         }
//     }
// }
//
// impl Hexgame {
//     fn taking(&self, iter: impl Iterator<Item = (Coord, &Hexcell)>) -> Option<Coord> {
//         if let Some((coord, Hexcell::Piece {color, ..})) = iter.next() {
//             if color != self.turn() {
//                 Some(coord)
//             } else {
//                 None
//             }
//         }
//     }
//     pub fn valid_moves_from(&self, start: Coord) -> Vec<Coord> {
//         if let Some(Hexcell::Piece {piece, ..}) = self.board.get(start) {
//             match piece {
//                 Piece::Queen => self.board.iter(start, )
//             }
//             for (coord, square) in self.board.iter(start, )
//             todo!()
//         } else {
//             vec![]
//         }
//
//     }
// }
