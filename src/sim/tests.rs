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
fn all_178_player_actions_have_increasing_root_keys() {
    assert_eq!(ACTIONS.iter().filter(|a| !a.name.ends_with("07")).count(), 178);
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

#[test]
fn jump_leaves_the_ground_at_frame_5_and_lands_about_51_frames_later() {
    use super::player::{Player, State};
    let mut p = Player::new();
    p.jump();
    let (mut peak, mut land) = (0.0f32, None);
    for f in 1..=120 {
        p.step(false, false);
        peak = peak.max(p.pos[1]);
        if p.state == State::Land && land.is_none() {
            land = Some(f);
        }
    }
    assert!((55..=62).contains(&land.unwrap()), "{land:?}");
    assert!(peak > 0.5 && peak < 1.5, "{peak}");
    assert_eq!(p.state, State::Idle);
}

#[test]
fn jump_carries_run_momentum_and_the_stick_steers_in_the_air() {
    use super::player::{Player, State};
    let mut p = Player::new();
    for _ in 0..40 { p.step(true, false); }
    let z0 = p.pos[2];
    p.jump();
    while p.state == State::Jump { p.step(false, false); }
    assert!(p.pos[2] - z0 > 1.5, "momentum carried: {}", p.pos[2] - z0);
    let mut q = Player::new();
    q.jump();
    q.target_heading = std::f32::consts::FRAC_PI_2;
    while q.state == State::Jump { q.step(true, false); }
    assert!(q.pos[0] > 1.0 && q.heading > 1.0, "steered from a standing jump: {:?} {}", q.pos, q.heading);
}

#[test]
fn turning_around_takes_pi_over_turn_rate_then_run_goes_backwards() {
    let mut p = super::player::Player::new();
    p.target_heading = std::f32::consts::PI;
    let mut n = 0;
    while p.heading.cos() > -0.99999 {
        p.step(false, false);
        n += 1;
        assert!(n < 100);
    }
    assert_eq!(n, 24); // pi / (7.854 / 60)
    for _ in 0..30 {
        p.step(true, false);
    }
    assert!(p.pos[2] < -1.6 && p.pos[0].abs() < 0.01, "{:?}", p.pos);
}

#[test]
fn walking_backward_and_sideways_follows_each_clip_root_motion() {
    use super::player::Player;
    for (clip, axis, want) in [("WALK_B02", 2, -1.55), ("WALK_R02", 0, -1.6), ("WALK_L02", 0, 1.6)] {
        let mut p = Player::new();
        p.walk = true;
        p.walk_clip = clip;
        for _ in 0..80 {
            p.step(true, false);
        }
        assert_eq!(p.world_action(), clip);
        assert!((p.pos[axis] - want).abs() < 0.05, "{clip} {:?}", p.pos);
    }
}

#[test]
fn hit_reactions_play_out_then_return_to_idle() {
    use super::{extracted::ACTIONS, player::{Player, State}};
    for a in ACTIONS.iter().filter(|a| a.name.starts_with("HI_")) {
        let mut p = Player::new();
        p.guard_held = true;
        p.step(false, false);
        p.hit(a.name);
        assert_eq!((p.state, p.world_action()), (State::Hit, a.name));
        for _ in 0..a.frames {
            p.step(false, false);
        }
        assert_eq!(p.state, State::Hit, "{} still plays on its last frame", a.name);
        p.step(false, false);
        assert_eq!(p.state, State::Idle, "{}", a.name);
    }
}

#[test]
fn ninja_actions_play_out_then_return_to_idle() {
    use super::{extracted::ACTIONS, player::{Player, State}};
    for n in ["SHEATH01", "PICK01", "THROW_A01", "THROW_B01", "DRINK01", "NINJYUTU01", "MAKIBISHI01"] {
        let a = ACTIONS.iter().find(|a| a.name == n).unwrap();
        let mut p = Player::new();
        p.special(n);
        assert_eq!((p.state, p.world_action()), (State::Special, n));
        for _ in 0..a.frames {
            p.step(false, false);
        }
        assert_eq!(p.state, State::Special, "{n} still plays on its last frame");
        p.step(false, false);
        assert_eq!(p.state, State::Idle, "{n}");
    }
}

#[test]
fn at_a01_damage_param_scales_with_power_and_bias() {
    let e = action("AT_A01").events.iter().find(|e| e.1 == 35).unwrap();
    assert_eq!(e.2, 7);
    assert_eq!(super::player::hit_damage(e.2, 10.0, 0.0), 70);
    assert_eq!(super::player::hit_damage(e.2, 10.0, -1.0), 35);
    assert_eq!(super::player::hit_damage(e.2, 10.0, 1.0), 140);
}

#[test]
fn terrain_walls_ramps_ledges_and_falling() {
    use super::{terrain::{Collider, Terrain}, player::Player};
    let terrain = Terrain { colliders: vec![Collider::new([ -2.0, 0.0, 1.0], [2.0, 2.0, 2.0], false)] };
    let mut p = Player::new();
    for _ in 0..120 { p.step_on(&terrain, true, false); }
    assert!(p.pos[2] <= 0.701 && p.pos[2] > 0.6, "{:?}", p.pos);
    assert_eq!(p.wall_normal, Some([0.0, 0.0, -1.0]));
    assert_eq!(terrain.ledge(p.pos, [0.0, 1.0], 2.1), Some([0.0, 2.0, 1.0]));
    p.pos = [0.0, 2.0, 1.5];
    for _ in 0..120 { p.step_on(&terrain, true, false); }
    assert_eq!(p.pos[1], 0.0);
    assert!(p.pos[2] > 3.0);
    let ramp = Terrain { colliders: vec![Collider::new([-2.0, 0.0, 0.0], [2.0, 1.0, 4.0], true)] };
    let mut p = Player::new();
    for _ in 0..50 { p.step_on(&ramp, true, false); }
    assert!((p.pos[1] - p.pos[2] / 4.0).abs() < 0.001, "{:?}", p.pos);
    assert!(p.pos[1] > 0.5);
}

#[test]
fn terrain_jump_lands_on_roof_and_ceiling_stops_ascent() {
    use super::{terrain::{Collider, Terrain}, player::{Player, State}};
    let roof = Terrain { colliders: vec![Collider::new([-2.0, 0.0, -2.0], [2.0, 2.0, 2.0], false)] };
    let mut p = Player::new();
    p.pos[1] = 2.0;
    p.jump();
    let mut peak = 2.0_f32;
    for _ in 0..120 { p.step_on(&roof, false, false); peak = peak.max(p.pos[1]); }
    assert!(peak > 2.5);
    assert_eq!(p.pos[1], 2.0);
    assert_eq!(p.state, State::Idle);
    let ceiling = Terrain { colliders: vec![Collider::new([-2.0, 1.9, -2.0], [2.0, 2.0, 2.0], false)] };
    let mut p = Player::new();
    p.jump();
    for _ in 0..120 { p.step_on(&ceiling, false, false); assert!(p.pos[1] <= 0.201); }
    assert_eq!(p.pos[1], 0.0);
}

#[test]
fn terrain_sweeps_thin_walls_slides_and_descends_ramps() {
    use super::terrain::{Collider, Terrain};
    let wall = Terrain { colliders: vec![Collider::new([-2.0, 0.0, 1.0], [2.0, 3.0, 1.01], false)] };
    let (p, normal, _) = wall.move_body([0.0; 3], [1.0, 0.0, 5.0], true);
    assert!(p[2] <= 0.701);
    assert!((p[0] - 1.0).abs() < 0.001);
    assert_eq!(normal, Some([0.0, 0.0, -1.0]));
    let ramp = Terrain { colliders: vec![Collider::new([-2.0, 0.0, 0.0], [2.0, 1.0, 4.0], true)] };
    let (p, _, _) = ramp.move_body([0.0, 0.75, 3.0], [0.0, 0.75, 1.0], true);
    assert!((p[1] - 0.25).abs() < 0.001, "{p:?}");
    let (p, normal, _) = ramp.move_body([0.0, 0.0, 5.0], [0.0, 0.0, 3.0], true);
    assert!(p[2] >= 4.3 - 0.001);
    assert_eq!(normal, Some([0.0, 0.0, 1.0]));
    assert_eq!(wall.ledge([0.0; 3], [0.0, -1.0], 4.0), None);
    assert_eq!(wall.ledge([0.0; 3], [0.0, 1.0], 2.0), None);
    let blocked = Terrain { colliders: vec![wall.colliders[0], Collider::new([-2.0, 3.5, 0.0], [2.0, 4.0, 2.0], false)] };
    assert_eq!(blocked.ledge([0.0; 3], [0.0, 1.0], 3.1), None);
}

#[test]
fn run_variants_loop_switch_and_stop_with_extracted_motion() {
    use super::player::{Player, State};
    let mut p = Player::new();
    for name in ["RUN01", "RUN_R01", "RUN_L01"] {
        p.run_clip = name;
        let before = p.pos;
        let a = action(name);
        for _ in 0..2 * a.frames { p.step(true, false); }
        assert_eq!((p.state, p.world_action()), (State::Run, name));
        for (i, start) in before.iter().enumerate() {
            let delta = a.root.last().unwrap().1[i] - a.root[0].1[i];
            assert!((p.pos[i] - start - 2.0 * delta).abs() < 0.001, "{name}: {:?}", p.pos);
        }
    }
    p.step(false, false);
    assert_eq!((p.state, p.world_action()), (State::Halt, "HALT01"));
    p.step(true, false);
    assert_eq!((p.state, p.world_action()), (State::Run, "RUN_L01"));
    p.walk = true;
    p.step(true, false);
    assert_eq!(p.state, State::Walk);
}

#[test]
fn idle_variants_loop_and_resume_after_interruptions() {
    use super::player::{Player, State};
    let mut p = Player::new();
    for name in ["TIRED01", "STOP02", "STOP01"] {
        p.idle_clip = name;
        for _ in 0..2 * action(name).frames + 1 {
            p.step(false, false);
            assert_eq!((p.state, p.world_action()), (State::Idle, name));
        }
        p.step(true, false);
        assert_eq!(p.state, State::Run);
        for _ in 0..action("HALT01").frames + 1 { p.step(false, false); }
        assert_eq!((p.state, p.world_action()), (State::Idle, name));
        p.special("PICK01");
        for _ in 0..action("PICK01").frames + 1 { p.step(false, false); }
        assert_eq!((p.state, p.world_action()), (State::Idle, name));
    }
}

#[test]
fn run_end_and_wall_stop_variants_play_once_then_selected_idle() {
    use super::player::{Player, State};
    for name in ["RUN_RE01", "RUN_LE01", "STOP_K01"] {
        let mut p = Player::new();
        p.idle_clip = "STOP02";
        p.special(name);
        let a = action(name);
        for _ in 0..a.frames {
            p.step(false, false);
            assert_eq!((p.state, p.world_action()), (State::Special, name));
        }
        p.step(false, false);
        assert_eq!((p.state, p.world_action()), (State::Idle, "STOP02"));
    }
}

#[test]
fn locomotion_selection_switches_mid_cycle_and_rejects_other_clips() {
    use super::player::{Player, State};
    let mut p = Player::new();
    p.step(true, false);
    p.run_clip = "RUN_R01";
    let before = p.pos;
    p.step(true, false);
    let mut first = super::World::new("RUN_R01");
    first.step();
    assert_eq!(p.world_action(), "RUN_R01");
    assert!((p.pos[2] - before[2] - first.pos[2]).abs() < 0.001);
    p.run_clip = "RUN_RE01"; // one-shot, not a locomotion loop
    p.step(true, false);
    assert_eq!((p.state, p.world_action()), (State::Run, "RUN01"));
    p.idle_clip = "missing";
    for _ in 0..action("HALT01").frames + 1 { p.step(false, false); }
    assert_eq!((p.state, p.world_action()), (State::Idle, "STOP01"));
}

#[test]
fn walking_off_a_ledge_plays_fall_then_landing_by_drop_and_ground_jumps_are_actions() {
    use super::{terrain::{Collider, Terrain}, player::{Player, State}};
    let terrain = Terrain { colliders: vec![Collider::new([-2.0, 0.0, -2.0], [2.0, 5.0, 2.0], false)] };
    let mut p = Player::new();
    p.pos = [0.0, 5.0, 1.5];
    let (mut fell, mut land) = (false, "");
    for _ in 0..200 {
        p.step_on(&terrain, true, false);
        fell |= p.state == State::Fall && p.world_action() == "FALL01";
        if p.state == State::Land && land.is_empty() { land = p.world_action(); }
    }
    assert!(fell);
    assert_eq!(land, "LAND_A01");
    let mut p = Player::new();
    p.special("JUMP_F02");
    for _ in 0..40 { p.step(false, false); }
    assert!(p.pos[2] > 3.0 && p.state == State::Idle, "{:?}", p.pos);
}

#[test]
fn flips_and_jump_attacks_play_once_then_idle() {
    use super::player::{Player, State};
    // ESTIMATE: triggers are caller-chosen one-shots with no cancels (KICK_J/THRO_J are ground-based in the sim).
    for name in ["FLIP_B01", "KICK_J01", "KICK_JB01", "THRO_J01", "CLIMB_B01"] {
        let mut p = Player::new();
        p.special(name);
        for _ in 0..action(name).frames {
            p.step(false, false);
            assert_eq!((p.state, p.world_action()), (State::Special, name));
        }
        p.step(false, false);
        assert_eq!(p.state, State::Idle, "{name}");
    }
}

#[test]
fn remaining_attacks_play_once_then_idle() {
    use super::player::{Player, State};
    // ESTIMATE: triggers are caller-chosen one-shots with no cancels; AT_*_H hug variants belong to the wall item.
    for name in ["AT_CA01", "AT_D01", "AT_DA01", "AT_E01", "SOCCER01", "BLADE01", "AT02", "KICK27", "FUTO27", "CLAW_YARAN01", "CLAW_YARA01", "CLAW_THROWN01"] {
        let mut p = Player::new();
        p.special(name);
        for _ in 0..action(name).frames {
            p.step(false, false);
            assert_eq!((p.state, p.world_action()), (State::Special, name));
        }
        p.step(false, false);
        assert_eq!(p.state, State::Idle, "{name}");
    }
}

#[test]
fn ledge_wall_and_corner_play_once_then_idle() {
    use super::player::{Player, State};
    // ESTIMATE: triggers are caller-chosen one-shots with no cancels; wall/ledge-driven selection comes with the world coupling.
    for name in ["KABE_E01", "KABE_U01", "STOP_A06", "TWO06", "STOP_B06", "ONE06", "BACK06", "IKOU_B06", "SHEATH06", "HANG_R06", "HANG_L06", "CLIMB06", "STOP_ABU05", "IKOU_B05", "SHEATH_BU05"] {
        let mut p = Player::new();
        p.special(name);
        for _ in 0..action(name).frames {
            p.step(false, false);
            assert_eq!((p.state, p.world_action()), (State::Special, name));
        }
        p.step(false, false);
        // a climb ends above flat ground, so it may fall afterwards
        assert!(matches!(p.state, State::Idle | State::Fall), "{name}");
    }
}

#[test]
fn item_and_carry_clips_play_once_then_idle() {
    use super::player::{Player, State};
    // ESTIMATE: triggers are caller-chosen one-shots with no cancels.
    for name in ["PICK01", "PUT01", "OPEN01", "CLOSE01", "MAKI01", "MAKURI01", "MAKURI_A01", "MAKURI_H01", "THROW_BACK01", "DRINK01", "DRINK01_H", "DRINK_K05", "DRINK_S05"] {
        let mut p = Player::new();
        p.special(name);
        for _ in 0..action(name).frames {
            p.step(false, false);
            assert_eq!((p.state, p.world_action()), (State::Special, name));
        }
        p.step(false, false);
        assert_eq!(p.state, State::Idle, "{name}");
    }
}

#[test]
fn remaining_hit_reactions_play_out_then_return_to_idle() {
    use super::{extracted::ACTIONS, player::{Player, State}};
    let names = ["DEKA_HIBA01", "DEKA_HIBBA01", "DEKA_HIBC01", "DEKA_HIBB01", "DEKA_HIBD01", "DEKA_HIUA01", "DEKA_HIUB01", "DEKA_HIC01", "DEKA_HICA01", "DEKA_HICCA01", "DEKA_HICC01", "DEKA_HICB01", "DEKA_HICD01", "NORI_RHI01", "NORI_LHI01", "BURA_HI01", "SUI_HI01", "SHOJI_HI01", "ITEM_HI01", "NAGERARE_A01"];
    for name in names {
        let a = ACTIONS.iter().find(|a| a.name == name).unwrap_or_else(|| panic!("{name} not extracted"));
        let mut p = Player::new();
        p.hit(name);
        for _ in 0..a.frames {
            p.step(false, false);
            assert_eq!((p.state, p.world_action()), (State::Hit, name));
        }
        p.step(false, false);
        assert_eq!(p.state, State::Idle, "{name}");
    }
}

#[test]
fn talk_and_idle_extras_play_once_then_idle() {
    use super::player::{Player, State};
    // ESTIMATE: triggers are caller-chosen one-shots with no cancels.
    for name in ["TOJO_B01", "TALK_A01", "KIME_A01", "KIME_B01", "TALK_UP01", "FURI01", "WAIT01"] {
        let mut p = Player::new();
        p.special(name);
        for _ in 0..action(name).frames {
            p.step(false, false);
            assert_eq!((p.state, p.world_action()), (State::Special, name));
        }
        p.step(false, false);
        assert_eq!(p.state, State::Idle, "{name}");
    }
}

#[test]
fn enemy_notices_chases_attacks_and_dies() {
    use super::enemy::{Enemy, State};
    let mut e = Enemy::new([0.0, 0.0, 5.0]);
    let player = [0.0; 3];
    e.step(player);
    assert_eq!((e.state, e.action()), (State::Idle, "STOP07"));
    e.alert();
    assert_eq!(e.action(), "SURP07");
    for _ in 0..40 { e.step(player); }
    assert_eq!(e.state, State::Chase);
    // facing the player (-z), RUN07 closes the distance until the swing starts
    let mut swung = false;
    for _ in 0..600 {
        e.step(player);
        swung |= e.state == State::Attack;
        if e.attack_active() { break; }
    }
    assert!(swung && e.attack_active());
    assert!(e.pos[2] < 5.0 && e.pos[2] > 0.5, "{:?}", e.pos);
    e.hit("HI_BS07", 30);
    assert_eq!((e.state, e.hp, e.action()), (State::Hit, 70, "HI_BS07"));
    e.hit("HI_BS07", 100);
    assert_eq!((e.state, e.hp, e.action()), (State::Dead, 0, "DIE_A07"));
    for _ in 0..400 { e.step(player); }
    assert_eq!(e.state, State::Dead);
}

#[test]
fn enemy_guard_blocks_damage_and_patrol_loops() {
    use super::enemy::{Enemy, State};
    let mut e = Enemy::new([0.0; 3]);
    e.guard_held = true;
    e.step([0.0, 0.0, 9.0]);
    assert_eq!(e.state, State::Guard);
    e.hit("HI_BS07", 50);
    assert_eq!(e.hp, super::data::ENEMY_HP);
    e.guard_held = false;
    e.step([0.0, 0.0, 9.0]);
    assert_eq!(e.state, State::Idle);
    e.patrol();
    for _ in 0..200 { e.step([0.0, 0.0, 9.0]); }
    assert_eq!((e.state, e.action()), (State::Patrol, "WALK_A07"));
}

#[test]
fn hits_land_both_ways_and_guard_blocks() {
    use super::{data, combat::Fight, enemy::{Enemy, State as ES}, player::{Player, State as PS}};
    // player swings at an enemy standing in front
    let mut f = Fight::new(Player::new(), Enemy::new([0.0, 0.0, 1.0]));
    f.step(false, true);
    for _ in 0..40 { f.step(false, false); }
    assert!(f.enemy.hp < data::ENEMY_HP, "hp {}", f.enemy.hp);
    // enemy behind the player is not hit
    let mut f = Fight::new(Player::new(), Enemy::new([0.0, 0.0, -1.0]));
    f.step(false, true);
    for _ in 0..40 { f.step(false, false); }
    assert_eq!(f.enemy.hp, data::ENEMY_HP);
    // enemy swing hurts an unguarded player
    let mut f = Fight::new(Player::new(), Enemy::new([0.0, 0.0, 1.0]));
    f.enemy.alert();
    for _ in 0..300 { f.step(false, false); }
    assert!(f.player_hp < data::PLAYER_HP);
    assert_ne!(f.enemy.state, ES::Dead);
    // a guarding player takes nothing
    let mut f = Fight::new(Player::new(), Enemy::new([0.0, 0.0, 1.0]));
    f.player.guard_held = true;
    f.enemy.alert();
    for _ in 0..300 { f.step(false, false); }
    assert_eq!((f.player_hp, f.player.state == PS::Guard), (data::PLAYER_HP, true));
}

#[test]
fn lock_on_selects_switches_releases_and_strafes() {
    use super::{lockon::LockOn, player::Player};
    let ts = [[0.0, 0.0, 5.0], [4.0, 0.0, 4.0], [-4.0, 0.0, 4.0], [0.0, 0.0, -5.0]];
    let mut l = LockOn::default();
    l.select([0.0; 3], 0.0, &ts);
    assert_eq!(l.target, Some(0)); // nearest the facing; the one behind is outside the cone
    l.switch([0.0; 3], &ts, true);
    assert_eq!(l.target, Some(1)); // clockwise (+x) neighbour
    l.switch([0.0; 3], &ts, false);
    l.switch([0.0; 3], &ts, false);
    assert_eq!(l.target, Some(2));
    l.release();
    assert_eq!(l.target, None);
    l.select([0.0; 3], 0.0, &[[0.0, 0.0, 50.0]]);
    assert_eq!(l.target, None); // out of range

    // strafe right around a target ahead: faces it, plays WALK_R02, moves +x
    let mut p = Player::new();
    l.select(p.pos, p.heading, &ts[..1]);
    for _ in 0..60 {
        let fwd = l.drive(&mut p, &ts[..1], [1.0, 0.0]);
        p.step(fwd, false);
    }
    assert_eq!(p.world_action(), "WALK_R02");
    assert!(p.pos[0] < -0.5, "x {}", p.pos[0]); // WALK_R02 root is -x at heading 0 (formats.md)
    let want = (0.0 - p.pos[0]).atan2(5.0 - p.pos[2]);
    assert!((p.heading - want).abs() < 0.3, "heading {} want {}", p.heading, want);
    // target vanishes: lock drops
    assert!(!l.drive(&mut p, &[], [1.0, 0.0]));
    assert_eq!(l.target, None);
}

#[test]
fn stealth_fills_from_sight_and_noise_and_alerts() {
    use super::{enemy::{Enemy, State}, stealth::{Alert, Stealth}};
    let run = |e: &mut Enemy, s: &mut Stealth, p: [f32; 3], act: &str, n: u32| (0..n).for_each(|_| s.step(e, p, act));
    // in front (heading 0 faces +z), standing: seen, alerts within a few seconds
    let (mut e, mut s) = (Enemy::new([0.0; 3]), Stealth::default());
    run(&mut e, &mut s, [0.0, 0.0, 5.0], "STOP01", 30);
    assert!(s.seen && s.meter > 0.0 && s.level() != Alert::Alerted);
    run(&mut e, &mut s, [0.0, 0.0, 5.0], "STOP01", 300);
    assert_eq!(s.level(), Alert::Alerted);
    assert_eq!(e.state, State::Alert);
    // behind, standing: unseen, silent -> stays at zero
    let (mut e, mut s) = (Enemy::new([0.0; 3]), Stealth::default());
    run(&mut e, &mut s, [0.0, 0.0, -2.0], "STOP01", 600);
    assert!(!s.seen && s.meter == 0.0);
    // behind, running close: heard, meter rises; walking close is quiet enough only inside 3 m, beyond it decays
    run(&mut e, &mut s, [0.0, 0.0, -2.0], "RUN01", 60);
    assert!(s.meter > 0.3, "{}", s.meter);
    run(&mut e, &mut s, [0.0, 0.0, -6.0], "WALK_F02", 600);
    assert_eq!(s.meter, 0.0);
}

#[test]
fn execution_cue_needs_the_right_position_and_enemy_state() {
    use super::{enemy::Enemy, execution::{available, Kind}, player::Player};
    let mut p = Player::new();
    let mut e = Enemy::new([0.0; 3]); // faces +z, Idle
    p.pos = [0.0, 0.0, -1.0];
    assert_eq!(available(&p, &e), Some(Kind::Back));
    p.pos = [0.0, 0.0, 1.0]; // in front of an unaware soldier: no kill
    assert_eq!(available(&p, &e), None);
    p.pos = [0.0, 2.5, 0.5];
    assert_eq!(available(&p, &e), Some(Kind::Above));
    p.pos = [0.0, 0.0, -3.0]; // out of reach
    assert_eq!(available(&p, &e), None);
    // staggered and low: front finisher
    e.hit("HI_BS07", 80);
    p.pos = [0.0, 0.0, 1.0];
    assert_eq!(available(&p, &e), Some(Kind::Front));
    e.hit("HI_BS07", 100);
    assert_eq!(available(&p, &e), None); // dead
}

#[test]
fn every_execution_plays_paired_and_kills() {
    use super::{combat::Fight, enemy::{Enemy, State}, execution::Kind, player::{Player, State as PS}};
    for kind in [Kind::Back, Kind::Above, Kind::Front] {
        let mut p = Player::new();
        let mut e = Enemy::new([0.0; 3]); // faces +z
        match kind {
            Kind::Back => p.pos = [0.0, 0.0, -1.2],
            Kind::Above => p.pos = [0.0, 2.5, 0.5],
            Kind::Front => {
                e.hit("HI_BS07", 80);
                p.pos = [0.0, 0.0, 1.2];
            }
        }
        let mut f = Fight::new(p, e);
        assert!(f.execute(), "{kind:?}");
        let (pc, ec) = kind.clips();
        assert_eq!((f.player.world_action(), f.enemy.action()), (pc, ec));
        assert_eq!((f.enemy.state, f.enemy.hp), (State::Dead, 0));
        let d = (f.enemy.pos[0] - f.player.pos[0]).hypot(f.enemy.pos[2] - f.player.pos[2]);
        assert!((d - super::data::EXEC_DIST).abs() < 1e-4);
        for _ in 0..400 {
            f.step(false, false);
        }
        assert_eq!(f.player.state, PS::Idle);
        assert_eq!(f.enemy.state, State::Dead);
        assert!(f.enemy.frame() >= 30);
    }
    // nothing to execute
    let mut f = Fight::new(Player::new(), Enemy::new([0.0, 0.0, 9.0]));
    assert!(!f.execute());
}

#[test]
fn sandbox_terrain_has_roof_slope_stairs_corridor_and_crates() {
    use super::terrain::Terrain;
    let t = Terrain::sandbox();
    assert_eq!(t.ground_height([10.0, 3.5, 10.0]), 3.5);
    assert!((t.ground_height([0.0, 1.0, -7.0]) - 0.5).abs() < 1e-5);
    assert!(t.ground_height([9.0, 3.0, -4.1]) > 2.3);
    assert_eq!(t.ground_height([-19.0, 0.0, -10.0]), 0.0); // corridor floor
    assert_eq!(t.ground_height([3.1, 2.0, 3.5]), 2.0); // stacked crates
    assert!(t.ledge([10.0, 0.0, 4.0], [0.0, 1.0], 4.0).is_some()); // roof edge in reach of a high jump
}

#[test]
fn pose_parses_and_poses_a_two_bone_chain() {
    use super::pose::*;
    // Skeleton: root id 23, hips (0) child of 23, id 7 child of hips offset 1 m along +z.
    let mut sk = Vec::new();
    sk.extend(28u32.to_le_bytes());
    for n in 0..28i32 {
        let (p, o): (i32, [f32; 3]) = match n {
            23 => (-1, [0.0; 3]),
            0 => (23, [0.0; 3]),
            7 => (0, [0.0, 0.0, 1.0]),
            _ => (23, [0.0; 3]),
        };
        sk.extend(p.to_le_bytes());
        for x in o {
            sk.extend(x.to_le_bytes());
        }
    }
    let skeleton = Skeleton::parse(&sk).unwrap();
    assert_eq!(skeleton.parent[7], Some(0));
    assert_eq!(skeleton.parent[23], None);
    // One clip, one frame: hips at y 1, node 0 identity, node 0 rotated 90 degrees about x (z -> -y).
    let mut row = vec![0.0f32; 87];
    row[1] = 1.0;
    for n in 0..21 {
        row[6 + 4 * n] = 1.0;
    }
    let s = std::f32::consts::FRAC_1_SQRT_2;
    (row[3], row[6]) = (s, s);
    let mut clips = Vec::new();
    clips.extend(1u32.to_le_bytes());
    clips.extend(b"T\0".iter().chain([0u8; 22].iter()));
    clips.extend(1u32.to_le_bytes());
    for x in &row {
        clips.extend(x.to_le_bytes());
    }
    let clips = Clips::parse(&clips).unwrap();
    assert!(clips.row("NOPE", 0).is_none());
    let j = skeleton.joints(clips.row("T", 9).unwrap()); // frame past the end holds the last one
    assert_eq!(j[0], [0.0, 1.0, 0.0]);
    assert!((j[7][1] - 0.0).abs() < 1e-5 && j[7][2].abs() < 1e-5, "{:?}", j[7]);
}

#[test]
fn fight_pushes_overlapping_bodies_apart() {
    use super::{data, combat::Fight, enemy::Enemy, player::Player};
    let mut f = Fight::new(Player::new(), Enemy::new([0.0, 0.0, 0.1]));
    f.step_enemy();
    let (p, e) = (f.player.pos, f.enemy.pos);
    assert!((e[0] - p[0]).hypot(e[2] - p[2]) >= 2.0 * data::BODY_RADIUS - 1e-3);
}

#[test]
fn request_gate_refuses_outside_free_state_and_cancel_window() {
    use super::player::{Player, State};
    let mut p = Player::new();
    p.step(false, true); // AT_A01 from idle
    assert_eq!(p.state, State::Attack);
    assert!(!p.request(State::Dodge, "FRONT01"), "no cancel window for FRONT01 at frame 0 of AT_A01");
    assert_eq!(p.world_action(), "AT_A01");
    let mut q = Player::new();
    q.jump();
    q.step(false, false);
    q.step(false, true);
    assert_eq!(q.state, State::Jump, "attack is refused mid-jump");
}

#[test]
fn dash_runs_faster_than_run() {
    use super::{input::Input, player::Player, terrain::Terrain};
    let dist = |dash: bool| {
        let (mut p, t, mut i) = (Player::new(), Terrain::default(), Input::default());
        i.stick = [0.0, 1.0];
        i.dash = dash;
        for _ in 0..120 { i.apply(&mut p, &t, 0.0); }
        (p.world_action(), p.pos[2])
    };
    let (d, run) = (dist(true), dist(false));
    assert_eq!(d.0, "RUN_D05");
    assert!(d.1 > run.1 * 1.3, "{d:?} vs {run:?}");
}

#[test]
fn crouch_holds_walks_and_is_quieter() {
    use super::{enemy::Enemy, player::{Player, State}, stealth::Stealth};
    let mut p = Player::new();
    p.crouch_held = true;
    p.step(false, false);
    assert_eq!((p.state, p.world_action()), (State::Crouch, "STOP_S05"));
    for _ in 0..90 { p.step(true, false); }
    assert_eq!(p.world_action(), "WALK_F05");
    assert!(p.pos[2] > 1.0, "crouch walk moves: {}", p.pos[2]);
    p.crouch_held = false;
    p.step(false, false);
    assert_ne!(p.state, State::Crouch, "release stands up");
    // 2 m behind the soldier: a crouch walk is heard only inside HEAR_CROUCH, a plain walk is heard
    let heard = |act: &str| {
        let (mut e, mut s) = (Enemy::new([0.0; 3]), Stealth::default());
        (0..60).for_each(|_| s.step(&mut e, [0.0, 0.0, -2.0], act));
        s.meter
    };
    assert!(heard("WALK_F05") == 0.0 && heard("WALK_F02") > 0.0);
}

#[test]
fn wall_hug_enters_slides_peeks_and_leaves() {
    use super::player::{Player, State};
    use super::terrain::Terrain;
    let t = Terrain::sandbox();
    let mut p = Player::new();
    p.pos = [-8.0, 0.0, 6.0];
    p.crouch_held = true;
    // walk crouched +z into the tall wall (z 8..8.5)
    for _ in 0..200 { p.target_heading = 0.0; p.step_on(&t, true, false); }
    assert_eq!(p.state, State::Hug);
    assert!((p.heading - (-std::f32::consts::PI)).abs() < 1e-3 || (p.heading - std::f32::consts::PI).abs() < 1e-3, "back to wall, faces -z");
    for _ in 0..70 { p.step_on(&t, false, false); }
    assert_eq!(p.world_action(), "STOP_H05");
    // facing -z, screen-left of the player is +x: stick toward +x
    let x0 = p.pos[0];
    for _ in 0..60 { p.target_heading = std::f32::consts::FRAC_PI_2; p.step_on(&t, true, false); }
    assert!(p.pos[0] > x0 + 0.5, "side-steps along the wall: {}", p.pos[0]);
    assert_eq!(p.world_action(), "HUG_RH05");
    let expected = super::World::new("HUG_RH05");
    let cycle = expected.root_at(expected.action.frames)[0];
    assert!((p.pos[0] - x0 + cycle + expected.root_at(8)[0]).abs() < 1e-4, "uses extracted lateral motion over a loop");
    p.target_heading = -std::f32::consts::FRAC_PI_2;
    p.step_on(&t, true, false);
    assert_eq!(p.world_action(), "HUG_LH05", "other direction");
    p.target_heading = std::f32::consts::FRAC_PI_2;
    for _ in 0..400 { p.step_on(&t, true, false); }
    assert!(p.pos[0] < -3.5, "stops at the wall end (x=-4): {}", p.pos[0]);
    assert_eq!(p.world_action(), "STOPLR05");
    p.target_heading = p.heading; // stick away from the wall
    p.step_on(&t, true, false);
    assert_eq!(p.state, State::Idle);
    for _ in 0..90 { p.step_on(&t, false, false); }
    assert_eq!(p.state, State::Crouch, "crouch is still held after leaving");
}

#[test]
fn hug_peeks_both_corners_and_returns_before_moving() {
    use super::player::{Player, State};
    use super::terrain::{Terrain, Collider};
    let terrain = Terrain { colliders: vec![Collider::new([-1.0, 0.0, 2.0], [1.0, 4.0, 3.0], false)] };
    for direction in [-1.0_f32, 1.0] {
        let mut p = Player::new();
        p.crouch_held = true;
        for _ in 0..180 { p.step_on(&terrain, true, false); }
        assert_eq!(p.state, State::Hug);
        p.target_heading = direction * std::f32::consts::FRAC_PI_2;
        let (look, hold, back, walk) = if direction > 0.0 {
            ("LOOK_RH05", "STOPLR05", "BACK_RH05", "HUG_LH05")
        } else { ("LOOK_LH05", "STOPLL05", "BACK_LH05", "HUG_RH05") };
        let mut saw_look = false;
        for _ in 0..220 {
            p.step_on(&terrain, true, false);
            saw_look |= p.world_action() == look;
        }
        assert!(saw_look);
        assert_eq!(p.world_action(), hold);
        let corner = p.pos;
        for _ in 0..100 { p.step_on(&terrain, true, false); }
        assert_eq!(p.pos, corner, "cannot walk beyond the corner");
        p.target_heading = -p.target_heading;
        p.step_on(&terrain, true, false);
        assert_eq!(p.world_action(), back);
        assert_eq!(p.pos, corner, "return from peek before translating");
        for _ in 0..45 { p.step_on(&terrain, true, false); }
        assert_eq!(p.world_action(), walk);
        assert!(p.pos[0].abs() < corner[0].abs());
        p.crouch_held = false;
        p.step_on(&terrain, false, false);
        assert_eq!(p.state, State::Idle);
    }
}

#[test]
fn low_blocks_do_not_allow_a_hug() {
    use super::player::{Player, State};
    use super::terrain::Terrain;
    let t = Terrain::sandbox();
    let mut p = Player::new();
    p.pos = [2.5, 0.0, 1.0];
    p.crouch_held = true;
    for _ in 0..120 { p.target_heading = 0.0; p.step_on(&t, true, false); }
    assert_ne!(p.state, State::Hug, "crate is 1 m tall");
}

#[test]
fn hug_pad_sequence_survives_camera_pull_in() {
    use super::{camera::Camera, input::Input, player::{Player, State}, terrain::Terrain};
    let mut p = Player::new();
    p.pos = [-8.0, 0.0, 6.0];
    let mut input = Input { crouch: true, ..Default::default() };
    let mut camera = Camera { yaw: 1.0_f32.to_radians(), ..Default::default() };
    let terrain = Terrain::sandbox();
    let mut clips = std::collections::HashSet::new();
    // Same pad sequence as tools/scripts/hug.txt, with a 60 Hz render clock.
    for tick in 0..840 {
        match tick {
            1 => input.stick = [0.0, 1.0],
            100 | 600 | 760 => input.stick = [0.0; 2],
            160 => input.stick = [-0.5, 0.8660254],
            650 => input.stick = [0.5, -0.8660254],
            800 => input.crouch = false,
            _ => {}
        }
        input.apply(&mut p, &terrain, camera.yaw);
        camera.follow_hug((p.state == State::Hug).then_some(p.heading), super::DT);
        clips.insert(p.world_action());
    }
    for name in ["HUG_RH05", "HUG_LH05", "LOOK_RH05", "STOPLR05", "BACK_RH05"] {
        assert!(clips.contains(name), "pad sequence must reach {name}; saw {clips:?}, yaw {}, position {:?}", camera.yaw, p.pos);
    }
    assert_eq!(p.state, State::Idle);
}

#[test]
fn running_into_a_tall_wall_with_jump_runs_up_then_kicks_off() {
    use crate::sim::{player::{Player, State}, terrain::Terrain};
    let t = Terrain::sandbox();
    let mut p = Player::new();
    p.pos = [-7.0, 0.0, 4.0];
    for _ in 0..120 { p.step_on(&t, true, false); }
    assert!(p.wall_normal.is_some(), "ran into the wall");
    p.jump();
    assert_eq!(p.world_action(), "KABE_U01");
    for _ in 0..20 { p.step_on(&t, true, false); }
    assert!(p.pos[1] > 1.0, "rising: {}", p.pos[1]);
    p.jump(); // kick off
    assert_eq!(p.world_action(), "TRI_J01");
    let z = p.pos[2];
    for _ in 0..200 { p.step_on(&t, false, false); }
    assert!(p.pos[2] < z - 0.3, "pushed away from the wall");
    assert!(p.pos[1] < 0.01 && !matches!(p.state, State::WallKick | State::Fall), "landed: {:?}", p.state);
}

#[test]
fn wall_run_reaching_the_top_kicks_off_with_kabe_e() {
    use crate::sim::{player::Player, terrain::Terrain};
    let t = Terrain::sandbox();
    let mut p = Player::new();
    p.pos = [-7.0, 0.0, 4.0];
    for _ in 0..120 { p.step_on(&t, true, false); }
    p.jump();
    for _ in 0..37 { p.step_on(&t, true, false); }
    assert_eq!(p.world_action(), "KABE_E01");
}

#[test]
fn jumping_at_a_ledge_hangs_shimmies_and_climbs_or_drops() {
    use crate::sim::{player::{Player, State}, terrain::Terrain};
    let t = Terrain::sandbox();
    let mut p = Player::new();
    p.pos = [10.0, 0.0, 3.0]; // rooftop face at z 6, 3.5 m high, facing +z
    for _ in 0..120 { p.step_on(&t, true, false); }
    p.jump(); // wall run up the roof's face; the hands catch the edge on the way
    let mut seen = std::collections::HashSet::new();
    for _ in 0..60 { p.step_on(&t, true, false); seen.insert(p.world_action()); if p.state == State::Hang { break; } }
    assert_eq!(p.state, State::Hang, "caught the ledge: {seen:?}");
    assert!(p.pos[1] > 0.5 && p.pos[1] < 3.5, "hang height {}", p.pos[1]);
    for _ in 0..120 { p.step_on(&t, false, false); }
    assert_eq!(p.world_action(), "STOP_A06");
    // shimmy to the right (-x) until the crate ends
    p.target_heading = p.heading - std::f32::consts::FRAC_PI_2;
    let x = p.pos[0];
    for _ in 0..400 { p.step_on(&t, true, false); }
    assert!(p.pos[0] < x && p.pos[0] > 6.0, "shimmied within the ledge: {} -> {}", x, p.pos[0]);
    assert_eq!(p.state, State::Hang);
    p.jump();
    assert_eq!(p.world_action(), "CLIMB06");
    for _ in 0..80 { p.step_on(&t, false, false); }
    assert_eq!(p.state, State::Idle);
    assert!((p.pos[1] - 3.5).abs() < 0.01, "on top: {}", p.pos[1]);
    // drop
    let mut q = Player::new();
    q.pos = [10.0, 0.0, 3.0];
    for _ in 0..120 { q.step_on(&t, true, false); }
    q.jump();
    for _ in 0..60 { q.step_on(&t, true, false); if q.state == State::Hang { break; } }
    q.crouch_held = true;
    for _ in 0..200 { q.step_on(&t, false, false); }
    assert!(q.pos[1] < 0.01 && q.state != State::Hang, "dropped: {:?}", q.state);
}

#[test]
fn jumping_at_a_crate_stack_vaults_over_it_with_climb_b() {
    use crate::sim::{player::{Player, State}, terrain::Terrain};
    let t = Terrain::sandbox();
    let mut p = Player::new();
    p.pos = [3.1, 0.0, 2.4]; // faces the 2 m crate stack (z 3), 0.6 m away
    p.step_on(&t, false, false);
    p.jump();
    assert_eq!(p.world_action(), "CLIMB_B01");
    for _ in 0..140 { p.step_on(&t, false, false); if p.state == State::Idle { break; } }
    assert_eq!(p.state, State::Idle);
    assert!(p.pos[2] > 4.0 && p.pos[1] < 0.01, "landed beyond: {:?}", p.pos);
    // a 1 m crate is jumped, not vaulted
    let mut q = Player::new();
    q.pos = [2.5, 0.0, 2.4];
    q.step_on(&t, false, false);
    q.jump();
    assert_eq!(q.world_action(), "JUMP01");
}

#[test]
fn air_attack_kicks_back_flick_flips_and_both_land() {
    use crate::sim::{player::{Player, State}, terrain::Terrain};
    let t = Terrain::default();
    let air = |setup: &dyn Fn(&mut Player)| {
        let mut p = Player::new();
        p.jump();
        for _ in 0..12 { p.step_on(&t, false, false); }
        setup(&mut p);
        p
    };
    let mut p = air(&|p| p.step_on(&t, false, true));
    assert_eq!((p.state, p.world_action()), (State::AirAct, "KICK_J01"));
    for _ in 0..200 { p.step_on(&t, false, false); if p.state == State::Idle { break; } }
    assert_eq!(p.state, State::Idle, "lands after the kick");
    let mut b = air(&|p| p.target_heading = std::f32::consts::PI);
    b.step_on(&t, true, false);
    assert_eq!(b.world_action(), "FLIP_B01");
    let mut g = Player::new();
    g.step_on(&t, false, true);
    assert_eq!(g.state, State::Attack, "on the ground Square is still a slash");
}
