#[macro_use]
extern crate criterion;
extern crate rayon;
extern crate rayon_adaptive;

use adaptive_filter::filter;
use rayon::prelude::*;
use rayon_adaptive::prelude::*;

use criterion::Criterion;

fn filter_collect_adaptive(c: &mut Criterion) {
    let mut c = c.benchmark_group("test");
    c.sample_size(10);
    let size = 2usize.pow(24);
    let x = (0..size).collect::<Vec<usize>>();
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(1)
        .steal_callback(|x| adaptive_algorithms::steal::steal(20, x))
        .build()
        .unwrap();

    c.bench_function("my adaptive filter_collect", |b| {
        b.iter(|| {
            pool.install(|| {
                let y = filter(&x, &|x| x % 2 == 0);
                assert!(y.len() == x.len() / 2);
                y
            });
        }) 
    });

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(1)
        .build()
        .unwrap();
    c.bench_function("adaptive filter_collect", |b| {
        b.iter(|| {
            pool.install(|| {
                x.into_adapt_iter()
                    .filter(|&x| x % 2 == 0)
                    .collect::<Vec<&usize>>()
            });
        })
    });
    c.bench_function("sequential filter_collect", |b| {
        b.iter(|| x.iter().filter(|&x| x % 2 == 0).collect::<Vec<&usize>>())
    });
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(1)
        .build()
        .unwrap();
    c.bench_function("rayon filter_collect", |b| {
        b.iter(|| {
            pool.install(|| {
                x.par_iter()
                    .filter(|&x| x % 2 == 0)
                    .collect::<Vec<&usize>>()
            });
        })
    });
}

criterion_group!(benches, filter_collect_adaptive);
criterion_main!(benches);
