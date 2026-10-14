// Source: https://github.com/InverseAki/arcaki_library/blob/main/src/Graph/scc.rs
pub fn scc(n: usize, e: &Vec<(usize, usize)>) -> Vec<Vec<usize>>{
    let mut ord = Vec::with_capacity(n);
    let mut used = vec![false; n];
    let mut cnt = 0;
    let mut edge = vec![Vec::new(); n];
    let mut rev = vec![Vec::new(); n];
    for &(u, v) in e{
        edge[u].push(v);
        rev[v].push(u);
    }
    for i in 0..n{
        scc_dfs1(i, &edge, &mut used, &mut ord, &mut cnt);
    }
    used = vec![false; n];
    let mut res = Vec::new();
    for &v in ord.iter().rev(){
        if used[v]{continue}
        let mut ret = Vec::new();
        scc_dfs2(v, &rev, &mut used, &mut ret);
        res.push(ret);
    }
    res
}

#[inline]
fn scc_dfs1(p: usize, edge: &Vec<Vec<usize>>, used: &mut Vec<bool>, ord: &mut Vec<usize>, cnt: &mut usize){
    if used[p]{return;}
    used[p] = true;
    for &nex in &edge[p]{
        if used[nex]{continue}
        scc_dfs1(nex, edge, used, ord, cnt);
    }
    ord.push(p);
    *cnt += 1;
}

#[inline]
fn scc_dfs2(p: usize, edge: &Vec<Vec<usize>>, used: &mut Vec<bool>, res: &mut Vec<usize>){
    if used[p]{return;}
    used[p] = true;
    res.push(p);
    for &nex in &edge[p]{
        if used[nex]{continue}
        scc_dfs2(nex, edge, used, res);
    }
}