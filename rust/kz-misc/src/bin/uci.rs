use std::fs::File;
use std::io::Write;
use std::io::{BufRead, BufReader};
use std::time::Instant;

use board_game::board::{Board, Player};
use board_game::games::chess::{ChessBoard, Rules};
use board_game::pov::NonPov;
use board_game::wdl::WDL;
use flume::{Receiver, RecvError, Sender, TryRecvError};
use kn_cuda_eval::Device;
use kn_graph::onnx::load_graph_from_onnx_path;
use rand::rngs::StdRng;
use rand::{thread_rng, SeedableRng};
use vampirc_uci::{UciMessage, UciTimeControl, UciSearchControl};

use kz_core::mapping::chess::ChessStdMapper;
use kz_core::network::cudnn::CudaNetwork;
use kz_core::zero::node::UctWeights;
use kz_core::zero::step::{FpuMode, QMode};
use kz_core::zero::tree::Tree;
use kz_core::zero::wrapper::ZeroSettings;

const INFO_PERIOD: f32 = 0.5;

fn main() -> std::io::Result<()> {
    // io
    let (sender, receiver) = flume::unbounded();
    std::thread::spawn(|| io_thread(sender).unwrap());

    let mut debug = File::create("kzero_log.txt")?;
    let log = &mut debug;

    // search settings
    let path = "chess_16x128_gen3634.onnx";
    let batch_size = 100;
    let settings = ZeroSettings::simple(batch_size, UctWeights::default(), QMode::wdl(), FpuMode::Relative(0.0));

    let graph = load_graph_from_onnx_path(path, false).unwrap();
    let mut network = CudaNetwork::new(ChessStdMapper, &graph, batch_size, Device::new(0));
    let mut rng = StdRng::from_entropy();

    // state
    let mut max_nodes = u64::MAX;
    let mut tree = None;
    let mut searching = false;
    let mut nodes = 0;

    let mut alloc = 0;
    let mut start_time = Instant::now();

    loop {
        // search until we receive a message
        if searching {
            if let Some(tree) = &mut tree {
                let mut prev_send = Instant::now();

                settings.expand_tree(tree, &mut network, &mut rng, |tree| {
                    let now = Instant::now();

                    if tree.root_visits() > 0 && (now - prev_send).as_secs_f32() > INFO_PERIOD {
                        let root = &tree[0];
                        let root_player = tree.root_board().next_player();

                        let mut children: Vec<_> = root.children.unwrap().iter().collect();
                        children.sort_by_key(|&c| tree[c].complete_visits);
                        children.reverse();

                        for (i, &child_index) in children.iter().enumerate() {
                            let child = &tree[child_index];
                            let wdl: WDL<u32> = (child.values().wdl_abs.pov(root_player) * 1000.0).cast();
                            let (min_depth, max_depth) = tree.depth_range(child_index);

                            println!(
                                "info depth {} seldepth {} nodes {} wdl {} {} {} multipv {} pv {}",
                                min_depth,
                                max_depth,
                                tree.root_visits(),
                                wdl.win,
                                wdl.draw,
                                wdl.loss,
                                i + 1,
                                child.last_move.unwrap(),
                            )
                        }

                        prev_send = now;
                    }

                    if nodes >= max_nodes || (start_time.elapsed().as_millis() as i64 > alloc) {
                        nodes = 0;
                        searching = false;

                        let best_move = if tree.root_visits() > 0 {
                            tree.best_move().unwrap()
                        } else {
                            tree.root_board().random_available_move(&mut thread_rng()).unwrap()
                        };

                        println!("bestmove {}", best_move);
                        return true;
                    }

                    nodes += 1;

                    !receiver.is_empty()
                });
            }
        }

        // process all messages
        while let Some(message) = receive(&receiver, !searching).unwrap() {
            writeln!(log, "> {}", message)?;

            match message {
                UciMessage::Uci => {
                    println!("id kZero");
                    println!("uciok");
                }
                UciMessage::IsReady => {
                    println!("readyok")
                }
                UciMessage::Position { startpos, fen, moves } => {
                    let mut board = match (startpos, fen) {
                        (true, None) => ChessBoard::default(),
                        (false, Some(fen)) => ChessBoard::new_without_history_fen(fen.as_str(), Rules::default()),
                        _ => panic!("Invalid position command"),
                    };

                    for mv in moves {
                        board.play(mv).unwrap();
                    }

                    writeln!(log, "setting curr_board to {}", board)?;
                    tree = Some(Tree::new(board));
                }
                UciMessage::Go { time_control, search_control } => {
                    if let Some(UciSearchControl { nodes, .. } ) = search_control {
                        max_nodes = nodes.unwrap();
                        start_time = Instant::now();
                        searching = true;
                    } else if let Some(tc) = time_control {
                        if let Some(t) = &tree {
                            alloc = match tc {
                                UciTimeControl::Infinite => i64::MAX,
                                UciTimeControl::MoveTime(x) => x.num_milliseconds(),
                                UciTimeControl::TimeLeft { white_time, black_time, white_increment, black_increment, moves_to_go } => {
                                    let mtg = i64::from(moves_to_go.unwrap_or(25));
                                    let (remaining, inc) = match t.root_board().next_player() {
                                        Player::A => (white_time, white_increment),
                                        Player::B => (black_time, black_increment),
                                    };
                                    let remaining = remaining.unwrap().num_milliseconds();
                                    let inc = if let Some(i) = inc {i.num_milliseconds()} else {0};

                                    let base = remaining / mtg + 3 * inc / 4;
                                    (base.min(remaining) - 5).max(10)
                                },
                                UciTimeControl::Ponder => unimplemented!(),
                            };

                            start_time = Instant::now();
                            searching = true;
                        } else {
                            println!("info string error no position set!");
                        }
                    }
                }
                UciMessage::Stop => {
                    searching = false;

                    if let Some(tree) = &tree {
                        let best_move = if tree.root_visits() > 0 {
                            tree.best_move().unwrap()
                        } else {
                            tree.root_board().random_available_move(&mut thread_rng()).unwrap()
                        };

                        println!("bestmove {}", best_move);
                    }
                }
                UciMessage::UciNewGame => {
                    tree = None;
                }
                UciMessage::Quit => {
                    return Ok(());
                }
                UciMessage::Debug(_) => {}
                UciMessage::Register { .. } => {}
                UciMessage::SetOption { .. } => {}
                UciMessage::PonderHit => {}
                UciMessage::Id { .. } => {}
                UciMessage::UciOk => {}
                UciMessage::ReadyOk => {}
                UciMessage::BestMove { .. } => {}
                UciMessage::CopyProtection(_) => {}
                UciMessage::Registration(_) => {}
                UciMessage::Option(_) => {}
                UciMessage::Info(_) => {}
                UciMessage::Unknown(_, _) => {}
            }
        }
    }
}

fn receive<T>(receiver: &Receiver<T>, blocking: bool) -> Result<Option<T>, RecvError> {
    if blocking {
        receiver.recv().map(Some)
    } else {
        match receiver.try_recv() {
            Ok(value) => Ok(Some(value)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => Err(RecvError::Disconnected),
        }
    }
}

fn io_thread(sender: Sender<UciMessage>) -> std::io::Result<()> {
    // io
    let stdin = std::io::stdin();
    let mut stdin = BufReader::new(stdin.lock());

    // message loop
    let mut line = String::new();

    loop {
        stdin.read_line(&mut line)?;
        let message = vampirc_uci::parse_one(&line);
        sender.send(message).unwrap();
        line.clear();
    }
}
