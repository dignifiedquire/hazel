use rand::{seq::IteratorRandom, thread_rng, Rng};
use std::collections::HashSet;

pub type NodeId = u64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PushPullRequest {
    from: NodeId,
    to: NodeId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PushPullResponse {
    from: NodeId,
    to: NodeId,
    selected: Option<NodeId>,
}

const DEGREE: usize = 4;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    id: NodeId,
    conns: HashSet<NodeId>,
    #[cfg(test)]
    force_send: bool,
}

impl Node {
    pub fn new(id: NodeId) -> Self {
        Node {
            id,
            conns: Default::default(),
            #[cfg(test)]
            force_send: false,
        }
    }

    pub fn add_conn(&mut self, other: NodeId) -> bool {
        self.conns.insert(other)
    }

    fn should_send(&self) -> bool {
        #[cfg(test)]
        {
            if self.force_send {
                return true;
            }
        }
        let p = self.conns.len() as f64 / DEGREE as f64;
        thread_rng().gen_bool(p)
    }

    fn should_respond(&self) -> bool {
        self.should_send()
    }

    pub fn start_push_pull(&mut self) -> Option<PushPullRequest> {
        // This Node v1
        if self.should_send() {
            //   - pick random neighbour v2
            let v2 = self
                .conns
                .iter()
                .choose(&mut thread_rng())
                .copied()
                .unwrap_or(self.id);

            //   - send push-pull request to v2
            Some(PushPullRequest {
                from: self.id,
                to: v2,
            })
        } else {
            None
        }
    }

    pub fn handle_push_pull_request(&mut self, request: PushPullRequest) -> PushPullResponse {
        // Node v2
        //   - on push-pull request from v1
        //     - pick random neighbour v3
        let selected = if self.should_respond() {
            let v3 = self.conns.iter().choose(&mut thread_rng()).copied();
            //     - delete connection: (v2, v3)
            if let Some(ref v3) = v3 {
                self.conns.remove(v3);
            }
            //     - add connection: (v2, v1)
            self.conns.insert(request.from);
            Some(v3.unwrap_or(self.id))
        } else {
            None
        };

        //     - send response to v1: "selected v3"
        PushPullResponse {
            from: self.id,
            to: request.from,
            selected,
        }
    }

    pub fn handle_push_pull_response(&mut self, response: PushPullResponse) {
        // Node x
        //    - on response
        if let Some(selected) = response.selected {
            //   - delete (v1, v2)
            self.conns.remove(&response.from);
            //      - add connection (v1, v3)
            self.conns.insert(selected);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_0() {
        let mut v1 = Node::new(1);
        v1.add_conn(2);
        let mut v2 = Node::new(2);
        v2.add_conn(3);
        let v3 = Node::new(3);

        run_proto(&mut v1, &mut v2);

        assert_eq!(v1.conns, [3].into_iter().collect());
        assert_eq!(v2.conns, [1].into_iter().collect());
        assert_eq!(v3.conns, [].into_iter().collect());
    }

    #[test]
    fn case_1() {
        let mut v1 = Node::new(1);
        v1.add_conn(2);
        let mut v2 = Node::new(2);
        v2.add_conn(1);

        run_proto(&mut v1, &mut v2);

        assert_eq!(v1.conns, [1].into_iter().collect());
        assert_eq!(v2.conns, [1].into_iter().collect());
    }

    #[test]
    fn case_2() {
        let mut v1 = Node::new(1);
        v1.add_conn(2);
        let mut v2 = Node::new(2);
        v2.add_conn(2);

        run_proto(&mut v1, &mut v2);

        assert_eq!(v1.conns, [2].into_iter().collect());
        assert_eq!(v2.conns, [1].into_iter().collect());
    }

    fn run_proto(v1: &mut Node, v2: &mut Node) {
        v1.force_send = true;
        v2.force_send = true;
        let req = v1.start_push_pull().unwrap();
        assert_eq!(req.to, 2);
        let res = v2.handle_push_pull_request(req);
        assert_eq!(res.to, 1);
        v1.handle_push_pull_response(res);
    }
}
