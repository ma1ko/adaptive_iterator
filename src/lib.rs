use adaptive_algorithms::task::SimpleTask;

#[test]
pub fn test() {
    main()
}
pub fn main() {
    // let pool = adaptive_algorithms::rayon::get_custom_thread_pool(1, 8);

    let a: Vec<u32> = (0..100_000_000).into_iter().collect();

    type Predicate = (dyn Fn(&&u32) -> bool + Sync + Send);
    let predicate: &Predicate = &|&&x| x % 2 == 0;
    #[cfg(not(feature = "logs"))]
    {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(1)
            .build()
            .unwrap();
        use rayon::prelude::*;
        // result = pool.install(|| filter(&mut a, &|&&a| a % 2 == 0));
        let result = pool.install(|| a.par_iter().filter(|&&x| x % 2 == 0).collect::<Vec<&u32>>());
    }

    #[cfg(feature = "logs")]
    {
        let start = std::time::Instant::now();

        let result: Vec<&u32> = a.iter().filter(predicate).collect::<Vec<&u32>>();
        assert_eq!(result.len(), 50_000_000);
        assert!(result.iter().all(|&&x| x % 2 == 0));

        println!("Sequential Runtime: {} ms", start.elapsed().as_millis());

        use rayon_logs::prelude::*;
        let pool = rayon_logs::ThreadPoolBuilder::new()
            .num_threads(1)
            .build()
            .unwrap();
        // let (r, log) = pool.logging_install(|| filter(&mut a, &|&&a| a % 2 == 0));
        let start = std::time::Instant::now();
        let (result, log) =
            pool.logging_install(|| a.par_iter().filter(predicate).collect::<Vec<&u32>>());

        // let (r, log) = pool.logging_install(|| a.par_iter().filter(&|&&a| a % 2 == 0).collect::<Vec<&u32>>());
        println!("Rayon Runtime: {} ms", start.elapsed().as_millis());
        log.save_svg("log.svg").unwrap();
        assert_eq!(result.len(), 50_000_000);
        assert!(result.iter().all(|&&x| x % 2 == 0));
    }
}

struct Filter<'a, T: Sync + Send, P: Send + Sync>
where
    P: Fn(&&T) -> bool,
{
    data: &'a [T],
    result: Vec<&'a T>,
    predicate: &'a P,
}

pub fn filter<'a, T: Sync + Send, P: Send + Sync + 'a>(
    input: &'a [T],
    predicate: &'a P,
) -> Vec<&'a T>
where
    P: Fn(&&T) -> bool,
{
    let mut x = Filter {
        data: input,
        predicate: predicate,
        result: Vec::new(),
    };
    x.run();
    return x.result;
}

impl<'a, T: Send + Sync, P: Send + Sync> SimpleTask for Filter<'a, T, P>
where
    P: Fn(&&T) -> bool,
{
    fn step(&mut self) {
        let cut = 1024.min(self.data.len());
        let left = cut_off_left(&mut self.data, cut);

        // let result = left.iter().filter(self.predicate).collect::<Vec<&T>>();
        for e in left {
            if (self.predicate)(&e) {
                self.result.push(e);
            }
        }
    }
    fn can_split(&self) -> bool {
        self.data.len() > 1024
    }
    fn is_finished(&self) -> bool {
        self.data.is_empty()
    }
    fn split(&mut self, mut runner: impl FnMut(&mut Vec<&mut Self>), steal_counter: usize) {
        let mid = self.data.len() / 2;
        let right = cut_off_right(&mut self.data, mid);
        let mut other = Filter {
            data: right,
            result: Vec::new(),
            predicate: self.predicate,
        };
        runner(&mut vec![self, &mut other]);
    }
    fn fuse(&mut self, other: &mut Self) {
        self.result.append(&mut other.result);
    }
}
pub fn cut_off_left<'a, T>(s: &mut &'a [T], mid: usize) -> &'a [T] {
    let tmp: &'a [T] = ::std::mem::replace(&mut *s, &mut []);
    let (left, right) = tmp.split_at(mid);
    *s = right;
    left
}
pub fn cut_off_right<'a, T>(s: &mut &'a [T], mid: usize) -> &'a [T] {
    let tmp: &'a [T] = ::std::mem::replace(&mut *s, &mut []);
    let (left, right) = tmp.split_at(mid);
    *s = left;
    right
}
