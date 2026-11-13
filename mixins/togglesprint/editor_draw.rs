fn editor_draw(game: &mut crate::game::Game<'_>, batch: &mut horizon_client::hud::HudBatch) -> core::ops::ControlFlow<()> {
    if crate::game::togglesprint_client::editor::draw(game, batch) {
        core::ops::ControlFlow::Break(())
    } else {
        core::ops::ControlFlow::Continue(())
    }
}
