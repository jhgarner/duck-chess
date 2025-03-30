mod bordered;
mod specular;

use bordered::*;
use specular::*;

use super::*;

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Mods(Rc<ModsInner>);

impl Mods {
    pub fn new<Loc: Into<Block>>(targetting: Vec<Targetting<Loc>>, dangers: HashSet<Loc>) -> Self {
        let mut actives = HashMap::new();
        let mut all_targets = HashMap::new();
        let dangers = dangers.into_iter().map(Loc::into).collect();
        for Targetting {
            targets,
            active,
            target_type,
        } in targetting
        {
            if let Active::Active(loc) = active {
                actives.insert(loc.into(), target_type);
            }
            all_targets.extend(targets.into_iter().map(|l| (l.into(), target_type)));
        }
        Mods(Rc::new(ModsInner {
            actives,
            all_targets,
            dangers,
        }))
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
struct ModsInner {
    actives: HashMap<Block, TargetType>,
    all_targets: HashMap<Block, TargetType>,
    dangers: HashSet<Block>,
}

#[component]
pub fn ActiveHighlight(block: Block) -> Element {
    let ModsInner { actives, .. } = &*use_mods();
    let Mouse { x, y, .. } = use_context::<Signal<Mouse>>()();
    if let Some(target_type) = actives.get(&block) {
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
pub fn DangerHighlights(block: Block) -> Element {
    let ModsInner { dangers, .. } = &*use_mods();
    if dangers.contains(&block) {
        rsx!(div { class: "danger" })
    } else {
        rsx!()
    }
}

#[component]
pub fn TargetHighlight(block: Block) -> Element {
    let ModsInner { all_targets, .. } = &*use_mods();
    let Mouse { x, y, w, h, cell } = use_context::<Signal<Mouse>>()();
    if let Some(target_type) = all_targets.get(&block) {
        let (size, t_x, t_y) = if *target_type == TargetType::Pick && cell == block {
            let dist = ((x - w / 2.0).powi(2) + (y - h / 2.0).powi(2)).sqrt();
            let t = (1.0 - dist / w * 2.0).max(0.0);
            let size = t * 5.0 + 30.0;
            (size, (x - w / 2.0) * 0.2 * t, (y - h / 2.0) * 0.2 * t)
        } else {
            (30.0, 0.0, 0.0)
        };
        let effect = if let TargetType::Pick = target_type {
            Effect::Flat
        } else {
            Effect::Fluent
        };
        // The specular effect is inside a smaller square inscribed in the larger rectangle, so the
        // x and y positions have to be offset.
        let b = w.min(h);
        let w_offset = (w - b) / 2.0;
        let h_offset = (h - b) / 2.0;
        let x = x - w_offset;
        let y = y - h_offset;
        let sat = target_type.to_sat();
        rsx!(
            div { class: sat }
            div {
                // class: "",
                // div {
                class: "target",
                style: "--x: {t_x}px; --y: {t_y}px; --size: {size}%",
                Specular {
                    x, y, effect,
                    color: "target",
                }
                // }
            }
        )
    } else {
        rsx!()
    }
}

pub fn provide_mods(mods: Mods) {
    let signal = with_signal(mods.clone());
    provide_context(signal);
}

fn use_mods() -> Rc<ModsInner> {
    use_context::<Signal<Mods>>()().0
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
