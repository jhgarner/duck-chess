mod bordered;
mod specular;

use bordered::*;
use specular::*;

use super::*;

#[derive_where(PartialEq; Loc: Eq + Hash)]
#[derive_where(Default)]
#[derive(Debug)]
pub struct Mods<Loc>(Rc<ModsInner<Loc>>);

impl<Loc: 'static + Hash + Eq> Mods<Loc> {
    pub fn new(targetting: Vec<Targetting<Loc>>, dangers: HashSet<Loc>) -> Self {
        let mut actives = HashMap::new();
        let mut all_targets = HashMap::new();
        for Targetting {
            targets,
            active,
            target_type,
        } in targetting
        {
            if let Active::Active(loc) = active {
                actives.insert(loc, target_type);
            }
            all_targets.extend(targets.into_iter().map(|l| (l, target_type)));
        }
        Mods(Rc::new(ModsInner {
            actives,
            all_targets,
            dangers,
        }))
    }
}

impl<Loc> Clone for Mods<Loc> {
    fn clone(&self) -> Self {
        Mods(self.0.clone())
    }
}

#[derive_where(PartialEq; Loc: Eq + Hash)]
#[derive_where(Default)]
#[derive(Debug)]
struct ModsInner<Loc> {
    actives: HashMap<Loc, TargetType>,
    all_targets: HashMap<Loc, TargetType>,
    dangers: HashSet<Loc>,
}

#[component]
pub fn ActiveHighlight<Loc: Clone + Hash + Eq + 'static>(at: Loc) -> Element {
    let ModsInner { actives, .. } = &*use_mods::<Loc>();
    let Mouse { x, y, .. } = use_context::<Signal<Mouse<Loc>>>()();
    if let Some(target_type) = actives.get(&at) {
        let sat = target_type.to_sat();
        let effect = if let TargetType::Pick = target_type {
            Effect::Flat
        } else {
            Effect::Fluent
        };
        rsx!(
            div { class: sat }
            Bordered {
                x, y, effect,
                color: "active",
            }
            Specular {
                x, y, effect,
                color: "active",
            }
        )
    } else {
        rsx!()
    }
}

#[component]
pub fn DangerHighlights<Loc: Eq + Hash + 'static>(at: Loc) -> Element {
    let ModsInner { dangers, .. } = &*use_mods::<Loc>();
    if dangers.contains(&at) {
        rsx!(div { class: "danger" })
    } else {
        rsx!()
    }
}

#[component]
pub fn TargetHighlight<Loc: Clone + Eq + Hash + 'static>(at: Loc) -> Element {
    let ModsInner { all_targets, .. } = &*use_mods::<Loc>();
    let Mouse { x, y, w, h, cell } = use_context::<Signal<Mouse<Loc>>>()();
    if let Some(target_type) = all_targets.get(&at) {
        let (size, t_x, t_y) = if *target_type == TargetType::Pick && cell == at {
            let dist = ((x - w / 2.0).powi(2) + (y - h / 2.0).powi(2)).sqrt();
            let t = (1.0 - dist / w * 2.0).max(0.0);
            let size = t * 5.0 + 10.0;
            (size, (x - w / 2.0) * 0.2 * t, (y - h / 2.0) * 0.2 * t)
        } else {
            (10.0, 0.0, 0.0)
        };
        let effect = if let TargetType::Pick = target_type {
            Effect::Flat
        } else {
            Effect::Fluent
        };
        let sat = target_type.to_sat();
        rsx!(
            div { class: sat }
            div {
                class: "target overlapper",
                style: "--x: {t_x}px; --y: {t_y}px; --size: {size}%",
                Specular {
                    x, y, effect,
                    color: "target",
                }
            }
        )
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

#[derive(Copy, Clone, PartialEq)]
enum Effect {
    Fluent,
    Flat,
}

impl Effect {
    fn class(self) -> &'static str {
        match self {
            Self::Fluent => "fluent",
            Self::Flat => "flat",
        }
    }
}
