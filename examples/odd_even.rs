// use rayon::prelude::*;
use rayon_logs::prelude::*;

pub fn main() {
    let a: Vec<u32> = (0..100_000_000).into_iter().collect();

    type Predicate = (dyn Fn(&&u32) -> bool + Sync + Send);
    let predicate: &Predicate = &|&&x| x % 2 == 0;
    // #[cfg(not(feature = "logs"))]
    // {

    //     // result = pool.install(|| filter(&mut a, &|&&a| a % 2 == 0));
    //     let result = pool.install(|| a.par_iter().filter(|&&x| x % 2 == 0).collect::<Vec<&u32>>());
    // }

    #[cfg(feature = "logs")]
    {
        let start = std::time::Instant::now();

        let result: Vec<&u32> = a.iter().filter(predicate).collect::<Vec<&u32>>();
        assert_eq!(result.len(), 50_000_000);
        assert!(result.iter().all(|&&x| x % 2 == 0));

        println!("Sequential Runtime: {} ms", start.elapsed().as_millis());

        let pool = rayon_logs::ThreadPoolBuilder::new()
            .num_threads(1)
            .build()
            .unwrap();
        let start = std::time::Instant::now();
        let (result, log) =
            pool.logging_install(|| a.par_iter().filter(predicate).collect::<Vec<&u32>>());

        println!("Rayon Runtime: {} ms", start.elapsed().as_millis());
        log.save_svg("log.svg").unwrap();
        assert_eq!(result.len(), 50_000_000);
        assert!(result.iter().all(|&&x| x % 2 == 0));


        let pool = adaptive_algorithms::rayon::get_custom_thread_pool(4, 6);

        let start = std::time::Instant::now();
        let (result, log) = pool.logging_install(|| adaptive_filter::filter(&a, &predicate));
        log.save_svg("log_mine.svg").unwrap();
        assert_eq!(result.len(), 50_000_000);
        assert!(result.iter().all(|&&x| x % 2 == 0));
        println!("Mine Runtime: {} ms", start.elapsed().as_millis());

    }
    #[cfg(feature = "statistics")]
    adaptive_algorithms::task::print_statistics();
}
