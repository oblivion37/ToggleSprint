fn sneak_mapping(game: &crate::game::Game<'_>, slot: usize, vanilla_down: bool) -> bool {
    crate::game::togglesprint_client::logical_key(game, slot, vanilla_down)
}
