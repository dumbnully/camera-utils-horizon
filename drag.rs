fn cu_drag(
    game: &crate::game::Game<'_>,
    screen: &crate::mod_api::NativeScreenView<'_>,
    canceled: &mut bool,
) {
    crate::game::camera_utils::drag(game, screen, canceled);
}
