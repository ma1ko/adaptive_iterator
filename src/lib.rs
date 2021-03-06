//! Adaptive reductions

mod blocked;

use rayon_try_fold::prelude::*;
use std::mem;
// use rayon_try_fold::small_channel::small_channel;
// use rayon_try_fold::Blocked;

use crate::blocked::Blocked;
use adaptive_algorithms::task::Task;

pub fn main() {}
struct Reduce<'f, T, OP, ID, P>
where
    T: Send,
    OP: Fn(T, T) -> T + Sync + Send,
    ID: Fn() -> T + Send + Sync,
    P: AdaptiveProducer<Item = T>,
{
    reducer: &'f ReduceCallback<'f, OP, ID>,
    producer: P,
    output: T,
}

impl<'f, T, OP, ID, P> Task for Reduce<'f, T, OP, ID, P>
where
    T: Send,
    OP: Fn(T, T) -> T + Sync + Send,
    ID: Fn() -> T + Send + Sync,
    P: AdaptiveProducer<Item = T>,
{
    fn is_finished(&self) -> bool {
        self.producer.completed()
    }
    fn step(&mut self) {
        let mut value = (self.reducer.identity)();
        std::mem::swap(&mut value, &mut self.output);
        let block = replace_with::replace_with_or_abort_and_return(&mut self.producer, |p| {
            p.divide_at(4096)
        });
        self.output = block.fold(value, self.reducer.op);
    }
    fn fuse(&mut self, other: &mut Self) {
        // this is ugly, but I don't really know how do to that otherwise without copy
        let mut id = (self.reducer.identity)();
        mem::swap(&mut id, &mut self.output);
        let mut other_id = (other.reducer.identity)();
        mem::swap(&mut other_id, &mut other.output);
        self.output = (self.reducer.op)(id, other_id);
    }
    fn can_split(&self) -> bool {
        // println!("Hint: {:?}", self.producer.size_hint());
        // self.producer.size_hint().0 > 4096
        true
    }
    fn split(&mut self, mut runner: impl FnMut(&mut Vec<&mut Self>), _steal_counter: usize) {
        // println!("Split");
        let other_producer =
            replace_with::replace_with_or_abort_and_return(&mut self.producer, |p| p.divide());
        let id = (self.reducer.identity)();
        let mut other = Self {
            reducer: self.reducer,
            producer: other_producer,
            output: id,
        };
        runner(&mut vec![self, &mut other]);
    }
}
// That might make it simple to convert to my Adaptive algorithm
// impl<P, I> From<P> for Adaptive<I>
// where
//     I: ParallelIterator + Sized,
//     P: rayon_try_fold::prelude::ParallelIterator<
//         Item = I::Item, Controlled = I::Controlled, Enumerable = I::Enumerable> + Sized
// {
//     // type Controlled = I::Controlled;
//     // type Enumerable = I::Enumerable;
//     // type Item = I::Item;
//     fn from(item: P) -> Self {
//         // Number { value: item }
//         unimplemented!()
//     }
// }
// that works but makes it rather annoying to use
pub fn mk_adaptive<P>(iterator: P) -> Adaptive<P>
where
    P: rayon_try_fold::prelude::ParallelIterator,
{
    Adaptive { base: iterator }
}

pub(crate) trait AdaptiveProducer: Producer {
    fn completed(&self) -> bool;
    fn partial_fold<B, F>(&mut self, init: B, fold_op: F, limit: usize) -> B
    where
        B: Send,
        F: Fn(B, Self::Item) -> B;
}

/*
pub(crate) fn block_sizes() -> impl Iterator<Item = usize> {
    // TODO: cap
    std::iter::successors(Some(1), |old: &usize| {
        old.checked_shl(1).or(Some(std::usize::MAX))
    })
}
*/

pub struct Adaptive<I> {
    pub(crate) base: I,
}

//TODO: is this always the same ?
struct ReduceCallback<'f, OP, ID> {
    op: &'f OP,
    identity: &'f ID,
}

impl<'f, T, OP, ID> ProducerCallback<T> for ReduceCallback<'f, OP, ID>
where
    T: Send,
    OP: Fn(T, T) -> T + Sync + Send,
    ID: Fn() -> T + Send + Sync,
{
    type Output = T;
    fn call<P>(self, producer: P) -> Self::Output
    where
        P: Producer<Item = T>,
    {
        let blocked_producer = Blocked::new(producer);
        let output = (self.identity)();
        adaptive_scheduler(&self, blocked_producer, output)
    }
}

//TODO: should we really pass the reduce refs by refs ?
fn adaptive_scheduler<'f, T, OP, ID, P>(
    reducer: &ReduceCallback<'f, OP, ID>,
    producer: P,
    output: T, // What's that for?
) -> T
where
    T: Send,
    OP: Fn(T, T) -> T + Sync + Send,
    ID: Fn() -> T + Send + Sync,
    P: AdaptiveProducer<Item = T>,
{
    // println!("Running");
    let mut r = Reduce {
        reducer,
        producer,
        output: (reducer.identity)(),
    };
    r.run();

    r.output
}

impl<I> ParallelIterator for Adaptive<I>
where
    I: ParallelIterator,
{
    type Controlled = I::Controlled;
    type Enumerable = I::Enumerable;
    type Item = I::Item;
    //TODO: why isnt this the default function ?
    //ANSWER: Maybe you could add an associated type ReduceCallback which has to implement
    //ProducerCallback, and then use this type in the default implementation of the reduce?
    //Oh, you can't because there is no default associated type
    fn reduce<OP, ID>(self, identity: ID, op: OP) -> Self::Item
    where
        OP: Fn(Self::Item, Self::Item) -> Self::Item + Sync + Send,
        ID: Fn() -> Self::Item + Sync + Send,
    {
        let reduce_cb = ReduceCallback {
            op: &op,
            identity: &identity,
        };
        self.with_producer(reduce_cb)
    }
    fn with_producer<CB>(self, callback: CB) -> CB::Output
    where
        CB: ProducerCallback<Self::Item>,
    {
        self.base.with_producer(callback)
    }
    fn for_each<OP>(self, op: OP)
    where
        OP: Fn(Self::Item) + Sync + Send,
    {
        self.map(op).adaptive().reduce(|| (), |_, _| ())
    }
}

// TODO: do I need that?
/*
struct Worker<'f, S, C, D, W, SD> {
    state: S,
    completed: &'f C,
    divide: &'f D,
    should_divide: &'f SD,
    work: &'f W,
}

impl<'f, S, C, D, W, SD> Iterator for Worker<'f, S, C, D, W, SD>
where
    W: Fn(&mut S, usize) + Sync,
    C: Fn(&S) -> bool + Sync,
{
    type Item = ();
            x
    fn next(&mut self) -> Option<Self::Item> {
        None
    }
    fn fold<B, F>(mut self, init: B, _f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        (self.work)(&mut self.state, std::usize::MAX);
        init
    }
}

impl<'f, S, C, D, W, SD> Divisible for Worker<'f, S, C, D, W, SD>
where
    S: Send,
    C: Fn(&S) -> bool + Sync,
    D: Fn(S) -> (S, S) + Sync,
    W: Fn(&mut S, usize) + Sync,
    SD: Fn(&S) -> bool + Sync,
{
    type Controlled = False;
    fn should_be_divided(&self) -> bool {
        (self.should_divide)(&self.state)
    }
    fn divide(self) -> (Self, Self) {
        let (left, right) = (self.divide)(self.state);
        (
            Worker {
                state: left,
                completed: self.completed,
                divide: self.divide,
                should_divide: self.should_divide,
                work: self.work,
            },
            Worker {
                state: right,
                completed: self.completed,
                should_divide: self.should_divide,
                divide: self.divide,
                work: self.work,
            },
        )
    }
    fn divide_at(self, _index: usize) -> (Self, Self) {
        panic!("should never be called")
    }
}

impl<'f, S, C, D, W, SD> Producer for Worker<'f, S, C, D, W, SD>
where
    S: Send,
    C: Fn(&S) -> bool + Sync,
    D: Fn(S) -> (S, S) + Sync,
    W: Fn(&mut S, usize) + Sync,
    SD: Fn(&S) -> bool + Sync,
{
    fn preview(&self, _index: usize) -> Self::Item {
        panic!("you cannot preview a Worker")
    }
}

impl<'f, S, C, D, W, SD> AdaptiveProducer for Worker<'f, S, C, D, W, SD>
where
    S: Send,
    C: Fn(&S) -> bool + Sync,
    D: Fn(S) -> (S, S) + Sync,
    W: Fn(&mut S, usize) + Sync,
    SD: Fn(&S) -> bool + Sync,
{
    fn completed(&self) -> bool {
        (self.completed)(&self.state)
    }
    fn partial_fold<B, F>(&mut self, init: B, _fold_op: F, limit: usize) -> B
    where
        B: Send,
        F: Fn(B, Self::Item) -> B,
    {
        (self.work)(&mut self.state, limit);
        init
    }
}

pub fn work<S, C, D, W, SD>(init: S, completed: C, divide: D, work: W, should_be_divided: SD)
where
    S: Send,
    C: Fn(&S) -> bool + Sync,
    D: Fn(S) -> (S, S) + Sync,
    W: Fn(&mut S, usize) + Sync,
    SD: Fn(&S) -> bool + Sync,
{
    let worker = Worker {
        state: init,
        completed: &completed,
        divide: &divide,
        work: &work,
        should_divide: &should_be_divided,
    };
    let identity = || ();
    let op = |_, _| ();
    let reducer = ReduceCallback {
        op: &op,
        identity: &identity,
    };
    adaptive_scheduler(&reducer, worker, ());
}
*/
