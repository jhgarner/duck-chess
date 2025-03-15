use std::hash::Hash;
use std::ops::{Add, Mul};

use serde::{Serialize, de::DeserializeOwned};

use crate::{Castle, Color, Square};

pub trait ChessBoard: Clone + PartialEq + Eq + Default + 'static {
    type Loc: Copy
        + Clone
        + Default
        + std::fmt::Debug
        + Add<Self::Rel, Output = Self::Loc>
        + Serialize
        + DeserializeOwned
        + Hash
        + PartialEq
        + Eq;
    type Rel: Copy
        + Serialize
        + DeserializeOwned
        + Hash
        + PartialEq
        + Eq
        + Default
        + std::fmt::Debug
        + Add<Self::Rel, Output = Self::Rel>
        + Mul<i32, Output = Self::Rel>;

    fn get(&self, i: Self::Loc) -> Option<Square>;
    fn get_mut(&mut self, i: Self::Loc) -> Option<&mut Square>;
    fn iter(&self) -> impl Iterator<Item = (Self::Loc, Square)>;
    fn iter_mut(&mut self) -> impl Iterator<Item = &mut Square>;

    fn knight_moves() -> impl IntoIterator<Item = Self::Rel>;

    fn forward_one(color: Color) -> Self::Rel;
    fn home_for(&self, color: Color, loc: Self::Loc) -> bool;
    fn takeable(&self, color: Color) -> impl IntoIterator<Item = Self::Rel>;
    fn can_promote(&self, color: Color, loc: Self::Loc) -> bool;

    fn rook_dirs(&self) -> impl IntoIterator<Item = Self::Rel>;
    fn castle_rooks(&self) -> impl IntoIterator<Item = Castle<Self::Rel>>;

    fn bishop_dirs(&self) -> impl IntoIterator<Item = Self::Rel>;
}
