#[derive(Clone, Debug)]
struct Node {
    next: HashMap<char, usize>,
    link: Option<usize>,
    len: usize,
}

#[derive(Clone, Debug)]
pub struct SuffixAutomaton {
    nodes: Vec<Node>,
    last: usize,
}

impl SuffixAutomaton {
    pub fn new() -> Self {
        let mut nodes = Vec::new();
        nodes.push(Node {
            next: HashMap::new(),
            link: None,
            len: 0,
        });
        Self { nodes, last: 0 }
    }

    pub fn push(&mut self, c: char) {
        let new_node = self.nodes.len();
        let last_len = self.nodes[self.last].len;
        self.nodes.push(Node {
            next: HashMap::new(),
            link: None,
            len: last_len + 1,
        });
        let mut p: Option<usize> = Some(self.last);
        while let Some(pi) = p {
            if self.nodes[pi].next.contains_key(&c) {
                break;
            }
            self.nodes[pi].next.insert(c, new_node);
            p = self.nodes[pi].link;
        }
        let q: usize = match p {
            None => 0,
            Some(pi) => *self.nodes[pi].next.get(&c).expect("must exist"),
        };

        if p.is_none() || self.nodes[p.unwrap()].len + 1 == self.nodes[q].len {
            self.nodes[new_node].link = Some(q);
        } else {
            let new_q = self.nodes.len();
            let p_len_plus_1 = self.nodes[p.unwrap()].len + 1;
            let cloned_next = self.nodes[q].next.clone();
            let cloned_link = self.nodes[q].link;
            self.nodes.push(Node {
                next: cloned_next,
                link: cloned_link,
                len: p_len_plus_1,
            });
            self.nodes[q].link = Some(new_q);
            self.nodes[new_node].link = Some(new_q);
            while let Some(pi) = p {
                let to = self.nodes[pi].next.get(&c).copied();
                if to != Some(q) {
                    break;
                }
                self.nodes[pi].next.insert(c, new_q);
                p = self.nodes[pi].link;
            }
        }
        self.last = new_node;
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }
}