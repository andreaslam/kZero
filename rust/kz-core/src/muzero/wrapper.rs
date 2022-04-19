use board_game::ai::Bot;
use std::fmt::{Debug, Formatter};

use board_game::board::Board;

use crate::mapping::BoardMapper;
use crate::muzero::step::{
    muzero_step_apply, muzero_step_gather, MuZeroExpandRequest, MuZeroRequest, MuZeroResponse, MuZeroRootRequest,
};
use crate::muzero::tree::MuTree;
use crate::network::muzero::{MuZeroExpandExecutor, MuZeroRootExecutor};
use crate::zero::node::UctWeights;
use crate::zero::step::FpuMode;

#[derive(Debug, Copy, Clone)]
pub struct MuZeroSettings {
    pub batch_size: usize,
    pub weights: UctWeights,
    pub use_value: bool,
    pub fpu_mode: FpuMode,
    pub top_moves: usize,
}

impl MuZeroSettings {
    pub fn new(batch_size: usize, weights: UctWeights, use_value: bool, fpu_mode: FpuMode, top_moves: usize) -> Self {
        Self {
            batch_size,
            weights,
            use_value,
            fpu_mode,
            top_moves,
        }
    }
}

impl MuZeroSettings {
    /// Construct a new tree from scratch on the given board.
    pub fn build_tree<B: Board, M: BoardMapper<B>>(
        self,
        root_board: &B,
        root_exec: &mut MuZeroRootExecutor<B, M>,
        expand_exec: &mut MuZeroExpandExecutor<B, M>,
        draw_depth: u32,
        stop: impl FnMut(&MuTree<B, M>) -> bool,
    ) -> MuTree<B, M> {
        assert_eq!(root_exec.mapper, expand_exec.mapper);

        let mut tree = MuTree::new(root_board.clone(), root_exec.mapper);
        self.expand_tree(&mut tree, root_exec, expand_exec, draw_depth, stop);
        tree
    }

    // Continue expanding an existing tree.
    pub fn expand_tree<B: Board, M: BoardMapper<B>>(
        self,
        tree: &mut MuTree<B, M>,
        root_exec: &mut MuZeroRootExecutor<B, M>,
        expand_exec: &mut MuZeroExpandExecutor<B, M>,
        draw_depth: u32,
        mut stop: impl FnMut(&MuTree<B, M>) -> bool,
    ) {
        'outer: loop {
            if stop(tree) {
                break 'outer;
            }

            // gather next request
            let request = muzero_step_gather(tree, self.weights, self.use_value, self.fpu_mode, draw_depth);

            // evaluate request
            if let Some(request) = request {
                let response = match request {
                    MuZeroRequest::Root(MuZeroRootRequest { node, board }) => {
                        let (state, eval) = root_exec.eval_root(&[board]).remove(0);
                        MuZeroResponse { node, state, eval }
                    }
                    MuZeroRequest::Expand(MuZeroExpandRequest {
                        node,
                        state,
                        move_index,
                    }) => {
                        let (state, eval) = expand_exec.eval_expand(&[(state, move_index)]).remove(0);
                        MuZeroResponse { node, state, eval }
                    }
                };

                // apply response
                muzero_step_apply(tree, self.top_moves, response);
            };
        }
    }
}

pub struct MuZeroBot<B: Board, M: BoardMapper<B>> {
    settings: MuZeroSettings,
    visits: u64,
    mapper: M,
    root_exec: MuZeroRootExecutor<B, M>,
    expand_exec: MuZeroExpandExecutor<B, M>,
}

impl<B: Board, M: BoardMapper<B>> Debug for MuZeroBot<B, M> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MuZeroBot")
            .field("settings", &self.settings)
            .field("visits", &self.visits)
            .field("mapper", &self.mapper)
            .finish()
    }
}

impl<B: Board, M: BoardMapper<B>> MuZeroBot<B, M> {
    pub fn new(
        settings: MuZeroSettings,
        visits: u64,
        mapper: M,
        root_exec: MuZeroRootExecutor<B, M>,
        expand_exec: MuZeroExpandExecutor<B, M>,
    ) -> Self {
        Self {
            settings,
            visits,
            mapper,
            root_exec,
            expand_exec,
        }
    }
}

impl<B: Board, M: BoardMapper<B>> Bot<B> for MuZeroBot<B, M> {
    fn select_move(&mut self, board: &B) -> B::Move {
        let tree = self
            .settings
            .build_tree(board, &mut self.root_exec, &mut self.expand_exec, u32::MAX, |tree| {
                tree.root_visits() >= self.visits
            });
        let index = tree.best_move_index().unwrap();
        // the root move index is always valid
        let mv = self.mapper.index_to_move(board, index).unwrap();
        mv
    }
}
