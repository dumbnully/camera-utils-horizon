fn cu_bob(game: &crate::game::Game<'_>, partial: f32) -> std::ops::ControlFlow<glam::Mat4> {
    fn original(game: &crate::game::Game<'_>, partial: f32) -> glam::Mat4 {
        let Some(p) = game.player.as_ref() else {
            return glam::Mat4::IDENTITY;
        };
        let hurt = &p.health.hurt;
        let state = BobState {
            living: Some(HurtBob {
                hurt_time: p.health.hurt_animation_time as f32 - partial,
                hurt_duration: hurt.hurt_duration,
                hurt_dir: p.health.hurt_dir,
                death_time: game.death_time().map(|t| t as f32 + partial),
            }),
            player: Some((
                game.avatar.backwards_interpolated_walk_distance(partial),
                game.avatar.interpolated_bob(partial),
            )),
        };
        let tilt = game.settings.file.double("damageTiltStrength");
        glam::Mat4::from_cols_array(
            &bob_matrix(
                &state,
                tilt,
                game.settings.view_bobbing && !crate::game::camera_utils::detached(game),
            )
            .m,
        ) * game.native_shake_matrix(partial)
    }
    if !crate::game::camera_utils::detached(game) {
        return std::ops::ControlFlow::Continue(());
    }
    std::ops::ControlFlow::Break(original(game, partial))
}
