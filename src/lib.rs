mod ai;

use std::fmt::Debug;
use std::hash::Hash;
use rand::{Rng};

pub use ai::{
    mcts,
    ismcts,
    game_tree::{
        GameTree,
        node::GameTreeNode,
        edge::GameTreeEdge,
    },
    random_rollout::random_rollout
};

pub trait Action: Clone {}

pub trait Player: 'static + Copy + Clone + Hash + Eq + PartialEq {}

pub trait State<A: Action, P: Player>: Sized + Clone {
    type Error: Debug;

    fn actions(&self) -> Vec<A>;
    fn apply_action<R: Rng>(&self, rng: &mut R, action: &A) -> Result<Self, Self::Error>;
    fn outcome(&self) -> Option<Outcome<P>>;

    fn current_player(&self) -> P;
}

pub enum Outcome<P: Player> {
    Winner(P),
    Draw(Vec<P>),
    Escape(String),
}

