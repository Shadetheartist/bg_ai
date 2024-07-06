use crate::{Action, Player};

pub struct Score<A, P> where A: Action, P: Player {
    pub action: A,
    pub player: P,
    pub score: f32,
    pub num_visits: u32,
}