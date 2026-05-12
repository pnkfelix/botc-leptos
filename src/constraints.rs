use crate::roles::Role;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TokenId(pub u32);

impl TokenId {
    pub fn parse(s: &str) -> Option<TokenId> {
        s.parse::<u32>().ok().map(TokenId)
    }
}

/// A role token that exists in exactly one slot at a time. Deliberately not
/// `Clone` and not `Copy` so the borrow checker rules out duplication: moving
/// from one slot to another goes through `&mut Constraints`, which forces the
/// caller to take the token by value.
#[derive(Debug, PartialEq, Eq)]
pub struct RoleToken {
    id: TokenId,
    role: Role,
}

impl RoleToken {
    pub fn id(&self) -> TokenId {
        self.id
    }
    pub fn role(&self) -> Role {
        self.role
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Slot {
    /// Constraint is unspecified — token is available for placement.
    Palette,
    /// In bag, explicitly NOT in play — e.g. the decoy token a Drunk or
    /// Marionette player draws: that role's token is in the bag, but the
    /// role itself isn't truly in play. Lives in the In-Bag ∩ Not-in-Play
    /// overlap.
    BagNotPlay,
    /// In bag, play status unspecified.
    BagOnly,
    /// In bag AND in play.
    BagAndPlay,
    /// In play, explicitly NOT in bag.
    PlayNotBag,
    /// NOT in bag, NOT in play, and shown to the Demon as one of its bluffs.
    /// Lives in the part of Not-in-Play that is outside In-Bag.
    Bluffs,
    /// Explicitly not in bag and not in play.
    Neither,
}

impl Slot {
    pub const ALL: &'static [Slot] = &[
        Slot::Palette,
        Slot::BagNotPlay,
        Slot::BagOnly,
        Slot::BagAndPlay,
        Slot::PlayNotBag,
        Slot::Bluffs,
        Slot::Neither,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Slot::Palette => "Palette",
            Slot::BagNotPlay => "In Bag, NOT in Play",
            Slot::BagOnly => "In Bag (play unspecified)",
            Slot::BagAndPlay => "In Bag AND in Play",
            Slot::PlayNotBag => "In Play, NOT in Bag",
            Slot::Bluffs => "Demon Bluff (not in bag, not in play)",
            Slot::Neither => "NOT in Bag, NOT in Play",
        }
    }
}

/// Owns every role token. There is exactly one `RoleToken` per role across
/// all slots; placement moves the token by value.
#[derive(Debug)]
pub struct Constraints {
    palette: Vec<RoleToken>,
    bag_not_play: Vec<RoleToken>,
    bag_only: Vec<RoleToken>,
    bag_and_play: Vec<RoleToken>,
    play_not_bag: Vec<RoleToken>,
    bluffs: Vec<RoleToken>,
    neither: Vec<RoleToken>,
}

impl Constraints {
    pub fn initial() -> Self {
        let palette = Role::ALL
            .iter()
            .copied()
            .enumerate()
            .map(|(i, role)| RoleToken {
                id: TokenId(i as u32),
                role,
            })
            .collect();
        Self {
            palette,
            bag_not_play: Vec::new(),
            bag_only: Vec::new(),
            bag_and_play: Vec::new(),
            play_not_bag: Vec::new(),
            bluffs: Vec::new(),
            neither: Vec::new(),
        }
    }

    pub fn slot(&self, slot: Slot) -> &[RoleToken] {
        match slot {
            Slot::Palette => &self.palette,
            Slot::BagNotPlay => &self.bag_not_play,
            Slot::BagOnly => &self.bag_only,
            Slot::BagAndPlay => &self.bag_and_play,
            Slot::PlayNotBag => &self.play_not_bag,
            Slot::Bluffs => &self.bluffs,
            Slot::Neither => &self.neither,
        }
    }

    pub fn token(&self, id: TokenId) -> Option<&RoleToken> {
        Slot::ALL
            .iter()
            .copied()
            .flat_map(|s| self.slot(s).iter())
            .find(|t| t.id() == id)
    }

    /// Move the token with `id` to `target`. No-op if the id isn't found.
    pub fn place(&mut self, id: TokenId, target: Slot) {
        if let Some(tok) = self.take(id) {
            self.slot_mut(target).push(tok);
        }
    }

    fn slot_mut(&mut self, slot: Slot) -> &mut Vec<RoleToken> {
        match slot {
            Slot::Palette => &mut self.palette,
            Slot::BagNotPlay => &mut self.bag_not_play,
            Slot::BagOnly => &mut self.bag_only,
            Slot::BagAndPlay => &mut self.bag_and_play,
            Slot::PlayNotBag => &mut self.play_not_bag,
            Slot::Bluffs => &mut self.bluffs,
            Slot::Neither => &mut self.neither,
        }
    }

    fn take(&mut self, id: TokenId) -> Option<RoleToken> {
        for slot in Slot::ALL.iter().copied() {
            let v = self.slot_mut(slot);
            if let Some(idx) = v.iter().position(|t| t.id() == id) {
                return Some(v.remove(idx));
            }
        }
        None
    }
}
