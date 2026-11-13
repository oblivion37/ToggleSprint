use super::{Game, Screen, native_hooks::NativeMod};
use horizon_client::{
    actions::Action,
    hud::HudBatch,
    keymapping::{SNEAK, SPRINT},
    text::Text,
    ui,
};
use std::{cell::RefCell, time::Instant};

pub(super) mod editor {
    use super::{Game, MOD_ID, SESSION, Screen, paint_label, preferences};
    use horizon_client::{
        extras::Corner,
        hud::HudBatch,
        hud_layout::{self, HotbarInputs},
        options_ui::Page,
        pvp_hud,
        text::Text,
        ui,
        widgets,
    };
    use winit::keyboard::KeyCode;

    pub(super) const SETTING: &str = "hud_layout";
    const MARGIN: f32 = 14.0;
    const BUTTON_HEIGHT: f32 = 20.0;
    const PREVIEW_PADDING: f32 = 3.0;
    const FONT_HEIGHT: f32 = 9.0;
    const MAX_POP_RATIO: f32 = 1.35 / 1.22;

    #[derive(Clone, Copy, Debug, PartialEq)]
    enum Position {
        Corner(Corner),
        Free(f32, f32),
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub(super) struct Layout {
        position: Position,
        size: u8,
    }

    impl Default for Layout {
        fn default() -> Self {
            Self { position: Position::Corner(Corner::BottomLeft), size: 50 }
        }
    }

    impl Layout {
        pub(super) fn load(saved: Option<&str>, old_corner: Option<&str>) -> Self {
            if let Some(layout) = saved.and_then(Self::parse) {
                return layout;
            }
            let corner = old_corner.map_or(Corner::BottomLeft, |corner| {
                Corner::parse(corner, Corner::BottomLeft)
            });
            Self { position: Position::Corner(corner), ..Self::default() }
        }

        fn parse(saved: &str) -> Option<Self> {
            let mut parts = saved.split_whitespace();
            let size: u8 = parts.next()?.parse().ok()?;
            let position = match (parts.next()?, parts.next(), parts.next()) {
                (corner, None, None) => Position::Corner(Corner::parse(corner, Corner::BottomLeft)),
                (x, Some(y), None) => {
                    let (x, y): (f32, f32) = (x.parse().ok()?, y.parse().ok()?);
                    if !x.is_finite() || !y.is_finite() {
                        return None;
                    }
                    Position::Free(x.clamp(0.0, 1.0), y.clamp(0.0, 1.0))
                }
                _ => return None,
            };
            Some(Self { position, size: size.min(100) })
        }

        fn encode(self) -> String {
            match self.position {
                Position::Corner(corner) => format!("{} {}", self.size, corner.name()),
                Position::Free(x, y) => format!("{} {x} {y}", self.size),
            }
        }

        pub(super) fn scale(self) -> f32 {
            super::SETTLED_SCALE * (1.0 + (f32::from(self.size) - 50.0) / 125.0)
        }

        fn extent(self, width: f32) -> (f32, f32) {
            let peak = self.scale() * MAX_POP_RATIO;
            (width * peak, FONT_HEIGHT * peak)
        }

        pub(super) fn placement(self, screen: (f32, f32), width: f32, scale: f32) -> (f32, f32) {
            let (w, h) = self.extent(width);
            let (x, y) = match self.position {
                Position::Corner(corner) => {
                    let (room_x, room_y) = ((screen.0 - w).max(0.0), (screen.1 - h).max(0.0));
                    let (inset_x, inset_y) = (MARGIN.min(room_x / 2.0), MARGIN.min(room_y / 2.0));
                    (
                        if corner.left() { inset_x } else { room_x - inset_x },
                        if corner.top() { inset_y } else { room_y - inset_y },
                    )
                }
                Position::Free(x, y) => pvp_hud::from_fractions((x, y), (w, h), screen),
            };
            (x + (w - width * scale) / 2.0, y + (h - FONT_HEIGHT * scale) / 2.0)
        }

        fn move_to(&mut self, screen: (f32, f32), width: f32, (x, y): (f32, f32)) {
            let scale = self.scale();
            let (w, h) = self.extent(width);
            let origin = (x - (w - width * scale) / 2.0, y - (h - FONT_HEIGHT * scale) / 2.0);
            let (fx, fy) = pvp_hud::to_fractions(origin, (w, h), screen);
            self.position = Position::Free(fx, fy);
        }

        pub(super) fn corner(self) -> Option<Corner> {
            match self.position {
                Position::Corner(corner) => Some(corner),
                Position::Free(..) => None,
            }
        }
    }

    #[derive(Clone, Copy)]
    enum Drag {
        Label { offset: (f32, f32) },
        Size(Rect),
    }

    #[derive(Default)]
    pub(super) struct Editor {
        drag: Option<Drag>,
        dirty: bool,
    }

    fn on_settings_page(game: &Game<'_>) -> bool {
        if game.screen != Screen::Options {
            return false;
        }
        let Page::NativeMod(index) = game.ui.options.page() else {
            return false;
        };
        game.settings.mods.installed.get(index).is_some_and(|mod_entry| mod_entry.id == MOD_ID)
    }

    pub(super) fn open(game: &mut Game<'_>) {
        if on_settings_page(game) && !active(game) {
            preferences(game);
            SESSION.with_borrow_mut(|session| session.editor = Some(Editor::default()));
        }
    }

    pub(in crate::game) fn active(game: &Game<'_>) -> bool {
        on_settings_page(game) && SESSION.with_borrow(|session| session.editor.is_some())
    }

    fn save_layout(game: &Game<'_>) {
        if let Some(layout) = SESSION.with_borrow_mut(|session| {
            let editor = session.editor.as_mut()?;
            std::mem::take(&mut editor.dirty).then(|| session.preferences.unwrap().layout)
        }) {
            game.native_set_setting(MOD_ID, SETTING, &layout.encode());
        }
    }

    fn close(game: &mut Game<'_>) {
        save_layout(game);
        SESSION.with_borrow_mut(|session| session.editor = None);
        game.dragging = false;
    }

    pub(super) fn update(game: &mut Game<'_>) {
        if SESSION.with_borrow(|session| session.editor.is_some()) {
            if !on_settings_page(game) {
                close(game);
            } else if !game.focused {
                end_drag(game);
            }
        }
    }

    #[derive(Clone, Copy)]
    struct Rect {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    }

    impl Rect {
        fn contains(self, cursor: (f32, f32)) -> bool {
            ui::hovered(cursor, self.x, self.y, self.w, self.h)
        }

        fn overlaps(self, other: Self) -> bool {
            self.x < other.x + other.w
                && other.x < self.x + self.w
                && self.y < other.y + other.h
                && other.y < self.y + self.h
        }
    }

    fn toolbar((width, height): (f32, f32), label: Rect, drag: Option<Drag>) -> [Rect; 3] {
        let gap = 4.0;
        let total = (width - 16.0).clamp(36.0, 300.0);
        let button_width = ((total - gap * 2.0) / 4.0).floor();
        let slider_width = total - button_width * 2.0 - gap * 2.0;
        let left = ((width - total) / 2.0).floor();
        let top = Rect { x: left, y: 8.0, w: total, h: BUTTON_HEIGHT };
        let y = match drag {
            Some(Drag::Size(track)) => track.y,
            _ if top.overlaps(label) => (height - 48.0).max(top.y),
            _ => top.y,
        };
        let rect = |x, w| Rect { x, y, w, h: BUTTON_HEIGHT };
        [
            rect(left, button_width),
            rect(left + button_width + gap, slider_width),
            rect(left + button_width + gap + slider_width + gap, button_width),
        ]
    }

    fn preview(layout: Layout, screen: (f32, f32), width: f32) -> Rect {
        let scale = layout.scale();
        let (x, y) = layout.placement(screen, width, scale);
        Rect {
            x: x - PREVIEW_PADDING,
            y: y - PREVIEW_PADDING,
            w: width * scale + 2.0 * PREVIEW_PADDING,
            h: FONT_HEIGHT * scale + 2.0 * PREVIEW_PADDING,
        }
    }

    fn draw_hotbar(game: &Game<'_>, batch: &mut HudBatch) {
        let inputs = HotbarInputs {
            selected: game.selected.min(8) as i32,
            right_handed: !game.settings.main_hand_left,
            offhand: false,
            slots: [None; 9],
            offhand_pop: 0.0,
            attack: None,
        };
        let draws = hud_layout::hotbar(batch.width as i32, batch.height as i32, &inputs);
        hud_layout::paint(batch, &game.gui.atlas, &draws);
    }

    pub(in crate::game) fn draw(game: &Game<'_>, batch: &mut HudBatch) -> bool {
        if !active(game) {
            return false;
        }
        let atlas = &game.gui.atlas;
        let screen = (batch.width, batch.height);
        let width = game.native_text_width(super::LABEL);
        let layout = preferences(game).layout;
        let drag = SESSION.with_borrow(|session| session.editor.as_ref().and_then(|editor| editor.drag));
        widgets::begin_screen(|| widgets::ScreenInfo::new("options/togglesprint/hud_position", "HUD"));
        draw_hotbar(game, batch);

        let preview = preview(layout, screen, width);
        let [reset, size, done] = toolbar(screen, preview, drag);
        for (rect, label) in [(reset, "controls.reset"), (done, "gui.done")] {
            ui::button(
                batch, atlas, rect.x, rect.y, rect.w, rect.h, &Text::translate(label, &[]),
                true, rect.contains(game.cursor),
            );
        }
        ui::slider(
            batch, atlas, size.x, size.y, size.w, &Text::literal(format!("Size: {}", layout.size)),
            f32::from(layout.size) / 100.0, size.contains(game.cursor), true,
            matches!(drag, Some(Drag::Size(_))),
        );

        let selected = matches!(drag, Some(Drag::Label { .. })) || preview.contains(game.cursor);
        let Rect { x, y, w, h } = preview;
        let (right, bottom) = ((x + w).min(screen.0), (y + h).min(screen.1));
        let (x, y) = (x.max(0.0), y.max(0.0));
        let (w, h) = (right - x, bottom - y);
        let outline = if selected { 0xFFFF_FFFF } else { 0x99FF_FFFF };
        game.native_fill(batch, [x, y, w, 1.0], outline);
        game.native_fill(batch, [x, y + h - 1.0, w, 1.0], outline);
        game.native_fill(batch, [x, y + 1.0, 1.0, h - 2.0], outline);
        game.native_fill(batch, [x + w - 1.0, y + 1.0, 1.0, h - 2.0], outline);
        paint_label(game, batch, layout, layout.scale());
        true
    }

    pub(in crate::game) fn mouse(game: &mut Game<'_>, pressed: bool, right: bool) -> bool {
        if !active(game) {
            return false;
        }
        if right {
            return true;
        }
        if !pressed {
            end_drag(game);
            return true;
        }

        let cursor = game.cursor;
        let screen = game.gui_size();
        let width = game.native_text_width(super::LABEL);
        let layout = preferences(game).layout;
        let preview = preview(layout, screen, width);
        let [reset, size, done] = toolbar(screen, preview, None);
        if preview.contains(cursor) {
            let (x, y) = layout.placement(screen, width, layout.scale());
            start_drag(game, Drag::Label { offset: (cursor.0 - x, cursor.1 - y) });
        } else if reset.contains(cursor) {
            set_layout(Layout::default());
            save_layout(game);
            game.audio.play_button_click();
        } else if done.contains(cursor) {
            game.audio.play_button_click();
            close(game);
        } else if size.contains(cursor) {
            start_drag(game, Drag::Size(size));
            drag(game);
        }
        true
    }

    fn end_drag(game: &mut Game<'_>) {
        let dragging = SESSION.with_borrow_mut(|session| {
            session.editor.as_mut().and_then(|editor| editor.drag.take()).is_some()
        });
        if dragging {
            game.dragging = false;
            save_layout(game);
        }
    }

    fn start_drag(game: &mut Game<'_>, drag: Drag) {
        game.dragging = true;
        SESSION.with_borrow_mut(|session| session.editor.as_mut().unwrap().drag = Some(drag));
    }

    fn set_layout(layout: Layout) {
        SESSION.with_borrow_mut(|session| {
            let prefs = session.preferences.as_mut().unwrap();
            if prefs.layout != layout {
                prefs.layout = layout;
                session.editor.as_mut().unwrap().dirty = true;
            }
        });
    }

    pub(in crate::game) fn drag(game: &mut Game<'_>) -> bool {
        if !active(game) {
            return false;
        }
        let Some(drag) = SESSION.with_borrow(|session| session.editor.as_ref().unwrap().drag) else {
            return true;
        };
        let mut layout = preferences(game).layout;
        let screen = game.gui_size();
        match drag {
            Drag::Label { offset: (dx, dy) } => {
                let width = game.native_text_width(super::LABEL);
                layout.move_to(screen, width, (game.cursor.0 - dx, game.cursor.1 - dy));
            }
            Drag::Size(size) => {
                layout.size = (ui::slider_value(size.x, size.w, game.cursor.0) * 100.0).round() as u8;
            }
        }
        set_layout(layout);
        true
    }

    pub(in crate::game) fn key(game: &mut Game<'_>, key: KeyCode) -> bool {
        if !active(game) {
            return false;
        }
        if matches!(key, KeyCode::Escape | KeyCode::Enter | KeyCode::NumpadEnter) {
            close(game);
        }
        true
    }

    pub(in crate::game) fn scroll(game: &Game<'_>) -> bool {
        active(game)
    }
}

const MOD_ID: &str = "togglesprint";
const HUD_CORNER: &str = "hud_corner";
const CROUCH_ENABLED: &str = "crouch_enabled";
const CROUCH_KEY: &str = "toggle_crouch";
const DEFAULT_KEY: &str = "key.keyboard.c";
const LABEL: &str = "Sprint";
const SETTLED_SCALE: f32 = 1.22;
const PEAK_SCALE: f32 = 1.35;

#[derive(Clone, Copy)]
struct Preferences {
    layout: editor::Layout,
    crouch_enabled: bool,
}

struct ClientSession {
    preferences: Option<Preferences>,
    sprint_since: Option<Instant>,
    latched_binding: Option<String>,
    editor: Option<editor::Editor>,
}

thread_local! {
    static SESSION: RefCell<ClientSession> = const { RefCell::new(ClientSession {
        preferences: None,
        sprint_since: None,
        latched_binding: None,
        editor: None,
    }) };
}

fn preferences(game: &Game<'_>) -> Preferences {
    SESSION.with_borrow_mut(|session| {
        *session.preferences.get_or_insert_with(|| Preferences {
            layout: editor::Layout::load(
                game.native_setting(MOD_ID, editor::SETTING).as_deref(),
                game.native_setting(MOD_ID, HUD_CORNER).as_deref(),
            ),
            crouch_enabled: game.native_setting(MOD_ID, CROUCH_ENABLED).as_deref() == Some("true"),
        })
    })
}

fn ready(game: &Game<'_>) -> bool {
    game.native_playing()
        && game.player.is_some()
        && game.sleeping.is_none()
        && !game.screen_over_playing()
}

fn crouch_binding<'a>(game: &'a Game<'_>) -> &'a str {
    game.settings
        .mods
        .keys
        .get(MOD_ID)
        .and_then(|keys| keys.get(CROUCH_KEY))
        .map_or(DEFAULT_KEY, String::as_str)
}

fn vanilla_uses_toggle_binding(game: &Game<'_>, binding: &str) -> bool {
    game.settings.toggle_crouch && game.settings.keys.get("key.sneak") == binding
}

fn binding_still_valid(game: &Game<'_>, binding: &str) -> bool {
    binding == crouch_binding(game) && !vanilla_uses_toggle_binding(game, binding)
}

pub(super) fn update_frame(game: &mut Game<'_>) {
    if !game.native_window() {
        return;
    }

    editor::update(game);
    let in_world = game.title.is_none() && game.player.is_some();
    let enabled = game.native_on(MOD_ID);
    let ready = ready(game);
    let sprint_toggle = game.settings.toggle_sprint;
    let needs_crouch_sync = SESSION.with_borrow_mut(|session| {
        if !enabled || !in_world {
            session.sprint_since = None;
            return session.latched_binding.take().is_some();
        }

        if !sprint_toggle || (ready && !game.toggles.is_down(SPRINT)) {
            session.sprint_since = None;
        } else if ready {
            session.sprint_since.get_or_insert_with(Instant::now);
        }

        let Some(binding) = session.latched_binding.as_deref() else {
            return false;
        };
        let keep_latch = binding_still_valid(game, binding);
        if !keep_latch {
            session.latched_binding = None;
        }
        // Menus, focus changes and rebinding can change the logical key between presses.
        let down = game.toggles.is_down(SNEAK) || (ready && keep_latch);
        game.keys.held(Action::Sneak) != down
    });
    if needs_crouch_sync {
        game.sync_toggle(SNEAK);
    }
}

pub(super) fn key_pressed(game: &mut Game<'_>, name: &str, pressed: bool) {
    if !pressed || !game.native_window() || !game.native_on(MOD_ID) || !ready(game) {
        return;
    }
    if !binding_still_valid(game, name) || !preferences(game).crouch_enabled {
        return;
    }

    SESSION.with_borrow_mut(|session| {
        session.latched_binding = if session.latched_binding.as_deref() == Some(name) {
            None
        } else {
            Some(name.to_owned())
        };
    });
    game.sync_toggle(SNEAK);
}

pub(super) fn logical_key(game: &Game<'_>, slot: usize, vanilla_down: bool) -> bool {
    vanilla_down
        || (slot == SNEAK
            && game.native_window()
            && game.native_on(MOD_ID)
            && ready(game)
            && SESSION.with_borrow(|session| {
                session
                    .latched_binding
                    .as_deref()
                    .is_some_and(|binding| binding_still_valid(game, binding))
            }))
}

fn eased_scale(elapsed_ms: f32) -> f32 {
    let (start, end, fraction) = if elapsed_ms <= 65.0 {
        (1.04, PEAK_SCALE, elapsed_ms / 65.0)
    } else {
        (PEAK_SCALE, SETTLED_SCALE, ((elapsed_ms - 65.0) / 125.0).min(1.0))
    };
    let weight = fraction * fraction * (3.0 - 2.0 * fraction);
    start * (1.0 - weight) + end * weight
}

pub(super) fn paint(game: &Game<'_>, batch: &mut HudBatch) {
    if !game.native_on(MOD_ID)
        || !ready(game)
        || !game.settings.toggle_sprint
        || !game.toggles.is_down(SPRINT)
    {
        return;
    }
    let layout = preferences(game).layout;
    if layout.corner().is_some_and(|corner| !game.native_corner_shown(corner)) {
        return;
    }
    let since = SESSION.with_borrow(|session| session.sprint_since);
    let pulse = since.map_or(1.0, |start| {
        eased_scale(start.elapsed().as_secs_f32() * 1000.0) / SETTLED_SCALE
    });
    paint_label(game, batch, layout, layout.scale() * pulse);
}

fn paint_label(game: &Game<'_>, batch: &mut HudBatch, layout: editor::Layout, scale: f32) {
    let width = game.native_text_width(LABEL);
    let (x, y) = layout.placement((batch.width, batch.height), width, scale);
    ui::text_scaled(batch, &game.gui.atlas, &Text::literal(LABEL), x, y, scale, u32::MAX);
}

pub(super) fn add_mod_row(mods: &mut Vec<NativeMod>) {
    mods.push(NativeMod {
        id: MOD_ID.to_owned(),
        title: "ToggleSprint".to_owned(),
        description: "Horizon ToggleSprint mod.".to_owned(),
        keys: vec![[
            CROUCH_KEY.to_owned(),
            "ToggleCrouch bind".to_owned(),
            DEFAULT_KEY.to_owned(),
        ]],
    });
}

pub(super) fn settings_rows(game: &Game<'_>, mod_id: &str, rows: &mut Vec<String>) {
    if mod_id == MOD_ID {
        let prefs = preferences(game);
        let toggle = if prefs.crouch_enabled { "ON" } else { "OFF" };
        rows.extend([
            "HUD".to_owned(),
            format!("ToggleCrouch: {toggle}"),
        ]);
    }
}

pub(super) fn choose_setting(game: &mut Game<'_>, mod_id: &str, row: usize) {
    if mod_id != MOD_ID {
        return;
    }
    let mut prefs = preferences(game);
    match row {
        0 => {
            editor::open(game);
            return;
        }
        1 => {
            prefs.crouch_enabled = !prefs.crouch_enabled;
            game.native_set_setting(MOD_ID, CROUCH_ENABLED, &prefs.crouch_enabled.to_string());
            SESSION.with_borrow_mut(|session| session.latched_binding = None);
            game.sync_toggle(SNEAK);
        }
        _ => return,
    }
    SESSION.with_borrow_mut(|session| session.preferences = Some(prefs));
}
