use std::collections::HashMap;
extern crate rand;
use crate::board::{BOARD_SIDE, BOARD_SIZE, PADDING};
use rand::random;

pub struct Direction {}

impl Direction {
    pub const NORTH: i32 = -(BOARD_SIDE as i32);
    pub const EAST: i32 = 1;
    pub const SOUTH: i32 = BOARD_SIDE as i32;
    pub const WEST: i32 = -1;
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Square {
    MyPiece(Piece),
    OpponentPiece(Piece),
    Empty,
    Wall, // Here to simplify detection of out of board moves
}

lazy_static! {
    pub static ref PIECE_MOVES: HashMap<Piece, Vec<i32>> = get_piece_moves();
    pub static ref PIECE_SQUARE_TABLES: HashMap<Piece, [i32; BOARD_SIZE]> =
        get_piece_square_tables();
    pub static ref ZOBRIST_MAP: HashMap<(usize, Square), u64> = get_zobrist_map();
}

pub fn get_piece_moves() -> HashMap<Piece, Vec<i32>> {
    let mut temp_piece_moves: HashMap<Piece, Vec<i32>> = HashMap::new();
    // TODO would like to use fixed sized slices/tuples
    temp_piece_moves.insert(
        Piece::Pawn,
        vec![
            Direction::NORTH,
            Direction::NORTH + Direction::NORTH,
            Direction::NORTH + Direction::WEST,
            Direction::NORTH + Direction::EAST,
        ],
    );
    temp_piece_moves.insert(
        Piece::Knight,
        vec![
            Direction::NORTH + Direction::NORTH + Direction::EAST,
            Direction::NORTH + Direction::NORTH + Direction::WEST,
            Direction::WEST + Direction::WEST + Direction::NORTH,
            Direction::WEST + Direction::WEST + Direction::SOUTH,
            Direction::SOUTH + Direction::SOUTH + Direction::WEST,
            Direction::SOUTH + Direction::SOUTH + Direction::EAST,
            Direction::EAST + Direction::EAST + Direction::SOUTH,
            Direction::EAST + Direction::EAST + Direction::NORTH,
        ],
    );
    temp_piece_moves.insert(
        Piece::Bishop,
        vec![
            Direction::NORTH + Direction::EAST,
            Direction::NORTH + Direction::WEST,
            Direction::WEST + Direction::SOUTH,
            Direction::SOUTH + Direction::EAST,
        ],
    );
    temp_piece_moves.insert(
        Piece::Rook,
        vec![
            Direction::NORTH,
            Direction::WEST,
            Direction::SOUTH,
            Direction::EAST,
        ],
    );
    temp_piece_moves.insert(
        Piece::Queen,
        vec![
            Direction::NORTH,
            Direction::WEST,
            Direction::SOUTH,
            Direction::EAST,
            Direction::NORTH + Direction::EAST,
            Direction::NORTH + Direction::WEST,
            Direction::WEST + Direction::SOUTH,
            Direction::SOUTH + Direction::EAST,
        ],
    );
    temp_piece_moves.insert(Piece::King, temp_piece_moves[&Piece::Queen].clone());
    temp_piece_moves
}

pub fn get_piece_square_tables() -> HashMap<Piece, [i32; BOARD_SIZE]> {
    // Piece square tables: piece value in different positions

    let mut piece_values: HashMap<Piece, i32> = HashMap::new();
    piece_values.insert(Piece::Pawn, 100);
    piece_values.insert(Piece::Knight, 280);
    piece_values.insert(Piece::Bishop, 320);
    piece_values.insert(Piece::Rook, 479);
    piece_values.insert(Piece::Queen, 929);
    piece_values.insert(Piece::King, 60000);

    // TODO: disable rustfmt here
    let mut piece_position_values: HashMap<Piece, [i32; 64]> = HashMap::new();
    piece_position_values.insert(
        Piece::Pawn,
        [
            0, 0, 0, 0, 0, 0, 0, 0, 78, 83, 86, 73, 102, 82, 85, 90, 7, 29, 21, 44, 40, 31, 44, 7,
            -17, 16, -2, 15, 14, 0, 15, -13, -26, 3, 10, 9, 6, 1, 0, -23, -22, 9, 5, -11, -10, -2,
            3, -19, -31, 8, -7, -37, -36, -14, 3, -31, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
    );
    piece_position_values.insert(
        Piece::Knight,
        [
            -66, -53, -75, -75, -10, -55, -58, -70, -3, -6, 100, -36, 4, 62, -4, -14, 10, 67, 1,
            74, 73, 27, 62, -2, 24, 24, 45, 37, 33, 41, 25, 17, -1, 5, 31, 21, 22, 35, 2, 0, -18,
            10, 13, 22, 18, 15, 11, -14, -23, -15, 2, 0, 2, 0, -23, -20, -74, -23, -26, -24, -19,
            -35, -22, -69,
        ],
    );
    piece_position_values.insert(
        Piece::Bishop,
        [
            -59, -78, -82, -76, -23, -107, -37, -50, -11, 20, 35, -42, -39, 31, 2, -22, -9, 39,
            -32, 41, 52, -10, 28, -14, 25, 17, 20, 34, 26, 25, 15, 10, 13, 10, 17, 23, 17, 16, 0,
            7, 14, 25, 24, 15, 8, 25, 20, 15, 19, 20, 11, 6, 7, 6, 20, 16, -7, 2, -15, -12, -14,
            -15, -10, -10,
        ],
    );
    piece_position_values.insert(
        Piece::Rook,
        [
            35, 29, 33, 4, 37, 33, 56, 50, 55, 29, 56, 67, 55, 62, 34, 60, 19, 35, 28, 33, 45, 27,
            25, 15, 0, 5, 16, 13, 18, -4, -9, -6, -28, -35, -16, -21, -13, -29, -46, -30, -42, -28,
            -42, -25, -25, -35, -26, -46, -53, -38, -31, -26, -29, -43, -44, -53, -30, -24, -18, 5,
            -2, -18, -31, -32,
        ],
    );
    piece_position_values.insert(
        Piece::Queen,
        [
            6, 1, -8, -104, 69, 24, 88, 26, 14, 32, 60, -10, 20, 76, 57, 24, -2, 43, 32, 60, 72,
            63, 43, 2, 1, -16, 22, 17, 25, 20, -13, -6, -14, -15, -2, -5, -1, -10, -20, -22, -30,
            -6, -13, -11, -16, -11, -16, -27, -36, -18, 0, -19, -15, -15, -21, -38, -39, -30, -31,
            -13, -13, -36, -34, -42,
        ],
    );
    piece_position_values.insert(
        Piece::King,
        [
            4, 54, 47, -99, -99, 60, 83, -62, -32, 10, 55, 56, 56, 55, 10, 3, -62, 12, -57, 44,
            -67, 28, 37, -31, -55, 50, 11, -4, -19, 13, 0, -49, -55, -43, -52, -28, -51, -47, -8,
            -50, -47, -42, -43, -79, -64, -32, -29, -32, -4, 3, -14, -50, -57, -18, 13, 4, 20, 50,
            -1, -14, -14, -1, 50, 18,
        ],
    );

    // TODO link to piece square tables, add explanation
    let mut temp_piece_square_tables: HashMap<Piece, [i32; BOARD_SIZE]> = HashMap::new();

    for (piece, position_values) in piece_position_values {
        let piece_value = piece_values[&piece];
        let mut piece_square_table = [0; BOARD_SIZE];

        for position in 0..(PADDING * BOARD_SIDE) {
            piece_square_table[position] = 0;
            piece_square_table[BOARD_SIZE - position - 1] = 0;
        }
        for rank in 0..8 {
            let first_of_rank = (rank + PADDING) * BOARD_SIDE;
            for pad in 0..PADDING {
                piece_square_table[first_of_rank + pad] = 0;
                piece_square_table[first_of_rank + BOARD_SIDE - pad - 1] = 0;
            }
            for file in 0..8 {
                piece_square_table[first_of_rank + PADDING + file] =
                    position_values[rank * 8 + file] + piece_value;
            }
        }
        temp_piece_square_tables.insert(piece, piece_square_table);
    }

    temp_piece_square_tables
}

pub fn get_zobrist_map() -> HashMap<(usize, Square), u64> {
    let mut zobrist_map_temp: HashMap<(usize, Square), u64> = HashMap::new();
    for position in 0..BOARD_SIZE {
        for (piece, _moves) in get_piece_moves() {
            zobrist_map_temp.insert((position, Square::MyPiece(piece)), random());
        }
        for (piece, _moves) in get_piece_moves() {
            zobrist_map_temp.insert((position, Square::OpponentPiece(piece)), random());
        }
        // Used for en passant, castling and king passant
        zobrist_map_temp.insert((position, Square::Empty), random());
    }
    zobrist_map_temp
}
