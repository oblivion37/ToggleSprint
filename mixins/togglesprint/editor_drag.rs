fn editor_drag(game: &mut crate::game::Game<'_>) -> core::ops::ControlFlow<Result<()>> {
    if crate::game::togglesprint_client::editor::drag(game) {
        core::ops::ControlFlow::Break(Ok(()))
    } else {
        core::ops::ControlFlow::Continue(())
    }
}
