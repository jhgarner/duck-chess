use super::*;

pub fn provide_rev(reverse: bool) {
    provide_context(Reverse(reverse));
}

pub fn use_reverse() -> bool {
    use_context::<Reverse>().0
}

pub fn reversable_grid<Loc: Gridable>(loc: Loc) -> Block {
    let g = loc.into();
    if use_reverse() {
        let y = Loc::height() - g.y;
        Block { y, ..g }
    } else {
        g
    }
}

#[derive(Copy, Clone, Debug)]
struct Reverse(bool);
