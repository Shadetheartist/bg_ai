# bg_ai (Board Game Artificial Intelligence)

This crate provides a set of board game traits and a generic implementation of the Monte-Carlo Tree Search (MCTS) and
Information-Set Monte-Carlo Tree Search (IS_MCTS) algorithms.

Vanilla MCTS is suitable for perfect information games, such as Chess or Go. This implementation allows for games having
any number of players. However, it is not suitable for many modern board games which utilize hidden information or
random chance.

Information-Set MCTS is suitable for use in multi-player imperfect information games, under which most modern board
games fit into. This implementation provides a multithreaded option which distributes the work needed to simulate each
determination of the game state across multiple CPUs.
At this time, games of random chance are not supported by the IS_MCTS algorithm.

## Example

This example bootstraps a game, "Acquire", which implements the trait `State<A, P>` where `A` is an action, and `P` is a
player id.

An external RNG is provided to ensure determinism.

A map of `Player` -> `Agent` is created to give each agent a player in the game in which to make decisions for, each
agent is given a specific strength based on the amount of computational work it's allowed to do.
Agents actually make decisions, players are just state data.

`bg_ai::ismcts::MultithreadedInformationSetGame` is a very simple abstraction of a game controlled by the agents in the
map.

``` rust
use std::collections::HashMap;
use bg_ai::ismcts::MtAgent;
use rand_chacha::rand_core::SeedableRng;
use acquire::{Acquire, Options, PlayerId};


fn main() {
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(2);
    let initial_game_state = Acquire::new(&mut rng, &Options::default());
    let agents: HashMap<PlayerId, MtAgent<PlayerId>> = initial_game_state
        .players()
        .iter()
        .enumerate()
        .map(|(idx, player)| (
            player.id,
            MtAgent {
                player: player.id,
                num_simulations: 100 + 250 * idx as u32,
                num_determinations: 4 + 4 * idx as u32,
            }
        )).collect();

    let mut game = bg_ai::ismcts::MultithreadedInformationSetGame::new(rng, initial_game_state, agents);

    loop {
        if game.is_terminated() {
            break;
        }

        let action = game.step().unwrap();

        println!("{}", action);
        println!("{}", game.state);
    }
}
```

## A note on `impl`

This crate separates MCTS from IS_MCTS from IS_MCTS_MT (multithreaded) because they each require more strict trait
bounds, respectively. So each implementation only requires an impl of it's specific traits. MCTS is the simplest to
implement, IS_MCTS is about the same, except it requires `Determinable` and a few other derivable traits. IS_MCTS_MT
requires the same as IS_MCTS but also some traits demanded by the use of multithreading such as `Send + Sync`. 


