pub mod node;
pub mod edge;
pub mod score;

use petgraph::graph::EdgeReference;
use petgraph::prelude::*;
use rand::Rng;
use crate::{Action, Outcome, Player, State};
use crate::ai::game_tree::edge::GameTreeEdge;
use crate::ai::game_tree::node::GameTreeNode;
use crate::ai::game_tree::score::Score;
use crate::ai::random_rollout::random_rollout;

pub struct GameTree<S, A, P> where S: State<A, P>, A: Action, P: Player {
    root_node_idx: NodeIndex,
    graph: Graph<GameTreeNode<S, A, P>, GameTreeEdge<A>, Directed>,
    constant_of_exploration: f32,
}

impl<S, A, P> GameTree<S, A, P> where S: State<A, P>, A: Action, P: Player + 'static {
    pub fn new(state: S) -> Self {
        let mut graph: Graph<GameTreeNode<S, A, P>, GameTreeEdge<A>, Directed> = Graph::new();
        let root_node_idx = graph.add_node(GameTreeNode::new(state));
        Self {
            root_node_idx,
            graph,
            constant_of_exploration: 2f32.sqrt(),
        }
    }

    pub fn graph(&self) -> &Graph<GameTreeNode<S, A, P>, GameTreeEdge<A>, Directed> {
        &self.graph
    }


    fn select(&self, node_idx: NodeIndex, perspective_player: P) -> NodeIndex {
        let children = self.node_children(node_idx);

        let selected = children.iter().fold((None, f32::MIN), |acc, child_idx| {
            let ucb = self.ucbt_value(*child_idx, perspective_player);
            if ucb > acc.1 {
                (Some(*child_idx), ucb)
            } else {
                acc
            }
        });

        if let Some(selected) = selected.0 {
            selected
        } else {
            panic!("could not select a node, this node has no children")
        }
    }

    fn expand<R: Rng>(&mut self, rng: &mut R, node_idx: NodeIndex) {
        let actions = {
            let node = self.get_node(node_idx);
            node.state.actions()
        };

        if actions.len() == 0 {
            panic!("no actions to expand into")
        }

        for action in actions {
            let node = self.get_node(node_idx);
            let state = node.state.apply_action(rng, &action).unwrap();

            let new_node_idx = self.graph.add_node(GameTreeNode::new(state));
            self.graph.add_edge(node_idx, new_node_idx, GameTreeEdge::new(action));
        }
    }

    pub fn search_n<R: Rng>(&mut self, rng: &mut R, iterations: u32) {
        for _ in 0..iterations {
            self.search(rng);
        }
    }

    pub fn search<R: Rng>(&mut self, rng: &mut R) {
        let mut current_node_idx = self.root_node_idx;

        // track visited nodes for back propagation
        let mut visited_nodes = Vec::new();
        visited_nodes.push(current_node_idx);

        // Determine the perspective player
        let perspective_player = self.get_node(current_node_idx).state.current_player();

        // iteratively select an optimal node to expand
        while self.is_leaf_node(current_node_idx) == false {
            current_node_idx = self.select(current_node_idx, perspective_player);
            visited_nodes.push(current_node_idx);
        }

        // determine the outcome of the selected leaf node
        let outcome = {
            let node = self.get_node(current_node_idx);
            let outcome = node.state.outcome();
            if let Some(outcome) = outcome {
                outcome
            } else {
                self.expand(rng, current_node_idx);

                let new_node_idx = self.select(current_node_idx, perspective_player);
                visited_nodes.push(new_node_idx);

                let node = self.get_node(current_node_idx);
                random_rollout(&node.state, rng)
            }
        };

        self.back_propagate(visited_nodes, outcome);
    }

    /// This updates the num visits and each player's score for each visited node
    fn back_propagate(&mut self, visited_nodes: Vec<NodeIndex>, outcome: Outcome<P>) {
        for visited_node_idx in visited_nodes {
            let node = self.get_node_mut(visited_node_idx);
            node.num_visits += 1;

            match &outcome {
                Outcome::Winner(winner_player) => {
                    *node.scores.entry(*winner_player).or_insert(0f32) += 1.0;

                    if let Some(edge) = self.edge_to_parent(visited_node_idx) {
                        self.graph.edge_weight_mut(edge.id()).unwrap().num_visits += 1;
                    }
                }
                Outcome::Draw(drawing_players) => {
                    for drawing_player in drawing_players {
                        *node.scores.entry(*drawing_player).or_insert(0f32) += 1.0;
                    }
                }
                Outcome::Escape(_) => {}
            }
        }
    }


    /// upper confidence bound 1 for trees
    fn ucbt_value(&self, node_idx: NodeIndex, perspective_player: P) -> f32 {
        let Some(node) = self.graph.node_weight(node_idx) else {
            return 0.0;
        };

        if node.num_visits == 0 {
            return f32::MAX;
        }

        let player_score = node.get_player_score(perspective_player);

        // first component of UCB1 formula corresponds to exploitation
        // as it is high for moves with a high average win ratio
        // this is the average reward, or win ratio, of the node
        let exploitation_component = player_score / node.num_visits as f32;

        // the second component corresponds to exploration
        let parent_visits = self.parent_visits(node_idx);
        let exploration_component = self.constant_of_exploration * ((parent_visits as f32 + 1.0).ln() / node.num_visits as f32).sqrt();

        // a small amount of noise helps to avoid ties
        // let noise = rng.next_u32() as f32 * 1e-6;

        exploitation_component + exploration_component // + noise
    }

    fn try_get_node(&self, node_idx: NodeIndex) -> Option<&GameTreeNode<S, A, P>> {
        self.graph.node_weight(node_idx)
    }

    fn get_node(&self, node_idx: NodeIndex) -> &GameTreeNode<S, A, P> {
        self.try_get_node(node_idx).unwrap()
    }

    fn try_get_node_mut(&mut self, node_idx: NodeIndex) -> Option<&mut GameTreeNode<S, A, P>> {
        self.graph.node_weight_mut(node_idx)
    }

    fn get_node_mut(&mut self, node_idx: NodeIndex) -> &mut GameTreeNode<S, A, P> {
        self.try_get_node_mut(node_idx).unwrap()
    }

    fn node_children(&self, node_idx: NodeIndex) -> Vec<NodeIndex> {
        self.graph
            .edges_directed(
                node_idx,
                Outgoing,
            )
            .map(|edge| edge.target())
            .collect()
    }

    fn parent_node_idx(&self, node_idx: NodeIndex) -> Option<NodeIndex> {
        let edge_to_parent = self.edge_to_parent(node_idx)?;
        Some(edge_to_parent.source())
    }

    fn edge_to_parent(&self, node_idx: NodeIndex) -> Option<EdgeReference<GameTreeEdge<A>>> {
        let incoming_edges: Vec<EdgeReference<GameTreeEdge<A>>> = self.graph.edges_directed(node_idx, Incoming).collect();
        if incoming_edges.len() == 0 {
            return None;
        }

        Some(incoming_edges[0])
    }

    fn parent_visits(&self, node_idx: NodeIndex) -> u32 {
        let Some(parent_idx) = self.parent_node_idx(node_idx) else {
            return 0;
        };

        if let Some(node) = self.graph.node_weight(parent_idx) {
            node.num_visits
        } else {
            0
        }
    }

    fn is_leaf_node(&self, node_idx: NodeIndex) -> bool {
        self.graph.edges_directed(node_idx, Outgoing).count() == 0
    }

    pub fn root_scores(&self) -> Vec<Score<A, P>> {
        let children = self.node_children(self.root_node_idx);
        children.iter().flat_map(|child_node_idx| {
            let child_node = self.get_node(*child_node_idx);
            let num_visits = child_node.num_visits;

            let mut edges = self.graph.edges_connecting(self.root_node_idx, *child_node_idx);
            let edge_weight = edges.next().unwrap().weight();
            let action = edge_weight.action.clone();

            child_node.scores.iter().map(move |(player, score)| {
                Score {
                    action: action.clone(),
                    player: *player,
                    score: *score,
                    num_visits,
                }
            })
        }).collect()
    }

    /// selects the best action from the current state of the decision tree
    pub fn best_action(&self) -> Option<&A> {
        let children = self.node_children(self.root_node_idx);
        if let Some(most_visited_child_node_idx) = children.iter().max_by_key(|node_idx| self.get_node(**node_idx).num_visits) {
            if let Some(edge_to_parent) = self.edge_to_parent(*most_visited_child_node_idx) {
                Some(&edge_to_parent.weight().action)
            } else {
                None
            }
        } else {
            None
        }
    }
}
