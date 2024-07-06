use rand::Rng;
use crate::{Action, Player, State};

pub trait Determinable<S: State<A, P>, A: Action, P: Player> {
    fn determine<R: Rng>(&self, rng: &mut R, perspective_player: P) -> S;
}