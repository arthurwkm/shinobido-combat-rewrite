use shinobido_combat_rewrite::sim::{extracted::ACTIONS, World};

fn main() {
    for a in ACTIONS {
        let mut w = World::new(a.name);
        let mut n = 0;
        while !w.done() {
            w.step();
            n += 1;
        }
        println!("{} {} {} {} {}", a.name, n, w.pos[0], w.pos[1], w.pos[2]);
    }
}
