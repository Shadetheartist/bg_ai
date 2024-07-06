use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutex};
use std::thread;
use rand::{Rng};
use crate::{Action, GameTree, Player, State};
use crate::ai::game_tree::score::Score;

pub trait Determinable<S: State<A, P>, A: Action, P: Player> {
    fn determine<R: Rng>(&self, rng: &mut R, perspective_player: P) -> S;
}

type Determinizations<A, P> = Vec<Determinization<A, P>>;

struct Determinization<A, P> where A: Action, P: Player {
    #[allow(dead_code)]
    determinization_idx: u32,
    scores: Vec<Score<A, P>>,
}

pub fn ismcts<
    R: Rng + Clone,
    S: State<A, P> + Determinable<S, A, P>,
    A: Action + Eq + Hash,
    P: Player,
>(state: &S, rng: &R, num_determinizations: u32, num_simulations: u32) -> Option<A> {
    let mut determinizations: Determinizations<A, P> = Vec::new();

    for determinization_idx in 0..num_determinizations {
        {
            let mut rng = clone_and_advance_rng(rng, determinization_idx);
            let game = state.determine(&mut rng, state.current_player());

            let mut decision_tree = GameTree::new(game);

            decision_tree.search_n(&mut rng, num_simulations);

            determinizations
                .push(Determinization {
                    determinization_idx,
                    scores: decision_tree.root_scores(),
                });
        }
    }

    let current_player = state.current_player();

    let mut total_action_scores: HashMap<&A, HashMap<P, f32>> = HashMap::default();
    for determinization in &determinizations {
        for score in &determinization.scores {
            total_action_scores
                .entry(&score.action)
                .and_modify(|map| {
                    map.entry(score.player)
                        .and_modify(|s| *s += score.score)
                        .or_insert(score.score);
                }).or_insert({
                let mut map = HashMap::new();
                map.insert(score.player, score.score);
                map
            });
        }
    }

    let best_action = total_action_scores.iter().max_by(|a, b| {
        let a_score = a.1.get(&current_player).unwrap_or(&0f32);
        let b_score = b.1.get(&current_player).unwrap_or(&0f32);

        // todo: maximize the difference between their best action the sum of other players' actions.

        a_score.total_cmp(&b_score)
    }).unwrap();

    let best_action = *(best_action.0);
    let best_action = best_action.clone();
    Some(best_action)
}

pub fn ismcts_mt<
    R: Rng + Clone + Send,
    S: State<A, P> + Determinable<S, A, P> + Send,
    A: Action + Send + Sync + Eq + Hash,
    P: Player + Send + Sync,
>(state: &S, rng: &R, num_determinizations: u32, num_simulations: u32) -> Option<A> {
    let determinizations: Arc<Mutex<Determinizations<A, P>>> = Arc::new(Mutex::new(Vec::new()));

    thread::scope(|scope| {
        for determinization_idx in 0..num_determinizations {
            {
                let mut rng = clone_and_advance_rng(rng, determinization_idx);

                let determinization_scores = determinizations.clone();

                let game = state.determine(&mut rng, state.current_player());

                let mut decision_tree = GameTree::new(game);

                scope.spawn(move || {
                    decision_tree.search_n(&mut rng, num_simulations);

                    determinization_scores
                        .lock()
                        .unwrap()
                        .push(Determinization {
                            determinization_idx,
                            scores: decision_tree.root_scores(),
                        });
                });
            }
        }
    });

    let current_player = state.current_player();

    let mut total_action_scores: HashMap<&A, HashMap<P, f32>> = HashMap::default();
    let determinizations = determinizations.lock().unwrap();
    for determinization in determinizations.iter() {
        for score in &determinization.scores {
            total_action_scores
                .entry(&score.action)
                .and_modify(|map| {
                    map.entry(score.player)
                        .and_modify(|s| *s += score.score)
                        .or_insert(score.score);
                }).or_insert({
                let mut map = HashMap::new();
                map.insert(score.player, score.score);
                map
            });
        }
    }


    let best_action = total_action_scores.iter().max_by(|a, b| {
        let a_score = a.1.get(&current_player).unwrap_or(&0f32);
        let b_score = b.1.get(&current_player).unwrap_or(&0f32);

        // todo: maximize the difference between their best action the sum of other players' actions.

        a_score.total_cmp(&b_score)
    }).unwrap();

    let best_action = *(best_action.0);
    let best_action = best_action.clone();
    Some(best_action)
}

fn clone_and_advance_rng<R: Rng + Clone>(rng: &R, delta: u32) -> R {
    // clone the rng so each thread has its own copy
    let mut rng = rng.clone();

    // advance the RNG by jumping ahead 'determinization_idx' number of jumps before
    // applying a determinization, that way each determinization is unique
    for _ in 0..delta {
        rng.next_u32();
    }

    rng
}

pub trait IsMctsAgent<P: Player> {
    fn player(&self) -> P;
    fn decide<
        R: Rng + Clone,
        S: State<A, P> + Determinable<S, A, P>,
        A: Action + Eq + Hash,
    >(&self, rng: &mut R, state: &S) -> Option<A>;
}

pub struct Agent<P: Player> {
    player: P,
    num_determinations: u32,
    num_simulations: u32,
}

impl<P: Player> IsMctsAgent<P> for Agent<P> {
    fn player(&self) -> P {
        self.player
    }

    fn decide<
        R: Rng + Clone,
        S: State<A, P> + Determinable<S, A, P>,
        A: Action + Eq + Hash,
    >(&self, rng: &mut R, state: &S) -> Option<A> {
        ismcts(
            state,
            rng,
            self.num_determinations,
            self.num_simulations,
        )
    }
}
