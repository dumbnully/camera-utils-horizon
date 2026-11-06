fn cu_mouse(game: &mut crate::game::Game<'_>) -> std::ops::ControlFlow<()> {
    if crate::game::camera_utils::mouse(game) {
        std::ops::ControlFlow::Break(())
    } else {
        std::ops::ControlFlow::Continue(())
    }
}
