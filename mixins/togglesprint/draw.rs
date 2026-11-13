fn draw(game: &crate::game::Game<'_>, batch: &mut horizon_client::hud::HudBatch) {
    crate::game::togglesprint_client::paint(game, batch);
}
