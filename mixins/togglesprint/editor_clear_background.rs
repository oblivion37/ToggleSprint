fn editor_clear_background(game: &crate::game::Game<'_>, radius: &mut Option<f32>) {
    if crate::game::togglesprint_client::editor::active(game) {
        *radius = None;
    }
}
