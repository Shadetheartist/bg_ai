use rand::{Rng};
use crate::{Action, GameTree, Player, State};

pub fn mcts<
    R: Rng,
    S: State<A, P>,
    A: Action,
    P: Player,
>(state: &S, rng: &mut R, num_simulations: u32) -> Option<A> {
    let tree = build_monte_carlo_game_tree(state, rng, num_simulations);
    tree.best_action().cloned()
}

pub fn build_monte_carlo_game_tree<
    R: Rng,
    S: State<A, P>,
    A: Action,
    P: Player,
>(state: &S, rng: &mut R, num_simulations: u32) -> GameTree<S, A, P> {
    let mut tree = GameTree::new(state.clone());
    tree.search_n(rng, num_simulations);
    tree
}


pub trait MctsAgent<P: Player> {
    fn player(&self) -> P;
    fn decide<
        R: Rng,
        S: State<A, P>,
        A: Action,
    >(&self, rng: &mut R, state: &S) -> Option<A>;
}

pub struct Agent<P: Player> {
    player: P,
    num_simulations: u32,
}

impl<P: Player> MctsAgent<P> for Agent<P> {
    fn player(&self) -> P {
        self.player
    }

    fn decide<
        R: Rng,
        S: State<A, P>,
        A: Action,
    >(&self, rng: &mut R, state: &S) -> Option<A> {
        mcts(
            state,
            rng,
            self.num_simulations,
        )
    }
}
