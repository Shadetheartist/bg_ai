use rand::Rng;
use crate::{Action, Outcome, Player, State};

pub fn random_rollout<
    R: Rng + Sized,
    S: State<A, P> + Clone,
    A: Action,
    P: Player,
>(game: &S, rng: &mut R) -> Outcome<P> {
    let mut game = game.clone();

    loop {
        if let Some(outcome) = game.outcome() {
            return outcome;
        }

        let actions = &game.actions()[..];
        let random_action = rand::seq::SliceRandom::choose(actions, rng);

        if let Some(action) = random_action {
            game = game.apply_action(rng, action).unwrap();
        } else {
            return Outcome::Escape("No actions available.".to_string());
        }
    }
}
