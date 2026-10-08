use super::extracted::ACTIONS;

fn action(name: &str) -> &'static super::extracted::Action {
    ACTIONS.iter().find(|a| a.name == name).unwrap()
}

#[test]
fn run_cycle_is_30_frames_and_moves_forward() {
    let run = action("RUN01");
    assert_eq!(run.frames, 30);
    let z = run.root.last().unwrap().1[2] - run.root[0].1[2];
    assert!((z - 1.69).abs() < 0.01, "{z}");
}

#[test]
fn run_footsteps_are_frames_8_and_23() {
    let steps: Vec<i32> = action("RUN01").events.iter().filter(|e| e.1 == 7).map(|e| e.0).collect();
    assert_eq!(steps, [8, 23]);
}

#[test]
fn all_105_player_actions_have_increasing_root_keys() {
    assert_eq!(ACTIONS.len(), 105);
    // Four actions (THRO_J01, FLIP_B01, SUI_HI01, ITEM_HI01) have no key before frame 4, 4, 15, 50.
    assert!(ACTIONS.iter().all(|a| a.root.windows(2).all(|w| w[0].0 < w[1].0)));
}

#[test]
fn stepping_run_for_its_length_covers_its_root_motion() {
    let mut w = super::World::new("RUN01");
    let mut n = 0;
    while !w.done() {
        w.step();
        n += 1;
    }
    assert_eq!(n, 30);
    assert!((w.pos[2] - 1.69).abs() < 0.01, "{:?}", w.pos);
}

#[test]
fn holding_forward_runs_at_the_run_cycle_speed_then_halts_in_place() {
    use super::player::{Player, State};
    let mut p = Player::new();
    for _ in 0..120 {
        p.step(true);
    }
    assert_eq!(p.state, State::Run);
    assert!((p.pos[2] - 4.0 * 1.69).abs() < 0.05, "{:?}", p.pos); // 4 cycles of 30 frames
    let z = p.pos[2];
    p.step(false);
    assert_eq!(p.state, State::Halt);
    for _ in 0..60 {
        p.step(false);
    }
    assert_eq!(p.state, State::Idle);
    assert!((p.pos[2] - z).abs() < 0.2, "halt moved {}", p.pos[2] - z);
}

#[test]
fn first_attack_has_hit_window_14_21_and_cancels_into_ids_30_31_34() {
    let a = action("AT_A01");
    assert_eq!(a.hits, [(14, 21)]);
    assert_eq!(a.cancels, [(30, 20, 24), (31, 20, 24), (34, 20, 26)]);
}
