#[path = "keyed_avl_tree.rs"]
mod sortable_avl;
pub use self::sortable_avl::{KeyedAvlMonoid, KeyedAvlTree};

struct SortableBlock<K: Ord, M: KeyedAvlMonoid> {
    tree: KeyedAvlTree<K, M>,
    descending: bool,
}

impl<K: Ord, M: KeyedAvlMonoid> SortableBlock<K, M> {
    fn prod(&self, l: usize, r: usize) -> M::S {
        if self.descending {
            let n = self.tree.len();
            self.tree.prod_reverse(n - r, n - l)
        } else {
            self.tree.prod(l, r)
        }
    }

    fn all_prod(&self) -> M::S {
        if self.descending {
            self.tree.all_prod_reverse()
        } else {
            self.tree.all_prod()
        }
    }
}

pub struct SortableSequence<K: Ord, M: KeyedAvlMonoid> {
    n: usize,
    size: usize,
    blocks: std::collections::BTreeMap<usize, SortableBlock<K, M>>,
    products: Vec<M::S>,
}

impl<K: Ord, M: KeyedAvlMonoid> SortableSequence<K, M> {
    pub fn from_vec(values: Vec<(K, M::S)>) -> Self {
        let n = values.len();
        let size = n.next_power_of_two();
        let mut products = vec![M::identity(); 2 * size];
        let blocks = values
            .into_iter()
            .enumerate()
            .map(|(i, (key, value))| {
                products[size + i] = value.clone();
                (
                    i,
                    SortableBlock {
                        tree: KeyedAvlTree::singleton(key, value),
                        descending: false,
                    },
                )
            })
            .collect();
        for i in (1..size).rev() {
            products[i] = M::op(&products[2 * i], &products[2 * i + 1]);
        }
        Self {
            n,
            size,
            blocks,
            products,
        }
    }

    pub fn len(&self) -> usize {
        self.n
    }
    pub fn is_empty(&self) -> bool {
        self.n == 0
    }

    fn update_product(&mut self, i: usize, value: M::S) {
        let mut p = self.size + i;
        self.products[p] = value;
        while p > 1 {
            p /= 2;
            self.products[p] = M::op(&self.products[2 * p], &self.products[2 * p + 1]);
        }
    }

    fn block_products(&self, l: usize, r: usize) -> M::S {
        let (mut l, mut r) = (l + self.size, r + self.size);
        let (mut a, mut b) = (M::identity(), M::identity());
        while l < r {
            if l & 1 != 0 {
                a = M::op(&a, &self.products[l]);
                l += 1;
            }
            if r & 1 != 0 {
                r -= 1;
                b = M::op(&self.products[r], &b);
            }
            l /= 2;
            r /= 2;
        }
        M::op(&a, &b)
    }

    fn boundary(&mut self, at: usize) {
        if at == self.n || self.blocks.contains_key(&at) {
            return;
        }
        let start = *self.blocks.range(..at).next_back().unwrap().0;
        let SortableBlock { tree, descending } = self.blocks.remove(&start).unwrap();
        let offset = at - start;
        let (left, right) = if descending {
            let split_at = tree.len() - offset;
            let (lo, hi) = tree.split(split_at);
            (hi, lo)
        } else {
            tree.split(offset)
        };
        let left = SortableBlock {
            tree: left,
            descending,
        };
        let right = SortableBlock {
            tree: right,
            descending,
        };
        self.update_product(start, left.all_prod());
        self.update_product(at, right.all_prod());
        self.blocks.insert(start, left);
        self.blocks.insert(at, right);
    }

    pub fn get(&self, i: usize) -> (&K, &M::S) {
        assert!(i < self.n, "index out of bounds");
        let (&start, block) = self.blocks.range(..=i).next_back().unwrap();
        let offset = i - start;
        block.tree.get_index(if block.descending {
            block.tree.len() - offset - 1
        } else {
            offset
        })
    }

    pub fn set(&mut self, i: usize, key: K, value: M::S) {
        assert!(i < self.n, "index out of bounds");
        self.boundary(i);
        self.boundary(i + 1);
        self.update_product(i, value.clone());
        self.blocks.insert(
            i,
            SortableBlock {
                tree: KeyedAvlTree::singleton(key, value),
                descending: false,
            },
        );
    }

    pub fn sort(&mut self, l: usize, r: usize, descending: bool) {
        assert!(l <= r && r <= self.n, "invalid range");
        if l == r {
            return;
        }
        self.boundary(l);
        self.boundary(r);
        let starts: Vec<usize> = self.blocks.range(l..r).map(|(&i, _)| i).collect();
        let mut tree = KeyedAvlTree::new();
        for start in starts {
            let block = self.blocks.remove(&start).unwrap();
            tree = tree.merge(block.tree);
            if start != l {
                self.update_product(start, M::identity());
            }
        }
        let block = SortableBlock { tree, descending };
        self.update_product(l, block.all_prod());
        self.blocks.insert(l, block);
    }

    pub fn sort_asc(&mut self, l: usize, r: usize) {
        self.sort(l, r, false);
    }
    pub fn sort_desc(&mut self, l: usize, r: usize) {
        self.sort(l, r, true);
    }

    pub fn prod(&self, l: usize, r: usize) -> M::S {
        assert!(l <= r && r <= self.n, "invalid range");
        if l == r {
            return M::identity();
        }
        let (&first, a) = self.blocks.range(..=l).next_back().unwrap();
        let (&last, b) = self.blocks.range(..r).next_back().unwrap();
        if first == last {
            return a.prod(l - first, r - first);
        }
        let left = a.prod(l - first, a.tree.len());
        let mid = self.block_products(first + 1, last);
        let right = b.prod(0, r - last);
        M::op(&M::op(&left, &mid), &right)
    }

    pub fn all_prod(&self) -> M::S {
        self.products[1].clone()
    }
}

#[cfg(test)]
include!("../../tests/support/sortable_sequence_invariants.rs");
