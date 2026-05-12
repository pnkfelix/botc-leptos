#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Team {
    Townsfolk,
    Outsider,
    Minion,
    Demon,
}

impl Team {
    pub fn label(self) -> &'static str {
        match self {
            Team::Townsfolk => "Townsfolk",
            Team::Outsider => "Outsiders",
            Team::Minion => "Minions",
            Team::Demon => "Demon",
        }
    }

    pub fn css_class(self) -> &'static str {
        match self {
            Team::Townsfolk => "townsfolk",
            Team::Outsider => "outsider",
            Team::Minion => "minion",
            Team::Demon => "demon",
        }
    }
}

/// What "in play" implies about "in bag" for a given role at setup.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InPlayRouting {
    /// Role in play ⇒ role-token is in the bag (the common case).
    ImpliesInBag,
    /// Role in play ⇒ role-token is NOT in the bag (a different token sits in
    /// the bag instead). Trouble Brewing's Drunk is the canonical example.
    ImpliesNotInBag,
    /// Role in play is ambiguous about bag membership; the user must pick.
    /// Bad Moon Rising's Lunatic is the canonical example.
    CanBeInOrOutOfBag,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Role {
    Washerwoman,
    Librarian,
    Investigator,
    Chef,
    Empath,
    FortuneTeller,
    Undertaker,
    Monk,
    Ravenkeeper,
    Virgin,
    Slayer,
    Soldier,
    Mayor,
    Butler,
    Drunk,
    Recluse,
    Saint,
    Lunatic,
    Poisoner,
    Spy,
    Baron,
    ScarletWoman,
    Marionette,
    Imp,
}

impl Role {
    pub const ALL: &'static [Role] = &[
        Role::Washerwoman,
        Role::Librarian,
        Role::Investigator,
        Role::Chef,
        Role::Empath,
        Role::FortuneTeller,
        Role::Undertaker,
        Role::Monk,
        Role::Ravenkeeper,
        Role::Virgin,
        Role::Slayer,
        Role::Soldier,
        Role::Mayor,
        Role::Butler,
        Role::Drunk,
        Role::Recluse,
        Role::Saint,
        Role::Lunatic,
        Role::Poisoner,
        Role::Spy,
        Role::Baron,
        Role::ScarletWoman,
        Role::Marionette,
        Role::Imp,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Role::Washerwoman => "Washerwoman",
            Role::Librarian => "Librarian",
            Role::Investigator => "Investigator",
            Role::Chef => "Chef",
            Role::Empath => "Empath",
            Role::FortuneTeller => "Fortune Teller",
            Role::Undertaker => "Undertaker",
            Role::Monk => "Monk",
            Role::Ravenkeeper => "Ravenkeeper",
            Role::Virgin => "Virgin",
            Role::Slayer => "Slayer",
            Role::Soldier => "Soldier",
            Role::Mayor => "Mayor",
            Role::Butler => "Butler",
            Role::Drunk => "Drunk",
            Role::Recluse => "Recluse",
            Role::Saint => "Saint",
            Role::Lunatic => "Lunatic",
            Role::Poisoner => "Poisoner",
            Role::Spy => "Spy",
            Role::Baron => "Baron",
            Role::ScarletWoman => "Scarlet Woman",
            Role::Marionette => "Marionette",
            Role::Imp => "Imp",
        }
    }

    pub fn team(self) -> Team {
        use Role::*;
        match self {
            Washerwoman | Librarian | Investigator | Chef | Empath | FortuneTeller
            | Undertaker | Monk | Ravenkeeper | Virgin | Slayer | Soldier | Mayor => {
                Team::Townsfolk
            }
            Butler | Drunk | Recluse | Saint | Lunatic => Team::Outsider,
            Poisoner | Spy | Baron | ScarletWoman | Marionette => Team::Minion,
            Imp => Team::Demon,
        }
    }

    pub fn in_play_routing(self) -> InPlayRouting {
        match self {
            // In play, but a different token sits in the bag instead.
            Role::Drunk | Role::Marionette => InPlayRouting::ImpliesNotInBag,
            // Bag membership is ambiguous when this role is in play.
            Role::Lunatic => InPlayRouting::CanBeInOrOutOfBag,
            _ => InPlayRouting::ImpliesInBag,
        }
    }
}
