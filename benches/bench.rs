#[macro_use]
extern crate criterion;
extern crate rayon;
extern crate rayon_adaptive;

// use num::iter::Range;
use rayon_adaptive::prelude::*;
use std::ops::Range;

use criterion::Criterion;

pub struct RayonFilter<'a, T: Sync + Send> {
    data: Range<T>,
    predicate: &'a (dyn Fn(&T) -> bool + Send + Sync),
}
use adaptive_algorithms::adaptive_bench::{TestConfig, Tester};
use adaptive_algorithms::Benchable;
impl<'a> Benchable<'a, usize> for RayonFilter<'a, usize> {
    fn start(&mut self) -> Option<usize> {
        use rayon::prelude::*;
        let data = self.data.clone();
        let sum = data
            .into_par_iter()
            .filter(self.predicate)
            .reduce(|| 0, |a, b| a + b);
        Some(sum)
    }
    fn name(&self) -> &'static str {
        "Rayon Filter"
    }
}

pub struct IteratorFilter<'a, T: Sync + Send> {
    data: Range<T>,
    predicate: &'a (dyn Fn(&T) -> bool + Send + Sync),
}
impl<'a> Benchable<'a, usize> for IteratorFilter<'a, usize> {
    fn start(&mut self) -> Option<usize> {
        let data = self.data.clone();
        let sum = data
            .into_iter()
            .filter(self.predicate)
            .fold(Default::default(), |a, b| a + b);
        Some(sum)
    }
    fn name(&self) -> &'static str {
        "Regalur Filter"
    }
}
pub struct AdaptiveFilter<'a, T: Sync + Send> {
    data: Range<T>,
    predicate: &'a (dyn Fn(&T) -> bool + Send + Sync),
}
impl<'a> Benchable<'a, usize> for AdaptiveFilter<'a, usize> {
    fn start(&mut self) -> Option<usize> {
        let data = self.data.clone();
        let sum = data
            .into_adapt_iter()
            .filter(self.predicate)
            // .cloned()
            .fold(Default::default, |a, b| a + b)
            .reduce(|a, b| a + b);

        Some(sum)
    }
    fn name(&self) -> &'static str {
        "rayon-adaptive filter"
    }
}

pub struct TryFoldFilter<'a, T: Sync + Send> {
    data: Range<usize>,
    predicate: &'a (dyn Fn(&T) -> bool + Send + Sync),
}
impl<'a> Benchable<'a, usize> for TryFoldFilter<'a, usize> {
    fn start(&mut self) -> Option<usize> {
        use rayon_try_fold::prelude::*;
        let data = self.data.clone();
        let result = data
            .into_par_iter()
            .adaptive()
            .filter(self.predicate)
            .rayon(2)
            .reduce(|| 0, |a, b| a + b);
        Some(result)
    }
    fn name(&self) -> &'static str {
        "rayon_try_fold"
    }
}
pub struct MyNewAdaptive<'a, T: Sync + Send> {
    data: Range<usize>,
    predicate: &'a (dyn Fn(&T) -> bool + Send + Sync),
}
impl<'a> Benchable<'a, usize> for MyNewAdaptive<'a, usize> {
    fn start(&mut self) -> Option<usize> {
        use adaptive_iterator::mk_adaptive;
        use rayon_try_fold::prelude::*;
        let data = self.data.clone();
        let iter = data.into_par_iter().filter(self.predicate).adaptive();
        mk_adaptive(iter).reduce(|| 0, |a, b| a + b);
        None
    }
    fn name(&self) -> &'static str {
        "New with adaptive fold"
    }
}
pub struct LoopFilter<'a, T: Sync + Send> {
    data: Range<T>,
    predicate: &'a (dyn Fn(&T) -> bool + Send + Sync),
}
impl<'a> Benchable<'a, usize> for LoopFilter<'a, usize> {
    fn start(&mut self) -> Option<usize> {
        let mut sum: usize = 0;
        let data = self.data.clone();
        for i in data {
            if (self.predicate)(&i) {
                sum = sum + i.clone();
            }
        }
        Some(sum)
    }
    fn name(&self) -> &'static str {
        "For Loop"
    }
}
// can also be used
fn is_prime(n: usize) -> bool {
    for a in 2..(n as f64).sqrt() as usize {
        if n % a == 0 {
            return false;
        }
    }
    true
}

type Predicate = (dyn Fn(&usize) -> bool + Sync + Send);
fn bench(c: &mut Criterion) {
    let data: std::ops::Range<usize> = 0..50_000_000;
    let len = data.end;
    let mut group = c.benchmark_group("Filter");
    let predicate: &Predicate = criterion::black_box(&|&x| x % 2 == 0);
    // let predicate: &Predicate = &|&&x| is_prime(x);
    group.warm_up_time(std::time::Duration::new(1, 0));
    group.measurement_time(std::time::Duration::new(3, 0));
    group.sample_size(10);

    let cpus: Vec<usize> = vec![1, 2, 3, 4, 8, 16, 24, 32]
        .iter()
        .filter(|&&i| i <= num_cpus::get())
        .cloned()
        .collect();

    let mut test: Vec<TestConfig<usize>> = vec![];
    let predicate = &predicate;
    for i in &cpus {
        for s in vec![0, 6, 8] {
            let t = TestConfig::new(
                len,
                *i,
                Some(s),
                MyNewAdaptive {
                    data: data.clone(),
                    predicate,
                },
            );
            test.push(t);
        }
        let f = AdaptiveFilter {
            data: data.clone(),
            predicate,
        };
        test.push(TestConfig::new(len, *i, None, f));
        let f = TryFoldFilter {
            data: data.clone(),
            predicate,
        };
        test.push(TestConfig::new(len, *i, None, f));

        let f = RayonFilter {
            data: data.clone(),
            predicate,
        };
        test.push(TestConfig::new(len, *i, None, f));
    }
    let r = LoopFilter {
        data: data.clone(),
        predicate,
    };

    let t = TestConfig::new(len, 1, None, r);
    test.push(t);
    let r = IteratorFilter {
        data: data.clone(),
        predicate,
    };

    let t = TestConfig::new(len, 1, None, r);
    test.push(t);

    // r.start();
    let mut t = Tester::new(test, group, None);
    t.run();

    // group.finish();
}
criterion_group!(benches, bench);
criterion_main!(benches);
