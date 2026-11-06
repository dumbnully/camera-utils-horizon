fn cu_mods(game: &mut crate::game::Game<'_>, mods: &mut Vec<crate::mod_api::NativeMod>) {
    crate::game::camera_utils::mods(game, mods);
}
