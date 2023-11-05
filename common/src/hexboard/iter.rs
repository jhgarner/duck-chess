use super::{Hexboard, Coord, Dir, ZERO_DIR};

pub struct HexIter<'r, T> {
    pub board: &'r Hexboard<T>,
    pub at: Coord,
    pub dir: Dir,
}

impl<'r, T> Iterator for HexIter<'r, T> {
    type Item = (Coord, &'r T);

    fn next(&mut self) -> Option<Self::Item> {
        self.at = self.at + self.dir;
        Some((self.at, self.board.get(self.at)?))
    }
}

pub struct HexIterMut<'r, T> {
    pub board: &'r mut Hexboard<T>,
    pub at: Coord,
    pub dir: Dir,
}

impl<'r, T> Iterator for HexIterMut<'r, T> {
    type Item = (Coord, &'r mut T);

    fn next(&mut self) -> Option<Self::Item> {
        if self.dir == ZERO_DIR {
            panic!("Attempt to mutably borrow the same element many times!")
        } else {
            self.at = self.at + self.dir;
            let as_mut_ref = self.board.get_mut(self.at)?;
            let as_mut_ptr = as_mut_ref as *mut T;
            // Safety: Because the direction vector is non-zero, each iteration will return a
            // unique element. Since every element is guaranteed to be unique, these mutable
            // references will never alias with each other.
            // Since the returned reference has the same lifetime as the board's mutable reference,
            // we know these mutable references can't alias with previous or future references
            // created outside the iterator.
            unsafe {
                Some((self.at, &mut *as_mut_ptr))
            }
        }
    }
}

