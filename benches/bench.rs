#[macro_use]
extern crate criterion;
extern crate rayon;
extern crate rayon_adaptive;

use adaptive_filter::filter;
use rayon::prelude::*;
use rayon_adaptive::prelude::*;

use criterion::Criterion;

pub struct RayonFilter<'a, T: Sync + Send> {
    data: &'a [T],
    result: Vec<&'a T>,
    predicate: &'a (dyn Fn(&&T) -> bool + Send + Sync),
}
use adaptive_algorithms::adaptive_bench::{TestConfig, Tester};
use adaptive_algorithms::Benchable;
impl<'a, T: Sync + Send, R> Benchable<'a, R> for RayonFilter<'a, T> {
    fn start(&mut self) {
        self.result = self
            .data
            .par_iter()
            .filter(self.predicate)
            .collect::<Vec<&T>>();
    }
    fn name(&self) -> &'static str {
        "Rayon Filter"
    }
    fn get_result(&self) -> R {
        unimplemented!();
    }
}
pub struct IteratorFilter<'a, T: Sync + Send> {
    data: &'a [T],
    result: Vec<&'a T>,
    predicate: &'a (dyn Fn(&&T) -> bool + Send + Sync),
}
impl<'a, T: Sync + Send> Benchable<'a, ()> for IteratorFilter<'a, T> {
    fn start(&mut self) {
        self.result = self.data.iter().filter(self.predicate).collect::<Vec<&T>>();
    }
    fn name(&self) -> &'static str {
        "Regalur Iterator"
    }
    fn get_result(&self) -> () {
        unimplemented!();
    }
    fn reset(&mut self) {
        self.result.clear();
    }
}
pub struct AdaptiveFilter<'a, T: Sync + Send> {
    data: &'a [T],
    result: Vec<&'a T>,
    predicate: &'a (dyn Fn(&&T) -> bool + Send + Sync),
}
impl<'a, T: Sync + Send> Benchable<'a, ()> for AdaptiveFilter<'a, T> {
    fn start(&mut self) {
        self.result = self
            .data
            .into_adapt_iter()
            .filter(self.predicate)
            .collect::<Vec<&T>>();
    }
    fn name(&self) -> &'static str {
        "Adaptive Filter"
    }
    fn get_result(&self) -> () {
        unimplemented!();
    }
    fn reset(&mut self) {
        self.result.clear();
    }
}
pub struct MyAdaptiveFilter<'a, T: Sync + Send, P: Send + Sync + 'a>
where
    P: Fn(&&T) -> bool,
{
    data: &'a [T],
    result: Vec<&'a T>,
    predicate: &'a P,
}
impl<'a, T: Sync + Send, P: Send + Sync> Benchable<'a, ()> for MyAdaptiveFilter<'a, T, P>
where
    P: Fn(&&T) -> bool,
{
    fn start(&mut self) {
        self.result = filter(self.data, self.predicate);
    }
    fn name(&self) -> &'static str {
        "My Adaptive Version"
    }
    fn get_result(&self) -> () {
        unimplemented!();
    }
    fn reset(&mut self) {
        self.result.clear();
    }
}
pub struct LoopFilter<'a, T: Sync + Send, P: Send + Sync + 'a>
where
    P: Fn(&&T) -> bool,
{
    data: &'a [T],
    result: Vec<&'a T>,
    predicate: &'a P,
}
impl<'a, T: Sync + Send, P: Send + Sync> Benchable<'a, ()> for LoopFilter<'a, T, P>
where
    P: Fn(&&T) -> bool,
{
    fn start(&mut self) {
        for i in self.data {
            if (self.predicate)(&i) {
                self.result.push(i);
            }
        }
    }
    fn name(&self) -> &'static str {
        "Regular Loop"
    }
    fn get_result(&self) {
        unimplemented!();
    }
    fn reset(&mut self) {
        self.result.clear();
    }
}

fn bench(c: &mut Criterion) {
    let data: &Vec<u32> = &(0..5_000_000).into_iter().collect();
    let mut group = c.benchmark_group("NearestNeighbor");
    let predicate: &(dyn Fn(&&u32) -> bool + Sync + Send) = &|&&x| x % 2 == 0;
    group.warm_up_time(std::time::Duration::new(1, 0));
    group.measurement_time(std::time::Duration::new(3, 0));
    group.sample_size(10);

    let cpus: Vec<usize> = vec![1, 2, 3, 4, 8, 16, 24, 32]
        .iter()
        .filter(|&&i| i <= num_cpus::get())
        .cloned()
        .collect();

    let mut test: Vec<TestConfig<()>> = vec![];
    for i in &cpus {
        for s in vec![6, 8] {
            let t = TestConfig {
                len: data.len(),
                num_cpus: *i,
                backoff: Some(s),
                test: Box::new(MyAdaptiveFilter {
                    data,
                    result: Vec::new(),
                    predicate: &predicate,
                }),
            };
            test.push(t);
        }
        let t = TestConfig {
            len: data.len(),
            num_cpus: *i,
            backoff: None,
            test: Box::new(AdaptiveFilter {
                data,
                result: Vec::new(),
                predicate: &predicate,
            }),
        };
        test.push(t);
    }
    let r = LoopFilter {
        data,
        result: Vec::new(),
        predicate: &predicate,
    };

    let t = TestConfig {
        len: data.len(),
        num_cpus: 1,
        backoff: None,
        test: Box::new(r),
    };
    test.push(t);
    let r = IteratorFilter {
        data,
        result: Vec::new(),
        predicate: &predicate,
    };

    let t = TestConfig {
        len: data.len(),
        num_cpus: 1,
        backoff: None,
        test: Box::new(r),
    };
    test.push(t);

    // r.start();
    let mut t = Tester::new(test, group, None);
    t.run();

    // group.finish();
}

