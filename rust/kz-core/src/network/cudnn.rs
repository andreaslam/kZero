use std::borrow::Borrow;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;

use board_game::board::Board;
use itertools::Itertools;
use kn_cuda_eval::executor::CudaExecutor;
use kn_cuda_eval::Device;
use kn_graph::dtype::{DTensor, Tensor};
use kn_graph::graph::Graph;

use kz_util::sequence::VecExtPad;

use crate::mapping::BoardMapper;
use crate::network::common::{check_graph_shapes, decode_output};
use crate::network::{Network, ZeroEvaluation};

pub struct CudaNetwork<B: Board, M: BoardMapper<B>> {
    mapper: M,
    max_batch_size: usize,

    executor: CudaExecutor,

    input: Vec<f32>,
    ph: PhantomData<B>,
}

impl<B: Board, M: BoardMapper<B>> CudaNetwork<B, M> {
    pub fn new(mapper: M, graph: &Graph, max_batch_size: usize, device: Device) -> Self {
        check_graph_shapes(mapper, graph);

        let executor = CudaExecutor::new(device, graph, max_batch_size);

        let input = vec![0.0; max_batch_size * mapper.input_full_len()];

        CudaNetwork {
            max_batch_size,
            mapper,
            executor,
            input,
            ph: PhantomData,
        }
    }

    pub fn executor(&mut self) -> &mut CudaExecutor {
        &mut self.executor
    }
}

impl<B: Board, M: BoardMapper<B>> Network<B> for CudaNetwork<B, M> {
    fn max_batch_size(&self) -> usize {
        self.max_batch_size
    }

    fn evaluate_batch(&mut self, boards: &[impl Borrow<B>]) -> Vec<ZeroEvaluation<'static>> {
        let batch_size = boards.len();
        let max_batch_size = self.max_batch_size;
        assert!(batch_size <= max_batch_size);

        // encode input, padded until batch size
        self.input.clear();
        for board in boards {
            self.mapper.encode_input_full(&mut self.input, board.borrow())
        }
        self.input.pad(max_batch_size * self.mapper.input_full_len(), f32::NAN);

        // TODO switch to tensor/array view here? this extra copy is sad
        let mut input_shape = vec![self.max_batch_size];
        input_shape.extend_from_slice(&self.mapper.input_full_shape());
        let input = Tensor::from_shape_vec(input_shape, self.input.clone()).unwrap();

        // run the actual computation
        let outputs = self.executor.evaluate(&[DTensor::F32(input)]);

        let relevant_outputs = outputs
            .iter()
            .map(|x| {
                let x = x.unwrap_f32().unwrap().as_slice().unwrap();
                let other_size = x.len() / max_batch_size;
                &x[0..batch_size * other_size]
            })
            .collect_vec();

        // decode the relevant part of the output
        // the number and shape of outputs has been checked already
        decode_output(self.mapper, boards, &relevant_outputs)
    }
}

//TODO figure out a better debug format, maybe something which includes network input and output dims
impl<B: Board, M: BoardMapper<B>> Debug for CudaNetwork<B, M> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CudnnNetwork")
            .field("mapper", &self.mapper)
            .field("max_batch_size", &self.max_batch_size)
            .finish()
    }
}
