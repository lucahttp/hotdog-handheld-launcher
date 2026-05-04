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




//! Navigation state machine + per-tab memory + view stack.
//!
//! Three layers:
//!   FocusState — pure cursor position (section, hero, carousel)
//!   TabMemory  — saved focus per tab (restored on tab switch)
//!   NavEngine  — wraps everything: owns focus + tab memory + view stack
//!
//! Layout:
//!   ┌─ Tabs ─────────────────────────────────────┐
//!   │ Tab0  Tab1  Tab2  Tab3  ...  Tab7          │  ← FocusSection::Tabs
//!   ├────────────────────────────────────────────┤
//!   │ Left[0]   │  Center (HeroTile)             │  ← FocusSection::Hero
//!   │ Left[1]   │                                │
//!   │ Left[2]   │                                │
//!   ├────────────────────────────────────────────┤
//!   │ GameCarousel[0..N]                         │  ← FocusSection::GamesCarousel
//!   ├────────────────────────────────────────────┤
//!   │ (A) Select  (B) Back  (X) Options  (Y) Srch│  ← ButtonHintBar
//!   └────────────────────────────────────────────┘

use crate::input::NavAction;

pub const TAB_COUNT: usize = 8;
pub const MAX_LEFT_COL: usize = 3;
pub const NAV_STACK_MAX: usize = 4;

// ── Section ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusSection {
    Tabs,
    Hero,
    GamesCarousel,
}

// ── HeroPos ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeroPos {
    LeftColumn(usize),
    Center,
}

// ── FocusState (pure cursor, no tab) ─────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FocusState {
    pub section: FocusSection,
    pub hero: HeroPos,
    pub carousel: usize,
}

impl FocusState {
    pub fn home() -> Self {
        Self { section: FocusSection::Hero, hero: HeroPos::LeftColumn(0), carousel: 0 }
    }
    pub fn hero_focused_tile(&self) -> Option<usize> {
        match self.section {
            FocusSection::Hero => match self.hero {
                HeroPos::LeftColumn(idx) => Some(idx),
                HeroPos::Center => Some(3),
            },
            _ => None,
        }
    }
    pub fn carousel_focused(&self) -> bool {
        self.section == FocusSection::GamesCarousel
    }
    pub fn section(&self) -> FocusSection {
        self.section
    }
}

// ── TabMemory ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TabMemory {
    section: FocusSection,
    hero: HeroPos,
    carousel: usize,
}

impl Default for TabMemory {
    fn default() -> Self {
        Self { section: FocusSection::Hero, hero: HeroPos::LeftColumn(0), carousel: 0 }
    }
}

// ── NavEffect ────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NavEffect {
    LaunchGame(usize),
    ActivateTile(usize),
    SwitchTab(usize),
    None,
}

// ── NavCtx ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NavCtx {
    pub can_access_carousel: bool,
    pub carousel_count: usize,
}

impl NavCtx {
    pub fn no_carousel() -> Self { Self { can_access_carousel: false, carousel_count: 0 } }
    pub fn with_carousel(count: usize) -> Self { Self { can_access_carousel: true, carousel_count: count } }
}

// ── Wrap helper ──────────────────────────────────────────────────────

fn wrap_tab(idx: isize) -> usize {
    (((idx % TAB_COUNT as isize) + TAB_COUNT as isize) % TAB_COUNT as isize) as usize
}

// ── FocusState transitions ───────────────────────────────────────────

impl FocusState {
    fn handle_internal(&mut self, action: NavAction, ctx: NavCtx) -> NavEffect {
        match action {
            NavAction::NavigateUp => self.up(),
            NavAction::NavigateDown => self.down(ctx),
            NavAction::NavigateLeft => self.left(),
            NavAction::NavigateRight => self.right(ctx),
            NavAction::Select => self.select(),
            NavAction::Back => self.back(),
            _ => NavEffect::None,
        }
    }

    fn up(&mut self) -> NavEffect {
        match self.section {
            FocusSection::Tabs => { self.section = FocusSection::Hero; self.hero = HeroPos::LeftColumn(0); }
            FocusSection::Hero => match self.hero {
                HeroPos::LeftColumn(idx) if idx > 0 => self.hero = HeroPos::LeftColumn(idx - 1),
                HeroPos::LeftColumn(_) => self.section = FocusSection::Tabs, // top → Tabs
                HeroPos::Center => self.section = FocusSection::Tabs,
            },
            FocusSection::GamesCarousel => { self.section = FocusSection::Hero; }
        }
        NavEffect::None
    }

    fn down(&mut self, ctx: NavCtx) -> NavEffect {
        match self.section {
            FocusSection::Tabs => { self.section = FocusSection::Hero; self.hero = HeroPos::LeftColumn(0); }
            FocusSection::Hero => match self.hero {
                HeroPos::LeftColumn(idx) if idx < MAX_LEFT_COL - 1 => self.hero = HeroPos::LeftColumn(idx + 1),
                HeroPos::LeftColumn(_) if ctx.can_access_carousel => self.section = FocusSection::GamesCarousel,
                HeroPos::LeftColumn(_) => {} // bottom of menu, no carousel → stays
                HeroPos::Center if ctx.can_access_carousel => self.section = FocusSection::GamesCarousel,
                HeroPos::Center => {},
            },
            FocusSection::GamesCarousel => { self.section = FocusSection::Hero; }
        }
        NavEffect::None
    }

    fn left(&mut self) -> NavEffect {
        match self.section {
            FocusSection::Tabs => {}
            FocusSection::Hero => match self.hero {
                HeroPos::LeftColumn(_) => {} // already at left edge
                HeroPos::Center => self.hero = HeroPos::LeftColumn(MAX_LEFT_COL - 1), // Center → last menu
            },
            FocusSection::GamesCarousel if self.carousel > 0 => self.carousel -= 1,
            FocusSection::GamesCarousel => {}
        }
        NavEffect::None
    }

    fn right(&mut self, ctx: NavCtx) -> NavEffect {
        match self.section {
            FocusSection::Tabs => {}
            FocusSection::Hero => match self.hero {
                HeroPos::LeftColumn(_) => self.hero = HeroPos::Center, // any menu → Hero tile
                HeroPos::Center => {} // already at rightmost
            },
            FocusSection::GamesCarousel if ctx.carousel_count > 0 => {
                if self.carousel < ctx.carousel_count - 1 { self.carousel += 1; }
            }
            FocusSection::GamesCarousel => {}
        }
        NavEffect::None
    }

    fn select(&mut self) -> NavEffect {
        match self.section {
            FocusSection::Hero => match self.hero {
                HeroPos::LeftColumn(idx) => NavEffect::ActivateTile(idx),
                _ => NavEffect::None,
            },
            FocusSection::GamesCarousel => NavEffect::LaunchGame(self.carousel),
            FocusSection::Tabs => NavEffect::None,
        }
    }

    fn back(&mut self) -> NavEffect {
        self.section = FocusSection::Hero;
        self.hero = HeroPos::LeftColumn(0);
        NavEffect::None
    }
}

// ── NavEngine ────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct NavEngine {
    pub focus: FocusState,
    pub active_tab: usize,
    tab_memory: [TabMemory; TAB_COUNT],
    stack: Vec<FocusState>,
}

impl NavEngine {
    pub fn new(start_tab: usize) -> Self {
        Self {
            focus: FocusState::home(),
            active_tab: start_tab,
            tab_memory: [TabMemory::default(); TAB_COUNT],
            stack: Vec::with_capacity(NAV_STACK_MAX),
        }
    }

    /// Main entry point for all input.
    pub fn handle(&mut self, action: NavAction, ctx: NavCtx) -> NavEffect {
        // Bumper tab switching — wraparound, restores per-tab focus
        if let Some(tab) = match action {
            NavAction::PreviousTab => Some(wrap_tab(self.active_tab as isize - 1)),
            NavAction::NextTab => Some(wrap_tab(self.active_tab as isize + 1)),
            _ => None,
        } {
            return self.switch_to_tab(tab);
        }

        // ←/→ on Tabs section → switch tabs directly
        if self.focus.section == FocusSection::Tabs {
            match action {
                NavAction::NavigateLeft if self.active_tab > 0 => {
                    return self.switch_to_tab(self.active_tab - 1);
                }
                NavAction::NavigateRight if self.active_tab < TAB_COUNT - 1 => {
                    return self.switch_to_tab(self.active_tab + 1);
                }
                _ => {}
            }
        }

        // Snapshot pre-mutation state for edge-crossing checks
        let pre_section = self.focus.section;
        let pre_hero = self.focus.hero;

        let effect = self.focus.handle_internal(action, ctx);

        // Edge crossing: Right from Hero Center → next tab
        if action == NavAction::NavigateRight && pre_section == FocusSection::Hero && pre_hero == HeroPos::Center {
            if self.active_tab < TAB_COUNT - 1 {
                return self.switch_to_tab(self.active_tab + 1);
            }
        }

        effect
    }

    // ── Tab memory ──────────────────────────────────────────────

    pub fn switch_to_tab(&mut self, tab: usize) -> NavEffect {
        self.save_tab_memory();
        self.active_tab = tab;
        self.load_tab_memory();
        NavEffect::SwitchTab(tab)
    }

    fn save_tab_memory(&mut self) {
        self.tab_memory[self.active_tab] = TabMemory {
            section: self.focus.section,
            hero: self.focus.hero,
            carousel: self.focus.carousel,
        };
    }

    fn load_tab_memory(&mut self) {
        let m = self.tab_memory[self.active_tab];
        self.focus.section = m.section;
        self.focus.hero = m.hero;
        self.focus.carousel = m.carousel;
    }

    // ── View stack ──────────────────────────────────────────────

    pub fn push_view(&mut self, new_focus: FocusState) -> bool {
        if self.stack.len() >= NAV_STACK_MAX { return false; }
        self.save_tab_memory();
        self.stack.push(self.focus);
        self.focus = new_focus;
        true
    }

    pub fn pop_view(&mut self) -> bool {
        if let Some(prev) = self.stack.pop() {
            self.focus = prev;
            self.load_tab_memory();
            true
        } else {
            false
        }
    }

    pub fn in_subview(&self) -> bool { !self.stack.is_empty() }

    // ── Queries ─────────────────────────────────────────────────

    pub fn hero_focused_tile(&self) -> Option<usize> { self.focus.hero_focused_tile() }
    pub fn carousel_focused(&self) -> bool { self.focus.carousel_focused() }
    pub fn section(&self) -> FocusSection { self.focus.section }
    pub fn tab(&self) -> usize { self.active_tab }
}
