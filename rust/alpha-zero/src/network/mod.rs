use std::borrow::Borrow;
use std::fmt::Debug;

use board_game::board::Board;
use board_game::wdl::WDL;

pub mod common;
pub mod dummy;
pub mod symmetry;

pub mod cpu;
pub mod cudnn;
pub mod onnx_runtime;

/// A board evaluation, either as returned by the network or as the final output of a zero tree search.
#[derive(Debug, Clone)]
pub struct ZeroEvaluation {
    /// The win, draw and loss probabilities, after normalization.
    pub wdl: WDL<f32>,

    /// The policy "vector", only containing the available moves in the order they are yielded by `available_moves`.
    /// Contains probabilities after normalization.
    pub policy: Vec<f32>,
}

//TODO maybe remove the debug bound on networks? are we using it anywhere?
pub trait Network<B: Board>: Debug {
    fn evaluate_batch(&mut self, boards: &[impl Borrow<B>]) -> Vec<ZeroEvaluation>;

    fn evaluate(&mut self, board: &B) -> ZeroEvaluation {
        let mut result = self.evaluate_batch(&[board]);
        assert_eq!(result.len(), 1);
        result.pop().unwrap()
    }
}

impl<B: Board, N: Network<B>> Network<B> for &mut N {
    fn evaluate_batch(&mut self, boards: &[impl Borrow<B>]) -> Vec<ZeroEvaluation> {
        (*self).evaluate_batch(boards)
    }
}