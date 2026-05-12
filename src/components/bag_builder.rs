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

/// Which slot a role lands in when dropped on one of the two "in play" zones.
/// `prefer_in_bag` is true for the "in bag & in play" zone, false for "in
/// play, not in bag"; a role whose `InPlayRouting` settles the matter wins
/// over the preference (e.g. the Drunk is always `PlayNotBag`).
fn in_play_slot(role: Role, prefer_in_bag: bool) -> Slot {
    match role.in_play_routing() {
        InPlayRouting::ImpliesInBag => Slot::BagAndPlay,
        InPlayRouting::ImpliesNotInBag => Slot::PlayNotBag,
        InPlayRouting::CanBeInOrOutOfBag => {
            if prefer_in_bag {
                Slot::BagAndPlay
            } else {
                Slot::PlayNotBag
            }
        }
    }
}

// Per-zone drop resolvers (named `fn` items so they coerce cleanly to the
// `fn(Role) -> Slot` prop type).
fn resolve_bag_only(_: Role) -> Slot { Slot::BagOnly }
fn resolve_bag_not_play(_: Role) -> Slot { Slot::BagNotPlay }
fn resolve_bluffs(_: Role) -> Slot { Slot::Bluffs }
fn resolve_neither(_: Role) -> Slot { Slot::Neither }
fn resolve_bag_and_play(role: Role) -> Slot { in_play_slot(role, true) }
fn resolve_play_not_bag(role: Role) -> Slot { in_play_slot(role, false) }

#[component]
pub fn BagBuilder() -> impl IntoView {
    let constraints = RwSignal::new(Constraints::initial());

    view! {
        <div class="bag-builder">
            <h2>"Trouble Brewing — Bag Builder"</h2>
            <p class="hint">
                "Drag a role into the zone for the constraint you want. The wide "
                <b>"In Bag"</b>" rectangle holds the three in-bag zones; "<b>"Not in Play"</b>
                " and "<b>"In Play"</b>" hang off its bottom — each one's upper zone overlaps "
                <b>"In Bag"</b>" (an intersection), its lower zone hangs outside. The "
                <b>"neither"</b>" strip at the bottom is \"not in bag and not in play\"; "
                "dropping outside any zone also lands there."
            </p>

            <Palette constraints=constraints />

            <section
                class="venn-container"
                on:dragover=move |ev: DragEvent| ev.prevent_default()
                on:drop=move |ev: DragEvent| {
                    ev.prevent_default();
                    if let Some(id) = parse_token_id(&ev) {
                        constraints.update(|c| c.place(id, Slot::Neither));
                    }
                }
            >
                // Decorative rectangles: grid items that span rows, painted
                // behind the zones and non-interactive (CSS pointer-events).
                <div class="venn-rect venn-rect-bag"></div>
                <div class="venn-rect venn-rect-notplay"></div>
                <div class="venn-rect venn-rect-play"></div>

                // The big rectangle labels, painted above the zones.
                <span class="venn-rect-label venn-rect-label-bag">"In Bag"</span>
                <span class="venn-rect-label venn-rect-label-notplay">"Not in Play"</span>
                <span class="venn-rect-label venn-rect-label-play">"In Play"</span>

                <ZoneChips
                    wrapper_class="zone-bag-only"
                    caption="in bag — play unspecified"
                    show=Slot::BagOnly
                    resolve=resolve_bag_only
                    constraints=constraints
                />
                <ZoneChips
                    wrapper_class="zone-inbag zone-bag-not-play"
                    caption="in bag & not in play"
                    show=Slot::BagNotPlay
                    resolve=resolve_bag_not_play
                    constraints=constraints
                />
                <ZoneChips
                    wrapper_class="zone-inbag zone-bag-and-play"
                    caption="in bag & in play"
                    show=Slot::BagAndPlay
                    resolve=resolve_bag_and_play
                    constraints=constraints
                />
                <ZoneChips
                    wrapper_class="zone-bluffs"
                    caption="demon bluffs — not in bag, not in play"
                    show=Slot::Bluffs
                    resolve=resolve_bluffs
                    constraints=constraints
                />
                <ZoneChips
                    wrapper_class="zone-play-not-bag"
                    caption="in play, not in bag"
                    show=Slot::PlayNotBag
                    resolve=resolve_play_not_bag
                    constraints=constraints
                />
                <ZoneChips
                    wrapper_class="zone-neither"
                    caption="neither — not in bag, not in play"
                    show=Slot::Neither
                    resolve=resolve_neither
                    constraints=constraints
                />
            </section>
        </div>
    }
}

/// Sorted (TokenId, Role) snapshot of a slot's contents so display order is
/// stable across moves.
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
                                slot_snapshot(constraints, Slot::Palette).into_iter()
                                    .filter(|(_, r)| r.team() == team)
                                    .map(|(id, role)| view! { <PaletteChip id=id role=role /> })
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
    /// Extra class(es) on the grid-cell wrapper (placement + tint).
    wrapper_class: &'static str,
    /// Caption shown above the chips.
    caption: &'static str,
    /// Slot whose contents are displayed here.
    show: Slot,
    /// Slot a dropped role resolves to — usually `show`, but the in-play zones
    /// remap per the role's `InPlayRouting` (so the Drunk can't sit "in bag").
    resolve: fn(Role) -> Slot,
    constraints: RwSignal<Constraints>,
) -> impl IntoView {
    let dragging = RwSignal::new(false);
    view! {
        <div
            class=move || {
                if dragging.get() {
                    format!("zone {wrapper_class} zone-active")
                } else {
                    format!("zone {wrapper_class}")
                }
            }
            on:dragover=move |ev: DragEvent| {
                ev.prevent_default();
                ev.stop_propagation();
                dragging.set(true);
            }
            on:dragleave=move |ev: DragEvent| {
                ev.stop_propagation();
                dragging.set(false);
            }
            on:drop=move |ev: DragEvent| {
                ev.prevent_default();
                ev.stop_propagation();
                dragging.set(false);
                let Some(id) = parse_token_id(&ev) else { return };
                let role = constraints.with(|c| c.token(id).map(|t| t.role()));
                if let Some(role) = role {
                    constraints.update(|c| c.place(id, resolve(role)));
                }
            }
        >
            <div class="zone-caption">{caption}</div>
            <div class="role-list">
                {move || slot_snapshot(constraints, show).into_iter()
                    .map(|(id, role)| view! {
                        <RemovableChip id=id role=role constraints=constraints />
                    })
                    .collect_view()}
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
