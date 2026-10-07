impl<K: Ord, M: KeyedAvlMonoid> SortableSequence<K, M> {
    pub(crate) fn check_invariants(&self)
    where
        M::S: PartialEq,
    {
        let mut at = 0;
        for (&start, block) in &self.blocks {
            assert_eq!(start, at);
            assert!(!block.tree.is_empty());
            block.tree.check_invariants();
            assert!(self.products[self.size + start] == block.all_prod());
            for i in start + 1..start + block.tree.len() {
                assert!(self.products[self.size + i] == M::identity());
            }
            at += block.tree.len();
        }
        assert_eq!(at, self.n);
        for i in (1..self.size).rev() {
            assert!(self.products[i] == M::op(&self.products[2 * i], &self.products[2 * i + 1]));
        }
    }
}
