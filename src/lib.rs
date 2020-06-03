use adaptive_algorithms::Task;
use rayon;

pub fn main() {
    let mut a =  (0..10_000).into_iter().collect();
        let r = filter(&mut a, &|a| a % 2 == 0);
    println!("{:?}", r);

}

struct Filter<'a, T: Sync + Send> {
    data: &'a mut [T],
    result: Vec<&'a T>,
    predicate: &'a (dyn Fn(&'a T) -> bool + Sync),
}

pub fn filter<'a, T: Sync + Send>(input: &'a mut Vec<T>, predicate: &'a (dyn Fn(&'a T)  -> bool + Sync)) -> Vec<&'a T> {
    let mut x = Filter {
        data: input,
        predicate,
        result: Vec::new()
    };
    x.run_();
    let res = std::mem::replace(&mut x.result, vec![]);
    return res;

}

impl<'a, T: Send + Sync> Task for Filter<'a, T> {
    fn step(&mut self) {
        let cut = 1024.max(self.data.len());
        let left = cut_off_left(&mut self.data, cut);
        for e in left {
            if (self.predicate)(e) {
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
pub fn cut_off_left<'a, T>(s: &mut &'a mut [T], mid: usize) -> &'a mut [T] {
    let tmp: &'a mut [T] = ::std::mem::replace(&mut *s, &mut []);
    let (left, right) = tmp.split_at_mut(mid);
    *s = right;
    left
}
pub fn cut_off_right<'a, T>(s: &mut &'a mut [T], mid: usize) -> &'a mut [T] {
    let tmp: &'a mut [T] = ::std::mem::replace(&mut *s, &mut []);
    let (left, right) = tmp.split_at_mut(mid);
    *s = left;
    right
}
