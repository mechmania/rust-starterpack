use crate::core::*;

pub fn get_strategy() -> Strategy {
    Strategy {
        on_reset: Box::new(goalee_formation),
        on_tick: Box::new(ball_chase),
    }
}

fn goalee_formation(_score: &TeamPair<u32>) -> [Vec2; NUM_PLAYERS as usize] {
    let conf = get_config();

    let field = conf.field.bottom_right();
    [
        Vec2::new(field.x * 0.1, field.y * 0.5),
        Vec2::new(field.x * 0.4, field.y * 0.4),
        Vec2::new(field.x * 0.4, field.y * 0.5),
        Vec2::new(field.x * 0.4, field.y * 0.6),
    ]
}

fn ball_chase(state: &GameState) -> [PlayerAction; NUM_PLAYERS as usize] {
    std::array::from_fn(|id| PlayerAction {
        dir: state.ball.pos - state.players[id].pos,
        pass: None.into()
    })
}
