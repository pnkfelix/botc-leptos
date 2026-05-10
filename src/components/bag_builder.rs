use leptos::prelude::*;
use web_sys::DragEvent;

use crate::roles::{InPlayRouting, Role, Team};

const TEAMS: &[Team] = &[
    Team::Townsfolk,
    Team::Outsider,
    Team::Minion,
    Team::Demon,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Zone {
    BagNotPlay,
    Bag,
    BagAndPlay,
    PlayNotBag,
    NeitherBagNorPlay,
}

impl Zone {
    pub fn label(self) -> &'static str {
        match self {
            Zone::BagNotPlay => "In Bag, NOT in Play",
            Zone::Bag => "In Bag (play unspecified)",
            Zone::BagAndPlay => "In Bag AND in Play",
            Zone::PlayNotBag => "In Play, NOT in Bag",
            Zone::NeitherBagNorPlay => "NOT in Bag, NOT in Play",
        }
    }

    pub fn css_class(self) -> &'static str {
        match self {
            Zone::BagNotPlay => "bag-notplay",
            Zone::Bag => "bag-only",
            Zone::BagAndPlay => "bag-and-play",
            Zone::PlayNotBag => "play-notbag",
            Zone::NeitherBagNorPlay => "neither",
        }
    }
}

const DRAG_ZONE_PREFIX: &str = "zone:";
const DRAG_PALETTE: &str = "palette";

fn parse_role_from_event(ev: &DragEvent) -> Option<Role> {
    let dt = ev.data_transfer()?;
    let payload = dt.get_data("text/plain").ok()?;
    let role_id = payload
        .strip_prefix(DRAG_ZONE_PREFIX)
        .and_then(|rest| rest.split_once(':').map(|(_, id)| id))
        .or_else(|| payload.strip_prefix(&format!("{DRAG_PALETTE}:")));
    role_id.and_then(Role::from_id)
}

#[component]
pub fn BagBuilder() -> impl IntoView {
    // One signal per zone holding the roles currently assigned to that constraint.
    // A role is in at most one zone; absence from all zones = "unspecified" (palette).
    let zones: [(Zone, RwSignal<Vec<Role>>); 5] = [
        (Zone::BagNotPlay, RwSignal::new(Vec::new())),
        (Zone::Bag, RwSignal::new(Vec::new())),
        (Zone::BagAndPlay, RwSignal::new(Vec::new())),
        (Zone::PlayNotBag, RwSignal::new(Vec::new())),
        (Zone::NeitherBagNorPlay, RwSignal::new(Vec::new())),
    ];

    // Closure to find the signal for a zone.
    let signal_for = move |z: Zone| -> RwSignal<Vec<Role>> {
        zones.iter().find(|(zz, _)| *zz == z).map(|(_, s)| *s).unwrap()
    };

    // A role placed somewhere has its previous home cleared first.
    let remove_everywhere = move |role: Role| {
        for (_, sig) in zones.iter() {
            sig.update(|v| v.retain(|r| *r != role));
        }
    };

    // Compute the set of roles currently assigned to *any* zone — used by the palette
    // to dim/hide already-placed roles.
    let placed = Memo::new(move |_| {
        let mut set = Vec::new();
        for (_, sig) in zones.iter() {
            for r in sig.get() {
                set.push(r);
            }
        }
        set
    });

    let drop_into_zone = move |target: Zone, ev: DragEvent| {
        ev.prevent_default();
        let Some(dt) = ev.data_transfer() else { return };
        let Ok(payload) = dt.get_data("text/plain") else { return };
        let role_id = payload
            .strip_prefix(DRAG_ZONE_PREFIX)
            .and_then(|rest| rest.split_once(':').map(|(_, id)| id))
            .or_else(|| payload.strip_prefix(&format!("{DRAG_PALETTE}:")));
        let Some(role_id) = role_id else { return };
        let Some(role) = Role::from_id(role_id) else { return };
        remove_everywhere(role);
        signal_for(target).update(|v| v.push(role));
    };

    // Dropping back onto the palette = clear the role's constraint.
    let drop_into_palette = move |ev: DragEvent| {
        ev.prevent_default();
        let Some(dt) = ev.data_transfer() else { return };
        let Ok(payload) = dt.get_data("text/plain") else { return };
        let Some(role_id) = payload
            .strip_prefix(DRAG_ZONE_PREFIX)
            .and_then(|rest| rest.split_once(':').map(|(_, id)| id))
        else {
            return;
        };
        let Some(role) = Role::from_id(role_id) else { return };
        remove_everywhere(role);
    };

    let bag_rect_ref: NodeRef<leptos::html::Div> = NodeRef::new();

    // Drop on the play rectangle: the role's own InPlayRouting decides for us.
    // - ImpliesInBag      → BagAndPlay
    // - ImpliesNotInBag   → PlayNotBag
    // - CanBeInOrOutOfBag → coord-based: if the cursor is also inside the bag
    //   rectangle (i.e. in the visual overlap) → BagAndPlay, else → PlayNotBag.
    //   The user disambiguates by where they drop.
    let drop_on_play_rect = move |ev: DragEvent| {
        ev.prevent_default();
        let Some(role) = parse_role_from_event(&ev) else { return };
        let zone = match role.in_play_routing() {
            InPlayRouting::ImpliesInBag => Zone::BagAndPlay,
            InPlayRouting::ImpliesNotInBag => Zone::PlayNotBag,
            InPlayRouting::CanBeInOrOutOfBag => {
                let cursor_in_bag = bag_rect_ref
                    .get()
                    .map(|el| {
                        let r = el.get_bounding_client_rect();
                        let x = ev.client_x() as f64;
                        let y = ev.client_y() as f64;
                        x >= r.left() && x <= r.right() && y >= r.top() && y <= r.bottom()
                    })
                    .unwrap_or(false);
                if cursor_in_bag {
                    Zone::BagAndPlay
                } else {
                    Zone::PlayNotBag
                }
            }
        };
        remove_everywhere(role);
        signal_for(zone).update(|v| v.push(role));
    };

    let bag_dragging = RwSignal::new(false);
    let play_dragging = RwSignal::new(false);
    let container_dragging = RwSignal::new(false);
    let neither_sig = signal_for(Zone::NeitherBagNorPlay);

    view! {
        <div class="bag-builder">
            <h2>"Trouble Brewing — Bag Builder"</h2>
            <p class="hint">
                "Drag a role into the zone that captures the constraint you want. "
                "The intersection of the two rectangles is implicitly "<b>"Bag AND Play"</b>". "
                "Dropping on the empty container background = "<b>"Neither"</b>"."
            </p>

            <Palette placed=placed.into() on_drop_back=drop_into_palette />

            <section
                class=move || {
                    if container_dragging.get() {
                        "venn-container venn-container-active"
                    } else {
                        "venn-container"
                    }
                }
                on:dragover=move |ev: DragEvent| {
                    ev.prevent_default();
                    container_dragging.set(true);
                }
                on:dragleave=move |_| container_dragging.set(false)
                on:drop=move |ev: DragEvent| {
                    container_dragging.set(false);
                    drop_into_zone(Zone::NeitherBagNorPlay, ev);
                }
            >
                // In Bag rectangle: drop target. Drops here that aren't caught
                // by the inner not-play sub-rect (higher z) or the play
                // rectangle (higher z in the overlap region) land here as
                // "Bag (play unspec)".
                <div
                    node_ref=bag_rect_ref
                    class=move || {
                        if bag_dragging.get() {
                            "venn-rect venn-rect-bag venn-rect-active"
                        } else {
                            "venn-rect venn-rect-bag"
                        }
                    }
                    on:dragover=move |ev: DragEvent| {
                        ev.prevent_default();
                        ev.stop_propagation();
                        bag_dragging.set(true);
                    }
                    on:dragleave=move |ev| {
                        ev.stop_propagation();
                        bag_dragging.set(false);
                    }
                    on:drop=move |ev: DragEvent| {
                        ev.stop_propagation();
                        bag_dragging.set(false);
                        drop_into_zone(Zone::Bag, ev);
                    }
                >
                    <span class="venn-rect-label">"In Bag"</span>
                    <ZoneChips
                        zone=Zone::Bag
                        contents=signal_for(Zone::Bag).into()
                        on_remove=move |role| remove_everywhere(role)
                        wrapper_class="zone-chips zone-chips-bag-only"
                        zone_label="bag (play unspec)"
                    />
                </div>

                // In Play rectangle: drop target with coord-based routing.
                <div
                    class=move || {
                        if play_dragging.get() {
                            "venn-rect venn-rect-play venn-rect-active"
                        } else {
                            "venn-rect venn-rect-play"
                        }
                    }
                    on:dragover=move |ev: DragEvent| {
                        ev.prevent_default();
                        ev.stop_propagation();
                        play_dragging.set(true);
                    }
                    on:dragleave=move |ev| {
                        ev.stop_propagation();
                        play_dragging.set(false);
                    }
                    on:drop=move |ev: DragEvent| {
                        ev.stop_propagation();
                        play_dragging.set(false);
                        drop_on_play_rect(ev);
                    }
                >
                    <span class="venn-rect-label">"In Play"</span>
                    <ZoneChips
                        zone=Zone::BagAndPlay
                        contents=signal_for(Zone::BagAndPlay).into()
                        on_remove=move |role| remove_everywhere(role)
                        wrapper_class="zone-chips zone-chips-bag-and-play"
                        zone_label="bag ∩ play"
                    />
                    <ZoneChips
                        zone=Zone::PlayNotBag
                        contents=signal_for(Zone::PlayNotBag).into()
                        on_remove=move |role| remove_everywhere(role)
                        wrapper_class="zone-chips zone-chips-play-not-bag"
                        zone_label="play, not bag"
                    />
                </div>

                // Inner not-play sub-rect: nested drop target inside bag area.
                <DropZone
                    zone=Zone::BagNotPlay
                    contents=signal_for(Zone::BagNotPlay).into()
                    on_drop=move |ev| drop_into_zone(Zone::BagNotPlay, ev)
                    on_remove=move |role| remove_everywhere(role)
                />

                <div class="venn-neither-badge" aria-hidden="true">
                    {Zone::NeitherBagNorPlay.label()}
                </div>
                <NeitherList
                    contents=neither_sig.into()
                    on_remove=move |role| remove_everywhere(role)
                />
            </section>
        </div>
    }
}

#[component]
fn ZoneChips<R>(
    zone: Zone,
    contents: Signal<Vec<Role>>,
    on_remove: R,
    wrapper_class: &'static str,
    zone_label: &'static str,
) -> impl IntoView
where
    R: Fn(Role) + 'static + Copy + Send + Sync,
{
    view! {
        <div class=wrapper_class>
            <div class="zone-chips-label">{zone_label}</div>
            <div class="role-list">
                {move || contents.get().into_iter().map(|role| {
                    let id = role.id();
                    let zc = zone.css_class();
                    view! {
                        <div
                            class=format!("role-chip role-chip-{}", role.team().css_class())
                            draggable="true"
                            on:dragstart=move |ev: DragEvent| {
                                if let Some(dt) = ev.data_transfer() {
                                    let _ = dt.set_data(
                                        "text/plain",
                                        &format!("{DRAG_ZONE_PREFIX}{}:{id}", zc),
                                    );
                                    dt.set_effect_allowed("move");
                                }
                            }
                        >
                            <span class="role-name">{role.name()}</span>
                            <button
                                class="remove"
                                on:click=move |_| on_remove(role)
                                title="Remove constraint"
                            >"×"</button>
                        </div>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}

#[component]
fn NeitherList<R>(contents: Signal<Vec<Role>>, on_remove: R) -> impl IntoView
where
    R: Fn(Role) + 'static + Copy + Send + Sync,
{
    view! {
        <Show when=move || !contents.get().is_empty()>
            <div class="venn-neither-list">
                <span class="venn-neither-list-label">"Neither:"</span>
                {move || contents.get().into_iter().map(|role| {
                    let id = role.id();
                    view! {
                        <div
                            class=format!("role-chip role-chip-{}", role.team().css_class())
                            draggable="true"
                            on:dragstart=move |ev: DragEvent| {
                                if let Some(dt) = ev.data_transfer() {
                                    let _ = dt.set_data(
                                        "text/plain",
                                        &format!("{DRAG_ZONE_PREFIX}neither:{id}"),
                                    );
                                    dt.set_effect_allowed("move");
                                }
                            }
                        >
                            <span class="role-name">{role.name()}</span>
                            <button
                                class="remove"
                                on:click=move |_| on_remove(role)
                                title="Remove constraint"
                            >"×"</button>
                        </div>
                    }
                }).collect_view()}
            </div>
        </Show>
    }
}

#[component]
fn Palette<F>(placed: Signal<Vec<Role>>, on_drop_back: F) -> impl IntoView
where
    F: Fn(DragEvent) + 'static + Copy + Send + Sync,
{
    view! {
        <section
            class="palette"
            on:dragover=move |ev: DragEvent| ev.prevent_default()
            on:drop=move |ev: DragEvent| on_drop_back(ev)
        >
            {TEAMS.iter().copied().map(|team| {
                view! {
                    <div class=format!("team-group team-{}", team.css_class())>
                        <h4>{team.label()}</h4>
                        <div class="role-list">
                            {Role::ALL.iter().copied()
                                .filter(move |r| r.team() == team)
                                .map(|role| view! {
                                    <PaletteChip role=role placed=placed />
                                })
                                .collect_view()}
                        </div>
                    </div>
                }
            }).collect_view()}
        </section>
    }
}

#[component]
fn PaletteChip(role: Role, placed: Signal<Vec<Role>>) -> impl IntoView {
    let id = role.id();
    let dim = move || placed.get().contains(&role);
    view! {
        <div
            class=move || {
                let mut c = format!("role-chip role-chip-{}", role.team().css_class());
                if dim() {
                    c.push_str(" placed");
                }
                c
            }
            draggable="true"
            on:dragstart=move |ev: DragEvent| {
                if let Some(dt) = ev.data_transfer() {
                    let _ = dt.set_data("text/plain", &format!("{DRAG_PALETTE}:{id}"));
                    dt.set_effect_allowed("move");
                }
            }
        >
            <span class="role-name">{role.name()}</span>
        </div>
    }
}

#[component]
fn DropZone<D, R>(
    zone: Zone,
    contents: Signal<Vec<Role>>,
    on_drop: D,
    on_remove: R,
) -> impl IntoView
where
    D: Fn(DragEvent) + 'static + Copy + Send + Sync,
    R: Fn(Role) + 'static + Copy + Send + Sync,
{
    let dragging_over = RwSignal::new(false);

    view! {
        <div
            class=move || {
                let base = format!("drop-zone drop-zone-{}", zone.css_class());
                if dragging_over.get() {
                    format!("{base} drop-zone-active")
                } else {
                    base
                }
            }
            on:dragover=move |ev: DragEvent| {
                ev.prevent_default();
                ev.stop_propagation();
                dragging_over.set(true);
            }
            on:dragleave=move |ev| {
                ev.stop_propagation();
                dragging_over.set(false);
            }
            on:drop=move |ev: DragEvent| {
                ev.stop_propagation();
                dragging_over.set(false);
                on_drop(ev);
            }
        >
            <h3>{zone.label()}</h3>
            <div class="role-list">
                {move || contents.get().into_iter().map(|role| {
                    let id = role.id();
                    view! {
                        <div
                            class=format!("role-chip role-chip-{}", role.team().css_class())
                            draggable="true"
                            on:dragstart=move |ev: DragEvent| {
                                if let Some(dt) = ev.data_transfer() {
                                    let _ = dt.set_data(
                                        "text/plain",
                                        &format!("{DRAG_ZONE_PREFIX}{}:{id}", zone.css_class()),
                                    );
                                    dt.set_effect_allowed("move");
                                }
                            }
                        >
                            <span class="role-name">{role.name()}</span>
                            <button
                                class="remove"
                                on:click=move |_| on_remove(role)
                                title="Remove constraint"
                            >"×"</button>
                        </div>
                    }
                }).collect_view()}
            </div>
            <Show when=move || contents.get().is_empty()>
                <div class="empty-hint">"drop role here"</div>
            </Show>
        </div>
    }
}
