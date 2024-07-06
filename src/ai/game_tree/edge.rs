use crate::Action;

pub struct GameTreeEdge<A> where A: Action {
    pub action: A,
    pub num_visits: u32,
}

impl<A> GameTreeEdge<A> where A: Action {
    pub fn new(action: A) -> Self {
        Self {
            action,
            num_visits: 1,
        }
    }
}