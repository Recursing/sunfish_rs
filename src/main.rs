#[macro_use]
extern crate lazy_static;
//use std::time::Duration;

mod board;
mod pieces;
mod search;
mod uci;
mod ui;
//use crate::board::{after_move, gen_moves, rotated, BOARD_SIZE, INITIAL_BOARD_STATE};
//use crate::search::{Searcher, MATE_LOWER, MATE_UPPER};
use crate::uci::uci_loop;
//use crate::ui::{parse_move, print_board, render};

fn main() {
    uci_loop();
    /*let mut board_state = INITIAL_BOARD_STATE;
    let mut searcher = Searcher::new();
    loop {
        print_board(&board_state);

        if board_state.score <= -MATE_LOWER {
            println!("You lost");
            break;
        }

        // We query the user until she enters a (pseudo) legal move.
        let mut mov = (0, 0);
        let available = gen_moves(&board_state);
        while !available.contains(&mov) {
            println!("Your move: ");
            let mut line = String::new();
            std::io::stdin().read_line(&mut line).unwrap();
            line.pop(); // \n
            if line.len() != 4 {
                continue;
            }
            mov = parse_move(&line);
            println!("Parsed move {:?}", mov);
        }
        board_state = after_move(&board_state, &mov);

        // After our move we rotate the board and print it again.
        // This allows us to see the effect of our move.
        print_board(&rotated(&board_state));

        if board_state.score <= -MATE_LOWER {
            println!("You won");
            break;
        }

        // Fire up the engine to look for a move.
        let (mov, score) = searcher.search(board_state.clone(), Duration::new(10, 0));
        println!("{:?} {}", mov, score);
        if score == MATE_UPPER {
            println!("Checkmate!");
        }

        // The black player moves from a rotated position, so we have to
        // 'back rotate' the move before printing it.
        println!(
            "My move: {}{}",
            render(BOARD_SIZE - 1 - mov.0),
            render(BOARD_SIZE - 1 - mov.1)
        );

        board_state = after_move(&board_state, &mov);
    }*/
}
