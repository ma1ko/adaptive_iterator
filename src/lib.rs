use adaptive_algorithms::Task;
use rayon;

pub fn main() {
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(4)
        .steal_callback(|x| adaptive_algorithms::steal::steal(6, x))
        .build()
        .unwrap();

    let mut a = (0..50_000_000).into_iter().collect();
    let r = pool.install(|| filter(&mut a, &|a| a % 2 == 0));
    assert_eq!(r.len(), 25_000_000);
    assert!(r.iter().all(|&&x| x % 2 == 0));
}

struct Filter<'a, T: Sync + Send> {
    data: &'a [T],
    result: Vec<&'a T>,
    predicate: &'a (dyn Fn(&'a T) -> bool + Sync),
}

pub fn filter<'a, T: Sync + Send>(
    input: &'a Vec<T>,
    predicate: &'a (dyn Fn(&'a T) -> bool + Sync),
) -> Vec<&'a T> {
    let len = input.len();
    let mut x = Filter {
        data: input,
        predicate,
        result: Vec::with_capacity(len / 2 ),
    };
    x.run_();
    return x.result;
}

impl<'a, T: Send + Sync> Task for Filter<'a, T> {
    fn step(&mut self) {
        let cut = 1024.min(self.data.len());
        let left = cut_off_left(&mut self.data, cut);

        // let result = left.iter().filter(self.predicate).collect::<Vec<&T>>();
        for e in left {
            if (self.predicate)(e) {
                self.result.push(e);
            }
        }
    }
    fn can_split(&self) -> bool {
        true
        // self.data.len() > 1024
    }
    fn is_finished(&self) -> bool {
        self.data.is_empty()
    }
    fn split(&mut self) -> Self {
        let mid = self.data.len() / 2;
        let right = cut_off_right(&mut self.data, mid);
        let other = Filter {
            data: right,
            result: Vec::new(),
            predicate: self.predicate,
        };
        other
    }
    fn fuse(&mut self, mut other: Self) {
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
