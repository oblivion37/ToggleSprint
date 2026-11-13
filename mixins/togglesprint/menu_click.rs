fn menu_click(game: &mut crate::game::Game<'_>, mod_id: &str, row: usize) {
    crate::game::togglesprint_client::choose_setting(game, mod_id, row)
}
