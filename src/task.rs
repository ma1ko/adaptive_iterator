use adaptive_algorithms::steal;

// if you can't use None because of typing errors, use Nothing
// #[derive(Copy, Clone)]

pub trait Task: Send + Sync {
    // run self *and* me, or return false if you can't
    // fn run_(&mut self) {
    //     self.run(NOTHING)
    // }
    fn run(&mut self) {
        while !self.is_finished() {
            let steal_counter = steal::get_my_steal_count();
            if steal_counter != 0 && self.can_split() {
                self.split_run(steal_counter);
                continue;
            }
            self.step();
        }
    }
    // fn run_recursive(&mut self) {
    //     let steal_counter = steal::get_my_steal_count();
    //     if steal_counter != 0 && self.can_split() {
    //         let mut other = self.split();
    //         self.split_run(steal_counter);
    //         self.fuse(other);
    //     }
    //     // self.run_(other);
    //     self.run_();
    // }
    fn step(&mut self);
    fn split_run_(&mut self) {
        self.split_run(1)
    }
    fn split_run(&mut self, steal_counter: usize) {
        // // run the parent task

        // // let mut other: Self = self.split();
        let runner = |left: &mut Self, mut right: &mut Self| {
            if steal_counter < 2 {
                rayon::join(
                    || {
                        steal::reset_my_steal_count();
                        left.run()
                    },
                    || right.run(),
                );
            // self.fuse(other);
            } else {
                rayon::join(
                    || left.split_run(steal_counter / 2),
                    || right.split_run(steal_counter / 2),
                );
                left.fuse(&mut right);
            }
        };
        self.split(runner);
    }
    // fn check_(&mut self){
    //     self.check();
    // }
    fn check(&mut self) {
        let steal_counter = steal::get_my_steal_count();
        if steal_counter != 0 && self.can_split() {
            self.split_run(steal_counter);
        }
    }
    fn can_split(&self) -> bool;
    fn is_finished(&self) -> bool;
    fn split(&mut self, runner: impl Fn(&mut Self, &mut Self));
    fn fuse(&mut self, _other: &mut Self) { }
}
