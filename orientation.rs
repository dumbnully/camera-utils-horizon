fn cu_orientation(game: &crate::game::Game<'_>) -> std::ops::ControlFlow<glam::Quat> {
    if crate::game::camera_utils::special(game) {
        std::ops::ControlFlow::Break(camera_rotation(
            perspective::CameraType::FirstPerson,
            game.view.y_rot,
            game.view.x_rot,
        ))
    } else {
        std::ops::ControlFlow::Continue(())
    }
}
