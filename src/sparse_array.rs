use std::mem::replace;
use std::num::NonZeroUsize;
use std::rc::Rc;

#[derive(Clone, Default)]
pub struct SparseArray<V> {
    root: Option<Ptr<V>>,
}

impl<V> SparseArray<V> {
    pub fn get(&self, index: u64) -> Option<&V> {
        self.root.as_ref().and_then(|root| root.get(0, index))
    }
}

impl<V: Clone> SparseArray<V> {
    pub fn set(&mut self, index: u64, value: V) -> Option<V> {
        match &mut self.root {
            None => {
                self.root = Some(Ptr::Leaf(Rc::new(Leaf { index, value })));
                None
            }
            Some(root) => root.set(0, index, value),
        }
    }
}

const MAX_CAPACITY: usize = 8 * std::mem::size_of::<usize>();
const BITS: u8 = MAX_CAPACITY.trailing_zeros() as u8;
const MASK: u64 = (MAX_CAPACITY - 1) as u64;

// Ideally this enum would be a single machine word with pointer tagging. But
// Rust doesn't do that yet, at least as of 1.67.
#[derive(Clone)]
enum Ptr<V> {
    Leaf(Rc<Leaf<V>>),
    Branch(Rc<Branch<V>>),
}

#[derive(Clone)]
struct Leaf<V> {
    index: u64,
    value: V,
}

#[derive(Clone)]
struct Branch<V> {
    present: NonZeroUsize,
    children: Vec<Ptr<V>>,
}

impl<V> Ptr<V> {
    fn get(&self, depth: u8, index: u64) -> Option<&V> {
        let bit = child_bit(depth, index);
        match self {
            Ptr::Branch(b) if b.present.get() & bit.get() != 0 => {
                let position = (b.present.get() & (bit.get() - 1)).count_ones() as usize;
                b.children[position].get(depth + BITS, index)
            }
            Ptr::Leaf(l) if l.index == index => Some(&l.value),
            _ => None,
        }
    }
}

impl<V: Clone> Ptr<V> {
    fn set(&mut self, depth: u8, cur_index: u64, cur_value: V) -> Option<V> {
        match self {
            Ptr::Leaf(l) if l.index == cur_index => {
                Some(replace(&mut Rc::make_mut(l).value, cur_value))
            }
            Ptr::Leaf(l) => {
                let l = l.clone();
                let mut b = Branch {
                    present: child_bit(depth, l.index),
                    children: vec![Ptr::Leaf(l)],
                };
                let result = b.set(depth, cur_index, cur_value);
                *self = Ptr::Branch(Rc::new(b));
                debug_assert!(result.is_none());
                None
            }
            Ptr::Branch(b) => Rc::make_mut(b).set(depth, cur_index, cur_value),
        }
    }
}

impl<V: Clone> Branch<V> {
    fn set(&mut self, depth: u8, index: u64, value: V) -> Option<V> {
        let Branch { present, children } = self;
        let bit = child_bit(depth, index);
        let position = (present.get() & (bit.get() - 1)).count_ones() as usize;
        if present.get() & bit.get() != 0 {
            children[position].set(depth + BITS, index, value)
        } else {
            *present |= bit;
            children.insert(position, Ptr::Leaf(Rc::new(Leaf { index, value })));
            None
        }
    }
}

fn child_bit(depth: u8, hash: u64) -> NonZeroUsize {
    let index_bits = (hash >> depth) & MASK;
    NonZeroUsize::new(1 << index_bits).unwrap()
}
