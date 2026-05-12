use leptos::prelude::*;
use web_sys::DragEvent;

use crate::constraints::{Constraints, Slot, TokenId};
use crate::roles::{InPlayRouting, Role, Team};

const TEAMS: &[Team] = &[
    Team::Townsfolk,
    Team::Outsider,
    Team::Minion,
    Team::Demon,
];

fn parse_token_id(ev: &DragEvent) -> Option<TokenId> {
    let dt = ev.data_transfer()?;
    let payload = dt.get_data("text/plain").ok()?;
    TokenId::parse(&payload)
}

#[component]
pub fn BagBuilder() -> impl IntoView {
    let constraints = RwSignal::new(Constraints::initial());

    let bag_rect_ref: NodeRef<leptos::html::Div> = NodeRef::new();

    let bag_dragging = RwSignal::new(false);
    let play_dragging = RwSignal::new(false);
    let notplay_dragging = RwSignal::new(false);
    let container_dragging = RwSignal::new(false);

    // Returns true if the drop's cursor lands within the "In Bag" rect's box.
    let cursor_in_bag_rect = move |ev: &DragEvent| -> bool {
        bag_rect_ref
            .get()
            .map(|el| {
                let r = el.get_bounding_client_rect();
                let x = ev.client_x() as f64;
                let y = ev.client_y() as f64;
                x >= r.left() && x <= r.right() && y >= r.top() && y <= r.bottom()
            })
            .unwrap_or(false)
    };

    // Generic drop handler for slots whose target is fixed.
    let drop_into_slot = move |target: Slot, ev: DragEvent| {
        ev.prevent_default();
        if let Some(id) = parse_token_id(&ev) {
            leptos::logging::log!("drop_into_slot id={} target={:?}", id.0, target);
            constraints.update(|c| c.place(id, target));
            constraints.with(|c| {
                leptos::logging::log!(
                    "after place: palette={} bag_only={} bag_not_play={} bag_and_play={} play_not_bag={} bluffs={} neither={}",
                    c.slot(Slot::Palette).len(),
                    c.slot(Slot::BagOnly).len(),
                    c.slot(Slot::BagNotPlay).len(),
                    c.slot(Slot::BagAndPlay).len(),
                    c.slot(Slot::PlayNotBag).len(),
                    c.slot(Slot::Bluffs).len(),
                    c.slot(Slot::Neither).len(),
                );
            });
        } else {
            leptos::logging::log!("drop_into_slot: no token id parsed, target was {:?}", target);
        }
    };

    // Play-rect drop: the role's InPlayRouting picks the slot, with the
    // ambiguous case (CanBeInOrOutOfBag) falling back to coord routing.
    let drop_on_play_rect = move |ev: DragEvent| {
        ev.prevent_default();
        let Some(id) = parse_token_id(&ev) else { return };
        let role = constraints.with(|c| c.token(id).map(|t| t.role()));
        let Some(role) = role else { return };
        let target = match role.in_play_routing() {
            InPlayRouting::ImpliesInBag => Slot::BagAndPlay,
            InPlayRouting::ImpliesNotInBag => Slot::PlayNotBag,
            InPlayRouting::CanBeInOrOutOfBag => {
                if cursor_in_bag_rect(&ev) {
                    Slot::BagAndPlay
                } else {
                    Slot::PlayNotBag
                }
            }
        };
        constraints.update(|c| c.place(id, target));
    };

    // Not-in-Play-rect drop: landing inside the "In Bag" rect means the role's
    // token is in the bag but the role isn't in play (a Drunk/Marionette-style
    // decoy); landing outside the bag means a Demon bluff.
    let drop_on_notplay_rect = move |ev: DragEvent| {
        ev.prevent_default();
        let Some(id) = parse_token_id(&ev) else { return };
        let target = if cursor_in_bag_rect(&ev) {
            Slot::BagNotPlay
        } else {
            Slot::Bluffs
        };
        constraints.update(|c| c.place(id, target));
    };

    view! {
        <div class="bag-builder">
            <h2>"Trouble Brewing — Bag Builder"</h2>
            <p class="hint">
                "Drag a role into the zone that captures the constraint you want. "
                "Where "<b>"In Bag"</b>" overlaps "<b>"In Play"</b>" is implicitly "<b>"Bag AND Play"</b>"; "
                "where "<b>"In Bag"</b>" overlaps "<b>"Not in Play"</b>" is the decoy-token zone "
                "("<b>"Bag, NOT in Play"</b>"). The part of "<b>"Not in Play"</b>" outside "<b>"In Bag"</b>" "
                "holds the "<b>"Demon's bluffs"</b>". Dropping on the empty container background = "<b>"Neither"</b>"."
            </p>

            <Palette constraints=constraints />

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
                    drop_into_slot(Slot::Neither, ev);
                }
            >
                // "In Bag": the wide rectangle spanning the top. Its clear
                // areas (top band, side margins, the gap between the two lower
                // rects) are the "bag, play unspecified" drop target.
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
                        drop_into_slot(Slot::BagOnly, ev);
                    }
                >
                    <span class="venn-rect-label">"In Bag"</span>
                    <ZoneChips
                        target=Slot::BagOnly
                        constraints=constraints
                        wrapper_class="zone-chips zone-chips-bag-only"
                        zone_label="bag (play unspec)"
                    />
                </div>

                // "Not in Play": lower-left rect, overlapping the left part of
                // "In Bag" (mirror of "In Play"). Top sub-zone (inside In Bag)
                // = "Bag, NOT in Play"; bottom sub-zone (outside) = Demon bluffs.
                <div
                    class=move || {
                        if notplay_dragging.get() {
                            "venn-rect venn-rect-notplay venn-rect-active"
                        } else {
                            "venn-rect venn-rect-notplay"
                        }
                    }
                    on:dragover=move |ev: DragEvent| {
                        ev.prevent_default();
                        ev.stop_propagation();
                        notplay_dragging.set(true);
                    }
                    on:dragleave=move |ev| {
                        ev.stop_propagation();
                        notplay_dragging.set(false);
                    }
                    on:drop=move |ev: DragEvent| {
                        ev.stop_propagation();
                        notplay_dragging.set(false);
                        drop_on_notplay_rect(ev);
                    }
                >
                    <span class="venn-rect-label">"Not in Play"</span>
                    <ZoneChips
                        target=Slot::BagNotPlay
                        constraints=constraints
                        wrapper_class="zone-chips zone-chips-inbag zone-chips-bag-not-play"
                        zone_label="bag, NOT in play"
                    />
                    <ZoneChips
                        target=Slot::Bluffs
                        constraints=constraints
                        wrapper_class="zone-chips zone-chips-bluffs"
                        zone_label="demon bluffs"
                    />
                </div>

                // "In Play": lower-right rect, overlapping the right part of
                // "In Bag". Top sub-zone (inside In Bag) = "Bag AND Play";
                // bottom sub-zone (outside) = "Play, NOT in Bag".
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
                        target=Slot::BagAndPlay
                        constraints=constraints
                        wrapper_class="zone-chips zone-chips-inbag zone-chips-bag-and-play"
                        zone_label="bag ∩ play"
                    />
                    <ZoneChips
                        target=Slot::PlayNotBag
                        constraints=constraints
                        wrapper_class="zone-chips zone-chips-play-not-bag"
                        zone_label="play, NOT in bag"
                    />
                </div>

                <div class="venn-neither-badge" aria-hidden="true">
                    {Slot::Neither.label()}
                </div>
                <NeitherList constraints=constraints />
            </section>
        </div>
    }
}

/// Sorted (TokenId, Role) snapshot of a slot's contents — used by the various
/// chip-list renders so display order is stable across moves.
fn slot_snapshot(constraints: RwSignal<Constraints>, slot: Slot) -> Vec<(TokenId, Role)> {
    constraints.with(|c| {
        let mut tokens: Vec<_> = c.slot(slot).iter().map(|t| (t.id(), t.role())).collect();
        tokens.sort_by_key(|(id, _)| *id);
        tokens
    })
}

#[component]
fn Palette(constraints: RwSignal<Constraints>) -> impl IntoView {
    view! {
        <section
            class="palette"
            on:dragover=move |ev: DragEvent| ev.prevent_default()
            on:drop=move |ev: DragEvent| {
                ev.prevent_default();
                if let Some(id) = parse_token_id(&ev) {
                    constraints.update(|c| c.place(id, Slot::Palette));
                }
            }
        >
            {TEAMS.iter().copied().map(|team| {
                view! {
                    <div class=format!("team-group team-{}", team.css_class())>
                        <h4>{team.label()}</h4>
                        <div class="role-list">
                            {move || {
                                let snap = slot_snapshot(constraints, Slot::Palette);
                                leptos::logging::log!("Palette render team={:?} total={}", team, snap.len());
                                snap.into_iter()
                                    .filter(|(_, r)| r.team() == team)
                                    .map(|(id, role)| view! {
                                        <PaletteChip id=id role=role />
                                    })
                                    .collect_view()
                            }}
                        </div>
                    </div>
                }
            }).collect_view()}
        </section>
    }
}

#[component]
fn PaletteChip(id: TokenId, role: Role) -> impl IntoView {
    view! {
        <div
            class=format!("role-chip role-chip-{}", role.team().css_class())
            draggable="true"
            on:dragstart=move |ev: DragEvent| {
                if let Some(dt) = ev.data_transfer() {
                    let _ = dt.set_data("text/plain", &id.0.to_string());
                    dt.set_effect_allowed("move");
                }
            }
        >
            <span class="role-name">{role.name()}</span>
        </div>
    }
}

#[component]
fn ZoneChips(
    target: Slot,
    constraints: RwSignal<Constraints>,
    wrapper_class: &'static str,
    zone_label: &'static str,
) -> impl IntoView {
    leptos::logging::log!("ZoneChips body executing target={:?} label={:?}", target, zone_label);
    view! {
        <div class=wrapper_class>
            <div class="zone-chips-label">{zone_label}</div>
            <div class="role-list">
                {move || {
                    let snap = slot_snapshot(constraints, target);
                    leptos::logging::log!("ZoneChips render target={:?} count={}", target, snap.len());
                    snap.into_iter()
                        .map(|(id, role)| view! {
                            <RemovableChip id=id role=role constraints=constraints />
                        })
                        .collect_view()
                }}
            </div>
        </div>
    }
}

#[component]
fn RemovableChip(
    id: TokenId,
    role: Role,
    constraints: RwSignal<Constraints>,
) -> impl IntoView {
    view! {
        <div
            class=format!("role-chip role-chip-{}", role.team().css_class())
            draggable="true"
            on:dragstart=move |ev: DragEvent| {
                if let Some(dt) = ev.data_transfer() {
                    let _ = dt.set_data("text/plain", &id.0.to_string());
                    dt.set_effect_allowed("move");
                }
            }
        >
            <span class="role-name">{role.name()}</span>
            <button
                class="remove"
                on:click=move |_| {
                    constraints.update(|c| c.place(id, Slot::Palette));
                }
                title="Return to palette"
            >"×"</button>
        </div>
    }
}

#[component]
fn NeitherList(constraints: RwSignal<Constraints>) -> impl IntoView {
    view! {
        <Show when=move || constraints.with(|c| !c.slot(Slot::Neither).is_empty())>
            <div class="venn-neither-list">
                <span class="venn-neither-list-label">"Neither:"</span>
                {move || slot_snapshot(constraints, Slot::Neither)
                    .into_iter()
                    .map(|(id, role)| view! {
                        <RemovableChip id=id role=role constraints=constraints />
                    })
                    .collect_view()}
            </div>
        </Show>
    }
}
