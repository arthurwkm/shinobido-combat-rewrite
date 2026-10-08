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
fn all_109_player_actions_have_increasing_root_keys() {
    assert_eq!(ACTIONS.len(), 109);
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
        p.step(true, false);
    }
    assert_eq!(p.state, State::Run);
    assert!((p.pos[2] - 4.0 * 1.69).abs() < 0.05, "{:?}", p.pos); // 4 cycles of 30 frames
    let z = p.pos[2];
    p.step(false, false);
    assert_eq!(p.state, State::Halt);
    for _ in 0..60 {
        p.step(false, false);
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

#[test]
fn attack_chains_only_inside_the_cancel_window() {
    use super::player::{Player, State};
    let run = |press_at: u32| {
        let mut p = Player::new();
        p.step(false, true);
        for f in 1..=press_at {
            p.step(false, f == press_at);
        }
        (p.state, p.world_action())
    };
    assert_eq!(run(10), (State::Attack, "AT_A01")); // window is frames 20..24
    assert_eq!(run(21), (State::Attack, "AT_C01"));
    let mut p = Player::new();
    p.step(false, true);
    for _ in 0..80 {
        p.step(false, false);
    }
    assert_eq!(p.state, State::Idle);
}

#[test]
fn guard_goes_a_then_b_while_held_and_ends_with_y() {
    use super::player::{Player, State};
    let mut p = Player::new();
    p.guard_held = true;
    p.step(false, false);
    assert_eq!(p.world_action(), "GU_A01");
    for _ in 0..8 {
        p.step(false, false);
    }
    assert_eq!(p.world_action(), "GU_B01");
    for _ in 0..300 {
        p.step(false, false);
    }
    assert_eq!((p.state, p.world_action()), (State::Guard, "GU_B01"));
    p.guard_held = false;
    p.step(false, false);
    assert_eq!(p.world_action(), "GU_Y01");
    for _ in 0..26 {
        p.step(false, false);
    }
    assert_eq!(p.state, State::Idle);
}

#[test]
fn dodges_move_by_their_root_motion_then_return_to_idle() {
    use super::player::{Player, State};
    for (dir, axis, want) in [("FRONT01", 2, 1.5), ("BACK01", 2, -1.6), ("RIGHT01", 0, -1.6), ("LEFT01", 0, 1.6)] {
        let mut p = Player::new();
        p.dodge(dir);
        for _ in 0..31 {
            p.step(false, false);
        }
        assert_eq!(p.state, State::Idle, "{dir}");
        assert!((p.pos[axis] - want).abs() < 0.05, "{dir} {:?}", p.pos);
    }
}

#[test]
fn walking_covers_the_walk_cycle_root_motion() {
    use super::player::{Player, State};
    let mut p = Player::new();
    p.walk = true;
    for _ in 0..160 {
        p.step(true, false);
    }
    assert_eq!(p.state, State::Walk);
    assert!((p.pos[2] - 2.0 * 1.65).abs() < 0.05, "{:?}", p.pos); // two 80-frame cycles
}
