use log::{info, trace, warn};
use std::collections::HashMap;
use std::time::Duration;

use crate::board::{A8, BOARD_SIZE, H8, INITIAL_BOARD_STATE, after_move, gen_moves};
use crate::pieces::Square;
use crate::search::Searcher;
use crate::ui::{parse_move, render_move};

fn read_line() -> String {
    let mut line = String::new();
    std::io::stdin().read_line(&mut line).unwrap();
    line.pop();
    line
}

fn move_time(command: &str, am_black: bool) -> Duration {
    let mut values = HashMap::new();
    let mut parameters = command.split_whitespace().skip(1);
    while let (Some(name), Some(value)) = (parameters.next(), parameters.next()) {
        values.insert(name, value.parse::<u64>().unwrap());
    }
    let (remaining_name, increment_name) = if am_black {
        ("btime", "binc")
    } else {
        ("wtime", "winc")
    };
    let remaining = *values.get(remaining_name).unwrap_or(&60_000);
    let increment = *values.get(increment_name).unwrap_or(&0);
    let milliseconds = values.get("movetime").copied().unwrap_or_else(|| {
        (remaining / 9 + increment * 9 / 10).min(remaining.saturating_sub(2_000) / 2)
    });
    Duration::from_millis(milliseconds.max(50))
}

pub fn uci_loop() {
    println!("Sunfish_rs");
    let mut board_state = INITIAL_BOARD_STATE;
    let mut history = Vec::new();
    let mut am_black = false;
    loop {
        let next_command = read_line();
        trace!("Received command {}", next_command);
        match next_command.split(' ').next().unwrap() {
            "quit" => return,
            "uci" => println!("uciok"),
            "isready" => println!("readyok"),
            "ucinewgame" => {
                board_state = INITIAL_BOARD_STATE;
                history.clear();
            }
            "position" => {
                //position startpos moves d2d4 d7d5 e2e4 d5e4
                info!("loading moves");
                let command_parts: Vec<&str> = next_command.split(' ').collect();
                if (command_parts.len() == 2 && command_parts[1] != "startpos")
                    || (command_parts.len() > 2
                        && (command_parts[0] != "position"
                            || command_parts[1] != "startpos"
                            || command_parts[2] != "moves"))
                {
                    warn!("UNKNOWN FORMAT!");
                    panic!();
                }
                board_state = INITIAL_BOARD_STATE;
                history.clear();
                am_black = false;
                for move_text in command_parts.iter().skip(3) {
                    let mut parsed_move = parse_move(move_text);
                    if am_black {
                        parsed_move.0 = BOARD_SIZE - 1 - parsed_move.0;
                        parsed_move.1 = BOARD_SIZE - 1 - parsed_move.1;
                    };
                    if !gen_moves(&board_state).contains(&parsed_move) {
                        warn!(
                            "Trying to make an illegal move {:?}, will probably fail",
                            parsed_move
                        );
                    }
                    board_state = after_move(&board_state, &parsed_move);
                    history.push(board_state);
                    am_black = !am_black;
                }
                // print_board(&board_state);
            }
            "go" => {
                let time_for_move = move_time(&next_command, am_black);
                info!("Computing move for {:?}", time_for_move);
                let mut searcher = Searcher::default();
                for position in &history {
                    searcher.mark_as_draw(position);
                }
                let (mut top_move, score, depth) = searcher.search(board_state, time_for_move);
                let is_promotion = (A8 <= top_move.1 && top_move.1 <= H8)
                    && board_state.board[top_move.0] == Square::MyPawn;
                if am_black {
                    top_move.0 = BOARD_SIZE - 1 - top_move.0;
                    top_move.1 = BOARD_SIZE - 1 - top_move.1;
                };
                if is_promotion {
                    println!("bestmove {}q ponder e7e5", render_move(&top_move));
                } else {
                    println!("bestmove {} ponder e7e5", render_move(&top_move));
                }
                info!("Sending bestmove {}", render_move(&top_move));
                info!(
                    "Searched {} nodes, reached depth {}, estimate score {}, tables at {} and {}",
                    searcher.nodes,
                    depth,
                    score,
                    searcher.move_transposition_table.len(),
                    searcher.score_transposition_table.len()
                );
            }
            _ => {
                warn!("UNKNOWN COMMAND {}", next_command);
                println!("Unknown command:{}", next_command);
            }
        }
    }
}
