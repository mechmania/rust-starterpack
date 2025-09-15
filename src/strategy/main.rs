use crate::core::*;

// This function tells the engine what strategy you want your bot to use
pub fn get_strategy(team: u8) -> Strategy {

    // team == 0 means I am on the left
    // team == 1 means I am on the right

    if team == 0 {
        println!("Hello! I am team A (on the left)");
        Strategy {
            on_reset: Box::new(goalee_formation),
            on_tick: Box::new(ball_chase),
        }
    } else {
        println!("Hello! I am team B (on the right)");
        Strategy {
            on_reset: Box::new(goalee_formation),
            on_tick: Box::new(do_nothing),
        }
    }
    // NOTE when actually submitting your bot, you probably want to have the SAME strategy for both
    // sides.
}

// The engine will call this function every time the field is reset:
// either after a goal, if the ball has not moved for too long, or right before endgame
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

// Very simple strategy to chase the ball and shoot on goal
fn ball_chase(state: &GameState) -> [PlayerAction; NUM_PLAYERS as usize] {
    let conf = get_config();

    // NOTE Do not worry about what side your bot is on! 
    // The engine mirrors the world for you if you are on the right, 
    // so to you, you always appear on the left.

    std::array::from_fn(|id| PlayerAction {
        dir: state.ball.pos - state.players[id].pos,
        pass: pass(conf.field.goal_other() - state.players[id].pos)
    })
}

// This strategy will do nothing :(
fn do_nothing(_state: &GameState) -> [PlayerAction; NUM_PLAYERS as usize] {
    std::array::from_fn(|_id| PlayerAction {
        dir: Vec2::ZERO,
        pass: no_pass()
    })
}
