use std::collections::HashMap;
use std::marker::PhantomData;
use crate::{Action, Player, State};

pub struct GameTreeNode<S, A, P> where S: State<A, P>, A: Action, P: Player {
    pub state: S,
    pub num_visits: u32,
    pub scores: HashMap<P, f32>,
    _phantom_data: PhantomData<A>,
}

impl<S, A, P> GameTreeNode<S, A, P> where S: State<A, P>, A: Action, P: Player {
    pub fn new(state: S) -> Self {
        Self {
            state,
            num_visits: 0,
            scores: Default::default(),
            _phantom_data: Default::default(),
        }
    }

    pub fn get_player_score(&self, player: P) -> f32 {
        if let Some(value) = self.scores.get(&player) {
            *value
        } else {
            0.0
        }
    }
}
