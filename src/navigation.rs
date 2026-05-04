//! Navigation state machine for controller/keyboard UI navigation.
//!
//! Consolidates all focus state into a single struct with pure transition methods.
//! The home layout is (is used to show last opened games and pinned menu items, is a template for other tabs):
//!   ┌──────────────────────────────────────────┐
//!   │ Tab0  Tab1  Tab2  Tab3  ...  Tab7        │  ← FocusSection::Tabs
//!   ├──────────────────────────────────────────┤
//!   │ Left[0]   │                              │
//!   │ Left[1]   │   Center (HeroTile)          │  ← FocusSection::Hero
//!   │ Left[2]   │                              │
//!   ├──────────────────────────────────────────┤
//!   │ Buttons icons and what they do           │  ← (A) Select
//!   └──────────────────────────────────────────┘




//! The game library layout is:
//!   ┌──────────────────────────────────────────┐
//!   │ Sort by                          page    │  ← FocusSection::Tabs
//!   ├──────────────────────────────────────────┤
//!   │                                          │
//!   │ GameCarousel[0..N]                       │  ← FocusSection::GamesCarousel
//!   │                                          │
//!   ├──────────────────────────────────────────┤
//!   │ Buttons icons and what they do           │  ← (A) Select, (B) Back, (X) More Options, (Y) Search
//!   └──────────────────────────────────────────┘




use crate::input::NavAction;

pub const TAB_COUNT: usize = 8;
pub const MAX_LEFT_COL: usize = 3; // Open Tray, My Pins, Recent (menu tiles)

/// Which top-level section has input focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusSection {
    Tabs,
    Hero,
    GamesCarousel,
}

/// Position within the hero content area of the home tab.
/// LeftColumn(0..2) = menu tiles (Open Tray, My Pins, Recent).
/// Center = hero banner tile (index 3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeroPos {
    LeftColumn(usize),
    Center,
}

/// Consolidated focus state. All navigation "where is the cursor" state lives here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FocusState {
    pub section: FocusSection,
    pub tab: usize,
    pub hero: HeroPos,
    pub carousel: usize,
}

/// Effects that can result from a navigation action beyond a simple state transition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NavEffect {
    /// Launch the game/carousel item at the given index.
    LaunchGame(usize),
    /// Activate a tile action by index (menu item).
    ActivateTile(usize),
    /// Switch to a new tab (when tab is changed via direction nav).
    SwitchTab(usize),
    /// Just a state change — no side effect needed.
    None,
}

// ── Helpers ──────────────────────────────────────────────────────────

fn clamp_tab(idx: isize) -> usize {
    idx.clamp(0, (TAB_COUNT - 1) as isize) as usize
}

// ── Transitions ──────────────────────────────────────────────────────

impl FocusState {
    /// Start at the default home position.
    pub fn home() -> Self {
        Self {
            section: FocusSection::Hero,
            tab: 1, // "home"
            hero: HeroPos::LeftColumn(0),
            carousel: 0,
        }
    }

    /// Process a raw `NavAction` and return the effect to perform.
    /// Only calls `cx.notify()` when `true` is OR'd with the effect check.
    pub fn handle(&mut self, action: NavAction) -> NavEffect {
        match action {
            NavAction::NavigateUp => self.up(),
            NavAction::NavigateDown => self.down(),
            NavAction::NavigateLeft => self.left(),
            NavAction::NavigateRight => self.right(),
            NavAction::PreviousTab => self.prev_tab(),
            NavAction::NextTab => self.next_tab(),
            NavAction::Select => self.select(),
            NavAction::Back => self.back(),
            NavAction::Menu => NavEffect::None,
        }
    }

    // ── Directional helpers ──────────────────────────────────────────

    fn up(&mut self) -> NavEffect {
        match self.section {
            FocusSection::Tabs => {
                // Tabs → Hero (top section)
                self.section = FocusSection::Hero;
                self.hero = HeroPos::LeftColumn(0);
            }
            FocusSection::Hero => {
                // Hero → Tabs
                self.section = FocusSection::Tabs;
            }
            FocusSection::GamesCarousel => {
                // Carousel → Hero
                self.section = FocusSection::Hero;
            }
        }
        NavEffect::None
    }

    fn down(&mut self) -> NavEffect {
        match self.section {
            FocusSection::Tabs => {
                // Tabs → Hero
                self.section = FocusSection::Hero;
                self.hero = HeroPos::LeftColumn(0);
            }
            FocusSection::Hero => {
                // Hero → GamesCarousel (guarded by app layer using active_tab check)
                self.section = FocusSection::GamesCarousel;
            }
            FocusSection::GamesCarousel => {
                // Carousel → Hero
                self.section = FocusSection::Hero;
            }
        }
        NavEffect::None
    }

    fn left(&mut self) -> NavEffect {
        match self.section {
            FocusSection::Tabs => {
                if self.tab > 0 {
                    self.tab -= 1;
                }
            }
            FocusSection::Hero => match self.hero {
                HeroPos::LeftColumn(idx) => {
                    if idx > 0 {
                        self.hero = HeroPos::LeftColumn(idx - 1);
                    } else if self.tab > 0 {
                        // At top of left column → previous tab
                        self.section = FocusSection::Tabs;
                        self.tab -= 1;
                        return NavEffect::SwitchTab(self.tab);
                    }
                }
                HeroPos::Center => {
                    // Center → last menu tile
                    self.hero = HeroPos::LeftColumn(MAX_LEFT_COL - 1);
                }
            },
            FocusSection::GamesCarousel => {
                if self.carousel > 0 {
                    self.carousel -= 1;
                }
            }
        }
        NavEffect::None
    }

    fn right(&mut self) -> NavEffect {
        match self.section {
            FocusSection::Tabs => {
                if self.tab < TAB_COUNT - 1 {
                    self.tab += 1;
                }
            }
            FocusSection::Hero => match self.hero {
                HeroPos::LeftColumn(idx) => {
                    if idx < MAX_LEFT_COL - 1 {
                        self.hero = HeroPos::LeftColumn(idx + 1);
                    } else {
                        // Last menu tile → hero banner
                        self.hero = HeroPos::Center;
                    }
                }
                HeroPos::Center => {
                    // Hero banner → next tab
                    if self.tab < TAB_COUNT - 1 {
                        self.section = FocusSection::Tabs;
                        self.tab += 1;
                        return NavEffect::SwitchTab(self.tab);
                    }
                }
            },
            FocusSection::GamesCarousel => {
                // carousel_max is enforced by app layer after handle() returns
                self.carousel += 1;
            }
        }
        NavEffect::None
    }

    fn prev_tab(&mut self) -> NavEffect {
        let old = self.tab;
        self.tab = clamp_tab(self.tab as isize - 1);
        self.section = FocusSection::Tabs;
        if self.tab != old {
            NavEffect::SwitchTab(self.tab)
        } else {
            NavEffect::None
        }
    }

    fn next_tab(&mut self) -> NavEffect {
        let old = self.tab;
        self.tab = clamp_tab(self.tab as isize + 1);
        self.section = FocusSection::Tabs;
        if self.tab != old {
            NavEffect::SwitchTab(self.tab)
        } else {
            NavEffect::None
        }
    }

    fn select(&mut self) -> NavEffect {
        match self.section {
            FocusSection::Tabs => {
                let t = self.tab;
                NavEffect::SwitchTab(t)
            }
            FocusSection::Hero => match self.hero {
                HeroPos::LeftColumn(idx) => NavEffect::ActivateTile(idx),
                _ => NavEffect::None,
            },
            FocusSection::GamesCarousel => {
                NavEffect::LaunchGame(self.carousel)
            }
        }
    }

    fn back(&mut self) -> NavEffect {
        // Default: go back to Hero section
        self.section = FocusSection::Hero;
        self.hero = HeroPos::LeftColumn(0);
        NavEffect::None
    }

    // ── Queries for render ───────────────────────────────────────────

    /// Whether the given tab index is focused (for TabBar visual highlight).
    pub fn tab_focused(&self, index: usize) -> bool {
        self.section == FocusSection::Tabs && self.tab == index
    }

    /// The tile index that should be highlighted in the home TileGrid.
    pub fn hero_focused_tile(&self) -> Option<usize> {
        if self.section == FocusSection::Hero {
            match self.hero {
                HeroPos::LeftColumn(idx) => Some(idx),
                HeroPos::Center => Some(3), // hero banner tile
            }
        } else {
            None
        }
    }

    /// Whether focus is currently on a game carousel item.
    pub fn carousel_focused(&self) -> bool {
        self.section == FocusSection::GamesCarousel
    }

    /// The focused tab index (for TabBar visual state).
    pub fn tab(&self) -> usize {
        self.tab
    }

    /// The section focus is currently in.
    pub fn section(&self) -> FocusSection {
        self.section
    }
}
