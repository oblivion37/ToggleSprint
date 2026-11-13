fn editor_mouse(game: &mut crate::game::Game<'_>, pressed: bool, right: bool) -> core::ops::ControlFlow<Result<()>> {
    if crate::game::togglesprint_client::editor::mouse(game, pressed, right) {
        core::ops::ControlFlow::Break(Ok(()))
    } else {
        core::ops::ControlFlow::Continue(())
    }
}
