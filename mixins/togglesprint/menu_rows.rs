fn menu_rows(game: &crate::game::Game<'_>, mod_id: &str, rows: &mut Vec<String>) {
    crate::game::togglesprint_client::settings_rows(game, mod_id, rows)
}
