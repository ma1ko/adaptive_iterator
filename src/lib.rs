use adaptive_algorithms::task::SimpleTask;
use std::collections::LinkedList;

pub mod adaptive;
pub mod blocked;
pub mod reduce;

#[test]
pub fn test() {
    main()
}
pub fn main() {
    let a: Vec<u32> = (0..100_000_000).into_iter().collect();

    type Predicate = (dyn Fn(&&u32) -> bool + Sync + Send);
    let predicate: &Predicate = &|&&x| x % 2 == 0;

    let pool = adaptive_algorithms::rayon::get_thread_pool();
    let result;
    #[cfg(feature = "logs")]
    {
        let (res, log) = pool.logging_install(|| filter(&a, &predicate));
        result = res;
        log.save_svg("log.svg").unwrap();
    }
    #[cfg(not(feature = "logs"))]
    {
        result = pool.install(|| filter(&a, &predicate));
    }
    assert_eq!(result.len(), 50_000_000);
    assert!(result.iter().all(|&&x| x % 2 == 0));

    #[cfg(feature = "statistics")]
    adaptive_algorithms::task::print_statistics();
}

struct Filter<'a, T: Sync + Send, P: Send + Sync>
where
    P: Fn(&&T) -> bool,
{
    data: &'a [T],
    result: Vec<&'a T>,
    predicate: &'a P,
    successors: std::collections::LinkedList<Vec<&'a T>>,
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
        predicate,
        result: Vec::with_capacity(input.len()),
        successors: LinkedList::new(),
    };
    x.run();

    let vec = std::mem::replace(&mut x.result, vec![]);
    x.successors.push_front(vec);
    x.successors.iter().flatten().copied().collect::<Vec<&T>>()
    // return x.result;
}

impl<'a, T: Send + Sync, P: Send + Sync> SimpleTask for Filter<'a, T, P>
where
    P: Fn(&&T) -> bool,
{
    fn step(&mut self) {
        let cut = 4096.min(self.data.len());
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
            successors: LinkedList::new(),
        };
        runner(&mut vec![self, &mut other]);
    }
    fn fuse(&mut self, other: &mut Self) {
        // adaptive_algorithms::rayon::subgraph("Fusing", other.result.len(), || {
        // self.result.append(&mut other.result)
        // });
        let vec = std::mem::replace(&mut other.result, vec![]);
        self.successors.push_back(vec);
        self.successors.append(&mut other.successors);
    }
    fn work(&self) -> Option<(&'static str, usize)> {
        Some(("Filtering", self.data.len()))
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
