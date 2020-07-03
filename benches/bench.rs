#[macro_use]
extern crate criterion;
extern crate rayon;
extern crate rayon_adaptive;

use adaptive_iterator::filter;
use num::iter::Range;
use num::traits::ToPrimitive;
use num::Integer;
use num::Num;
use rayon_adaptive::prelude::*;

use criterion::Criterion;

pub struct RayonFilter<'a, T: Sync + Send> {
    data: Range<T>,
    predicate: &'a (dyn Fn(&T) -> bool + Send + Sync),
}
use adaptive_algorithms::adaptive_bench::{TestConfig, Tester};
use adaptive_algorithms::Benchable;
impl<'a, T: Sync + Send> Benchable<'a, T> for RayonFilter<'a, T>
where
    T: Integer + Default + Clone + ToPrimitive,
{
    fn start(&mut self) -> Option<T> {
        use rayon::prelude::*;
        let sum = self
            .data
            .into_iter()
            .par_iter()
            .filter(self.predicate)
            // .cloned()
            .reduce(|| T::default(), |a: T, b: T| a + b);
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
impl<'a, T: Sync + Send> Benchable<'a, T> for IteratorFilter<'a, T>
where
    T: Integer + Default + Clone + ToPrimitive,
{
    fn start(&mut self) -> Option<T> {
        let sum = self
            .data
            .into_iter()
            .filter(self.predicate)
            // .cloned()
            .fold(Default::default(), |a, b| a + b);
        Some(sum)
    }
    fn name(&self) -> &'static str {
        "Regalur Filter"
    }
}
pub struct AdaptiveFilter<'a, T: Sync + Send> {
    data: &'a [T],
    predicate: &'a (dyn Fn(&&T) -> bool + Send + Sync),
}
impl<'a, T: Sync + Send> Benchable<'a, T> for AdaptiveFilter<'a, T>
where
    T: Num + Default + Clone, /* why? */
{
    fn start(&mut self) -> Option<T> {
        let sum = self
            .data
            .into_adapt_iter()
            .filter(self.predicate)
            .cloned()
            .fold(Default::default, |a, b| a + b)
            .reduce(|a, b| a + b);

        Some(sum)
    }
    fn name(&self) -> &'static str {
        "rayon-adaptive filter"
    }
}
// pub struct MyAdaptiveFilter<'a, T: Sync + Send, P: Send + Sync + 'a>
// where
//     P: Fn(&&T) -> bool,
// {
//     data: Range<T>,
//     predicate: &'a P,
// }
// impl<'a, T: Sync + Send, P: Send + Sync> Benchable<'a, T> for MyAdaptiveFilter<'a, T, P>
// where
//     P: Fn(&&T) -> bool,
//     T: Num + Clone + Default,
// {
//     fn start(&mut self) -> Option<T> {
//         let res: Vec<&T> = filter(self.data, self.predicate);
//         let sum = res.iter().cloned().fold(T::default(), |a, b| a + b.clone());
//         Some(sum)
//     }
//     fn name(&self) -> &'static str {
//         "My Adaptive Version"
//     }
// }
pub struct TryFoldFilter<'a, T: Sync + Send, P: Send + Sync + 'a>
where
    P: Fn(&&T) -> bool,
{
    data: &'a [T],
    predicate: &'a P,
}
impl<'a, T: Sync + Send, P: Send + Sync> Benchable<'a, T> for TryFoldFilter<'a, T, P>
where
    P: Fn(&&T) -> bool,
    T: Num + Default + Clone,
{
    fn start(&mut self) -> Option<T> {
        use rayon_try_fold::prelude::*;
        let result = self
            .data
            .into_par_iter()
            .adaptive()
            .filter(self.predicate)
            .map(|x| x.clone())
            .rayon(2)
            .reduce(|| T::default(), |a, b| a + b);
        Some(result)
    }
    fn name(&self) -> &'static str {
        "rayon_try_fold"
    }
}
pub struct MyNewAdaptive<'a, T: Sync + Send, P: Send + Sync + 'a>
where
    P: Fn(&&T) -> bool,
{
    data: &'a [T],
    predicate: &'a P,
}
impl<'a, T: Sync + Send, P: Send + Sync> Benchable<'a, T> for MyNewAdaptive<'a, T, P>
where
    P: Fn(&&T) -> bool,
    T: Num + Default + Clone,
{
    fn start(&mut self) -> Option<T> {
        use adaptive_iterator::adaptive::mk_adaptive;
        use rayon_try_fold::prelude::*;
        let iter = self
            .data
            .into_par_iter()
            .filter(self.predicate)
            .adaptive()
            .map(|x| x.clone());
        mk_adaptive(iter).reduce(|| T::default(), |a, b| a + b);
        None
    }
    fn name(&self) -> &'static str {
        "New with adaptive fold"
    }
}
pub struct LoopFilter<'a, T: Sync + Send, P: Send + Sync + 'a>
where
    P: Fn(&&T) -> bool,
{
    data: &'a [T],
    predicate: &'a P,
}
impl<'a, T: Sync + Send, P: Send + Sync> Benchable<'a, T> for LoopFilter<'a, T, P>
where
    P: Fn(&&T) -> bool,
    T: Num + Default + Clone,
{
    fn start(&mut self) -> Option<T> {
        let mut sum: T = Default::default();
        for i in self.data {
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
fn is_prime(n: u32) -> bool {
    for a in 2..(n as f64).sqrt() as u32 {
        if n % a == 0 {
            return false;
        }
    }
    true
}

type Predicate = (dyn Fn(&&u32) -> bool + Sync + Send);
fn bench(c: &mut Criterion) {
    let data: std::ops::Range<u32> = 0..10_000_000;
    let mut group = c.benchmark_group("Filter");
    let predicate: &Predicate = criterion::black_box(&|&&x| x % 2 == 0);
    // let predicate: &Predicate = &|&&x| is_prime(x);
    group.warm_up_time(std::time::Duration::new(1, 0));
    group.measurement_time(std::time::Duration::new(3, 0));
    group.sample_size(10);

    let cpus: Vec<usize> = vec![1, 2, 3, 4, 8, 16, 24, 32]
        .iter()
        .filter(|&&i| i <= num_cpus::get())
        .cloned()
        .collect();

    let mut test: Vec<TestConfig<u32>> = vec![];
    let predicate = &predicate;
    for i in &cpus {
        for s in vec![6, 8] {
            let t = TestConfig::new(
                data.len(),
                *i,
                Some(s),
                MyAdaptiveFilter { data, predicate },
            );
            test.push(t);
            let t = TestConfig::new(data.len(), *i, Some(s), MyNewAdaptive { data, predicate });
            test.push(t);
        }
        let f = AdaptiveFilter { data, predicate };
        test.push(TestConfig::new(data.len(), *i, None, f));
        test.push(TestConfig::new(
            data.len(),
            *i,
            None,
            TryFoldFilter { data, predicate },
        ));

        let t = TestConfig::new(data.len(), *i, None, RayonFilter { data, predicate });
        test.push(t);
    }
    let r = LoopFilter {
        data,
        predicate: &predicate,
    };

    let t = TestConfig::new(data.len(), 1, None, r);
    test.push(t);
    let r = IteratorFilter {
        data,
        predicate: &predicate,
    };

    let t = TestConfig::new(data.len(), 1, None, r);
    test.push(t);

    // r.start();
    let mut t = Tester::new(test, group, None);
    t.run();

    // group.finish();
}
criterion_group!(benches, bench);
criterion_main!(benches);
