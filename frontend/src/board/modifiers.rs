use super::*;

#[derive(Debug)]
pub struct Mods<Loc>(Rc<ModsInner<Loc>>);

impl<Loc: 'static + std::fmt::Debug> Mods<Loc> {
    pub fn new(active: Active<Loc>, targets: HashSet<Loc>) -> Self {
        Mods(Rc::new(ModsInner { active, targets }))
    }
}

impl<Loc> Clone for Mods<Loc> {
    fn clone(&self) -> Self {
        Mods(self.0.clone())
    }
}

impl<Loc> Default for Mods<Loc> {
    fn default() -> Self {
        Self(Rc::new(ModsInner {
            active: Active::default(),
            targets: HashSet::default(),
        }))
    }
}

impl<Loc: Eq + Hash> PartialEq for Mods<Loc> {
    fn eq(&self, other: &Self) -> bool {
        self.0.active == other.0.active && self.0.targets == other.0.targets
    }
}

#[derive(Debug)]
struct ModsInner<Loc> {
    active: Active<Loc>,
    targets: HashSet<Loc>,
}

#[component]
pub fn ActiveHighlight<Loc: PartialEq + Clone + 'static>(at: Loc) -> Element {
    let ModsInner { active, .. } = &*use_mods();
    if *active == Active::Active(at) {
        rsx!(div { class: "active" })
    } else {
        rsx!()
    }
}

#[component]
pub fn TargetHighlight<Loc: Eq + Hash + 'static>(at: Loc) -> Element {
    let ModsInner { targets, .. } = &*use_mods::<Loc>();
    if targets.contains(&at) {
        rsx!(div { class: "target" })
    } else {
        rsx!()
    }
}

pub fn provide_mods<Loc: 'static>(mods: Mods<Loc>) {
    let signal = with_signal(mods.clone());
    provide_context(signal);
}

fn use_mods<Loc: 'static>() -> Rc<ModsInner<Loc>> {
    use_context::<Signal<Mods<Loc>>>()().0
}
