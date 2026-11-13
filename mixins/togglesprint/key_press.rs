fn key_press(game: &mut crate::game::Game<'_>, name: &str, pressed: bool) {
    crate::game::togglesprint_client::key_pressed(game, name, pressed)
}
