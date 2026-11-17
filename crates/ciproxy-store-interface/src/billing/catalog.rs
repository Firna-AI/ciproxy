//! Published monthly prices and capacities for the shared billing values.

use super::state::{AddOns, Selection, Tier};

struct PublishedTier {
    monthly_cents: u64,
    starts: u64,
    machines: u64,
    history_days: u32,
}

impl Tier {
    /// The published monthly tier price in USD cents, without add-ons.
    pub fn monthly_cents(self) -> u64 {
        self.published().monthly_cents
    }

    /// The starts included in each period before add-ons.
    pub fn included_starts(self) -> u64 {
        self.published().starts
    }

    /// The registered machines included before add-ons.
    pub fn included_machines(self) -> u64 {
        self.published().machines
    }

    /// The number of finished-job history dates visible on this tier.
    pub fn history_days(self) -> u32 {
        self.published().history_days
    }

    fn published(self) -> PublishedTier {
        let (monthly_cents, starts, machines, history_days) = match self {
            Self::Free => (0, 2_000, 1, 7),
            Self::Team => (4_900, 25_000, 3, 30),
            Self::Business => (29_900, 150_000, 10, 90),
        };
        PublishedTier {
            monthly_cents,
            starts,
            machines,
            history_days,
        }
    }
}

impl AddOns {
    /// The starts supplied by one monthly start block.
    pub const STARTS_PER_BLOCK: u64 = 10_000;
    /// The monthly price of one start block in USD cents.
    pub const START_BLOCK_CENTS: u64 = 1_000;
    /// The monthly price of one additional machine in USD cents.
    pub const MACHINE_CENTS: u64 = 2_000;

    /// The monthly add-on total in USD cents.
    pub fn monthly_cents(self) -> u64 {
        u64::from(self.start_blocks) * Self::START_BLOCK_CENTS
            + u64::from(self.machines) * Self::MACHINE_CENTS
    }
}

impl Selection {
    /// The monthly tier and add-on total in USD cents, with Free at zero.
    pub fn monthly_cents(self) -> u64 {
        self.tier.monthly_cents() + self.paid_add_ons().monthly_cents()
    }

    /// The included starts, ignoring saved add-ons for Free.
    pub fn included_starts(self) -> u64 {
        self.tier.included_starts()
            + u64::from(self.paid_add_ons().start_blocks) * AddOns::STARTS_PER_BLOCK
    }

    /// The included machines, ignoring saved add-ons for Free.
    pub fn included_machines(self) -> u64 {
        self.tier.included_machines() + u64::from(self.paid_add_ons().machines)
    }

    fn paid_add_ons(self) -> AddOns {
        if self.tier == Tier::Free {
            AddOns::default()
        } else {
            self.add_ons
        }
    }
}
