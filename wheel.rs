fn cu_wheel(game: &mut crate::game::Game<'_>, steps: f32, claimed: &mut bool) {
    crate::game::camera_utils::wheel(game, steps, claimed);
}
