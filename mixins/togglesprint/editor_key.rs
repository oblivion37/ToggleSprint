fn editor_key(game: &mut crate::game::Game<'_>, key: winit::keyboard::KeyCode) -> core::ops::ControlFlow<Result<()>> {
    if crate::game::togglesprint_client::editor::key(game, key) {
        core::ops::ControlFlow::Break(Ok(()))
    } else {
        core::ops::ControlFlow::Continue(())
    }
}
