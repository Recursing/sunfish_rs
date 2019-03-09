use std::fs::File;
use std::io::Write;
use std::time::Duration;

use crate::board::{after_move, gen_moves, A8, BOARD_SIZE, H8, INITIAL_BOARD_STATE};
use crate::pieces::{Piece, Square};
use crate::search::Searcher;
use crate::ui::{parse_move, print_board, render};

fn read_line() -> String {
    let mut line = String::new();
    std::io::stdin().read_line(&mut line).unwrap();
    line.pop();
    line
}

pub fn uci_loop() {
    let mut searcher = Searcher::new();
    let mut debug_file = File::create("debug_log").expect("Unable to create file!");
    println!("Sunfish_rs");
    let mut board_state = INITIAL_BOARD_STATE;
    let mut rotated = false;
    loop {
        let next_command = read_line();
        writeln!(debug_file, "Received command {}", next_command);
        match next_command.split(" ").next().unwrap() {
            "quit" => return,
            "uci" => println!("uciok"),
            "isready" => println!("readyok"),
            "ucinewgame" => board_state = INITIAL_BOARD_STATE,
            "position" => {
                //position startpos moves d2d4 d7d5 e2e4 d5e4
                writeln!(debug_file, "loading moves");
                let moves: Vec<&str> = next_command.split(" ").collect();
                if moves.len() == 2 && moves[1] != "startpos" {
                    writeln!(debug_file, "UNKNOWN FORMAT!");
                    panic!();
                } else if moves[0] != "position" || moves[1] != "startpos" || moves[2] != "moves" {
                    writeln!(debug_file, "UNKNOWN FORMAT!");
                    panic!();
                }
                board_state = INITIAL_BOARD_STATE;
                rotated = false;
                for move_ in moves.iter().skip(3) {
                    let mut parsed_move = parse_move(move_);
                    if rotated {
                        parsed_move.0 = BOARD_SIZE - 1 - parsed_move.0;
                        parsed_move.1 = BOARD_SIZE - 1 - parsed_move.1;
                    };
                    if !gen_moves(&board_state).contains(&parsed_move) {
                        writeln!(
                            debug_file,
                            "Trying to make an illegal move {:?}, will probably fail",
                            parsed_move
                        );
                    }
                    board_state = after_move(&board_state, &parsed_move);
                    rotated = !rotated;
                }
            }
            "go" => {
                writeln!(debug_file, "Computing move!");
                // TODO parse_movetime
                let (mut top_move, _score) =
                    searcher.search(board_state.clone(), Duration::new(1, 0));
                let is_promotion = (A8 <= top_move.1 && top_move.1 <= H8)
                    && board_state.board[top_move.0] == Square::MyPiece(Piece::Pawn);
                if rotated {
                    top_move.0 = BOARD_SIZE - 1 - top_move.0;
                    top_move.1 = BOARD_SIZE - 1 - top_move.1;
                };
                if is_promotion {
                    println!(
                        "bestmove {}{}q ponder e7e5",
                        render(top_move.0),
                        render(top_move.1)
                    );
                } else {
                    println!(
                        "bestmove {}{} ponder e7e5",
                        render(top_move.0),
                        render(top_move.1)
                    );
                }
                writeln!(
                    debug_file,
                    "Sending bestmove {}{}",
                    render(top_move.0),
                    render(top_move.1)
                );
            }
            _ => {
                writeln!(debug_file, "UNKNOWN COMMAND {}", next_command);
                println!("Unknown command:{}", next_command);
            }
        }
    }
}
