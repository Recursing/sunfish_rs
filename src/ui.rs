use crate::board::{static_score, BoardState, A1, A8, BOARD_SIDE, PADDING};
use crate::pieces::{Piece, Square};
use std::collections::HashMap;

lazy_static! {
    static ref UNI_PIECES: HashMap<Square, char> = {
        let mut h = HashMap::new();
        h.insert(Square::MyPiece(Piece::Rook), '♜');
        h.insert(Square::MyPiece(Piece::Knight), '♞');
        h.insert(Square::MyPiece(Piece::Bishop), '♝');
        h.insert(Square::MyPiece(Piece::Queen), '♛');
        h.insert(Square::MyPiece(Piece::King), '♚');
        h.insert(Square::MyPiece(Piece::Pawn), '♟');
        h.insert(Square::OpponentPiece(Piece::Rook), '♖');
        h.insert(Square::OpponentPiece(Piece::Knight), '♘');
        h.insert(Square::OpponentPiece(Piece::Bishop), '♗');
        h.insert(Square::OpponentPiece(Piece::Queen), '♕');
        h.insert(Square::OpponentPiece(Piece::King), '♔');
        h.insert(Square::OpponentPiece(Piece::Pawn), '♙');
        h.insert(Square::Empty, '·');
        h
    };
}

pub fn print_board(board_state: &BoardState) {
    for (i, row) in board_state
        .board
        .chunks(BOARD_SIDE)
        .skip(PADDING)
        .take(8)
        .enumerate()
    {
        print!(" {} ", 8 - i);
        for p in row.iter().skip(PADDING).take(8) {
            print!(" {}", UNI_PIECES.get(p).unwrap());
        }
        print!("\n");
    }
    println!("    a b c d e f g h \n\n");
    println!("Static score: {}", board_state.score);
    if static_score(board_state) != board_state.score {
        println!(
            "STATIC SCORE ERROR, SHOULD BE: {}",
            static_score(board_state)
        );
    }
    if board_state.en_passant_position.is_some() {
        println!("En passant is {:?}", board_state.en_passant_position);
    }
    println!(
        "Castling rights are {:?} {:?}",
        board_state.my_castling_rights, board_state.opponent_castling_rights
    )
}

pub fn parse_move(move_: &str) -> (usize, usize) {
    let from = parse_coordinates(&move_[..2]);
    let to = parse_coordinates(&move_[2..]);
    (from, to)
}

fn parse_coordinates(coordinates: &str) -> usize {
    let mut chars = coordinates.chars();
    let file = chars.next().expect("Failed to parse coordinates");
    let rank = chars.next().expect("Failed to parse coordinates");
    return A1 + (file as i32 - 'a' as i32) as usize
        - BOARD_SIDE as usize * ((rank as i32 - '1' as i32) as usize);
}

pub fn render(move_: usize) -> String {
    let rank = b'8' - ((move_ - A8) as u8 / BOARD_SIDE as u8);
    let file = (move_ - A8) as u8 % BOARD_SIDE as u8 + b'a';
    return vec![file as char, rank as char].into_iter().collect();
}
