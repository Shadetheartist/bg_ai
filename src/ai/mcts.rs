use rand::{Rng};
use crate::{Action, GameTree, Player, State};

pub fn mcts<
    R: Rng,
    S: State<A, P>,
    A: Action,
    P: Player,
>(game: &S, rng: &mut R, num_simulations: usize) -> Option<A> {
    let tree = build_monte_carlo_game_tree(game, rng, num_simulations);
    tree.best_action().cloned()
}

pub fn build_monte_carlo_game_tree<
    R: Rng,
    S: State<A, P>,
    A: Action,
    P: Player,
>(game: &S, rng: &mut R, num_simulations: usize) -> GameTree<S, A, P> {
    let mut tree = GameTree::new(game.clone());
    tree.search_n(rng, num_simulations);
    tree
}

