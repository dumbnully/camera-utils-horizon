fn cu_command(game: &mut crate::game::Game<'_>, command: &str, ran: &mut bool) {
    crate::game::camera_utils::command(game, command, ran);
}
