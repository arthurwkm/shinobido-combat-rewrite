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
