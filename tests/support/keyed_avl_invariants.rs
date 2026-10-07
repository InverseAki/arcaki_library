impl<K: Ord, M: KeyedAvlMonoid> KeyedAvlTree<K, M> {
    pub(crate) fn check_invariants(&self)
    where
        M::S: PartialEq,
    {
        fn check<'a, K: Ord, M: KeyedAvlMonoid>(
            t: &'a AvlLink<K, M>,
        ) -> (usize, usize, M::S, M::S, Option<&'a K>, Option<&'a K>)
        where
            M::S: PartialEq,
        {
            let Some(t) = t else {
                return (0, 0, M::identity(), M::identity(), None, None);
            };
            let (ls, lh, lp, lr, lmin, lmax) = check(&t.left);
            let (rs, rh, rp, rr, rmin, rmax) = check(&t.right);
            assert!(lh.abs_diff(rh) <= 1);
            assert_eq!(t.size, ls + rs + 1);
            assert_eq!(t.height, 1 + lh.max(rh));
            assert!(lmax.is_none_or(|k| k < &t.key));
            assert!(rmin.is_none_or(|k| k > &t.key));
            let p = M::op(&M::op(&lp, &t.value), &rp);
            let r = M::op(&M::op(&rr, &t.value), &lr);
            assert!(t.prod == p && t.reverse_prod == r);
            (
                t.size,
                t.height,
                p,
                r,
                lmin.or(Some(&t.key)),
                rmax.or(Some(&t.key)),
            )
        }
        check(&self.root);
    }
}
