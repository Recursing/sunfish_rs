use crate::pieces::{Direction, Square};
use std::fmt::Debug;

pub const PADDING: usize = 2;
pub const BOARD_SIDE: usize = 8 + 2 * PADDING;
pub const BOARD_SIZE: usize = BOARD_SIDE * BOARD_SIDE;

pub const A8: usize = BOARD_SIDE * PADDING + PADDING;
pub const H8: usize = A8 + 7;
pub const A1: usize = A8 + 7 * BOARD_SIDE;
const H1: usize = A1 + 7;

// A move is a pair of indexes into the padded mailbox board.
pub type ChessMove = (usize, usize);

// The board is always shown from the side to move. Its pieces are "my"
// pieces and move north. after_move rotates the position for the next player.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct BoardState {
    pub board: [Square; BOARD_SIZE],
    pub score: i32,
    pub my_castling_rights: (bool, bool),       // west, east
    pub opponent_castling_rights: (bool, bool), // west, east
    pub en_passant_position: Option<usize>,
    // Treat castling like en passant so crossing an attacked square is illegal.
    pub king_passant_position: Option<usize>,
}

// Generate a deterministic pseudo-random number for each Zobrist table entry.
const fn zobrist_value(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e3779b97f4a7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d049bb133111eb);
    value ^ (value >> 31)
}

const PIECE_CODE_COUNT: usize = Square::OpponentKing as usize + 1;

static ZOBRIST_SQUARES: [u64; BOARD_SIZE * PIECE_CODE_COUNT] = {
    let mut values = [0; BOARD_SIZE * PIECE_CODE_COUNT];
    let mut index = 0;
    while index < values.len() {
        values[index] = zobrist_value(index as u64);
        index += 1;
    }
    values
};

pub fn zobrist_hash(board_state: &BoardState) -> u64 {
    let mut hash = 0;
    for (position, &square) in board_state.board[A8..=H1].iter().enumerate() {
        if square.is_my_piece() || square.is_opponent_piece() {
            hash ^= ZOBRIST_SQUARES[(position + A8) * PIECE_CODE_COUNT + square as usize];
        }
    }
    let castling = board_state.my_castling_rights.0 as u64
        | (board_state.my_castling_rights.1 as u64) << 1
        | (board_state.opponent_castling_rights.0 as u64) << 2
        | (board_state.opponent_castling_rights.1 as u64) << 3;
    hash ^ zobrist_value(0x10000 | board_state.score as u32 as u64)
        ^ zobrist_value(0x20000 | castling)
        ^ zobrist_value(0x30000 | board_state.en_passant_position.unwrap_or(BOARD_SIZE) as u64)
        ^ zobrist_value(0x40000 | board_state.king_passant_position.unwrap_or(BOARD_SIZE) as u64)
}

fn visit_piece_moves(
    board_state: &BoardState,
    moving_piece: Square,
    start_position: usize,
    mut visitor: impl FnMut(usize) -> bool,
) -> bool {
    for move_direction in moving_piece.move_directions() {
        for step in 1.. {
            let end_position = (start_position as i32 + move_direction * step) as usize;
            let destination_square = board_state.board[end_position];

            // The padded wall removes the need for coordinate bounds checks.
            if destination_square == Square::Wall || destination_square.is_my_piece() {
                break;
            }

            if moving_piece == Square::MyPawn {
                // Pawns move forward only into empty squares.
                if (*move_direction == Direction::NORTH
                    || *move_direction == Direction::NORTH + Direction::NORTH)
                    && destination_square != Square::Empty
                {
                    break;
                }
                // Diagonal pawn moves must capture a piece or an en passant target.
                if (*move_direction == Direction::NORTH + Direction::WEST
                    || *move_direction == Direction::NORTH + Direction::EAST)
                    && destination_square == Square::Empty
                    && board_state.en_passant_position != Some(end_position)
                    && board_state.king_passant_position != Some(end_position)
                {
                    break;
                }
                // A double move starts on the second rank and cannot jump a piece.
                if *move_direction == Direction::NORTH + Direction::NORTH
                    && (start_position < (A1 as i32 + Direction::NORTH) as usize
                        || board_state.board[(start_position as i32 + Direction::NORTH) as usize]
                            != Square::Empty)
                {
                    break;
                }
            }

            // Emit pseudo-legal moves. The search handles king safety.
            if visitor(end_position) {
                return true;
            }

            // Stop pieces that don't slide
            if moving_piece == Square::MyPawn
                || moving_piece == Square::MyKnight
                || moving_piece == Square::MyKing
            {
                break;
            }

            // Stop sliding after capture
            if destination_square != Square::Empty {
                break;
            }
        }
    }
    false
}

pub fn gen_moves(board_state: &BoardState) -> Vec<ChessMove> {
    let mut generated_moves = Vec::with_capacity(42);
    for (start_position, start_square) in board_state.board[A8..=H1].iter().enumerate() {
        let start_position = start_position + A8;
        if !start_square.is_my_piece() {
            continue;
        }
        visit_piece_moves(board_state, *start_square, start_position, |end_position| {
            generated_moves.push((start_position, end_position));
            // A rook that can reach its king also generates the castling move.
            if start_position == A1
                && board_state.board[(end_position as i32 + Direction::EAST) as usize]
                    == Square::MyKing
                && board_state.my_castling_rights.0
            {
                generated_moves.push((
                    (end_position as i32 + Direction::EAST) as usize,
                    (end_position as i32 + Direction::WEST) as usize,
                ))
            } else if start_position == H1
                && board_state.board[(end_position as i32 + Direction::WEST) as usize]
                    == Square::MyKing
                && board_state.my_castling_rights.1
            {
                generated_moves.push((
                    (end_position as i32 + Direction::WEST) as usize,
                    (end_position as i32 + Direction::EAST) as usize,
                ))
            }
            false
        });
    }
    generated_moves
}

pub fn rotate(board_state: &mut BoardState) {
    let total_padding = PADDING * BOARD_SIDE + PADDING;
    for coordinate in total_padding..(BOARD_SIZE / 2) {
        let old_val = board_state.board[coordinate];
        board_state.board[coordinate] = board_state.board[BOARD_SIZE - 1 - coordinate].swap_color();
        board_state.board[BOARD_SIZE - 1 - coordinate] = old_val.swap_color();
    }
    board_state.score = -board_state.score;
    std::mem::swap(
        &mut board_state.my_castling_rights,
        &mut board_state.opponent_castling_rights,
    );
    board_state.en_passant_position = board_state
        .en_passant_position
        .map(|position| BOARD_SIZE - 1 - position);
    board_state.king_passant_position = board_state
        .king_passant_position
        .map(|position| BOARD_SIZE - 1 - position);
}

// Pass the turn without moving. En passant rights last for only one turn.
pub fn nullmove(board_state: &BoardState) -> BoardState {
    let mut new_board_state = *board_state;
    rotate(&mut new_board_state);
    new_board_state.en_passant_position = None;
    new_board_state.king_passant_position = None;
    new_board_state
}

pub fn after_move(board_state: &BoardState, chess_move: &ChessMove) -> BoardState {
    let (start_position, end_position) = *chess_move;
    let start_square = board_state.board[start_position];
    let mut new_board = board_state.board;
    let mut my_castling_rights = board_state.my_castling_rights;
    let mut opponent_castling_rights = board_state.opponent_castling_rights;
    let mut en_passant_position = None;
    let mut king_passant_position = None;

    // Actual move
    new_board[end_position] = start_square;
    new_board[start_position] = Square::Empty;

    // Castling rights, we move the rook or capture the opponent's
    if start_position == A1 {
        my_castling_rights = (false, my_castling_rights.1)
    }
    if start_position == H1 {
        my_castling_rights = (my_castling_rights.0, false)
    }
    if end_position == A8 {
        opponent_castling_rights = (opponent_castling_rights.0, false)
    }
    if end_position == H8 {
        opponent_castling_rights = (false, opponent_castling_rights.1)
    }

    // Castling
    if start_square == Square::MyKing {
        my_castling_rights = (false, false);
        if (start_position as i32 - end_position as i32).abs() == 2 {
            let final_rook_position: usize = (start_position + end_position) / 2;
            new_board[final_rook_position] = Square::MyRook;
            king_passant_position = Some(final_rook_position);
            if start_position > end_position {
                new_board[A1] = Square::Empty;
            } else {
                new_board[H1] = Square::Empty;
            }
        }
    }

    // Pawn promotion, double move and en passant capture
    if start_square == Square::MyPawn {
        let displacement = end_position as i32 - start_position as i32;
        if (A8..=H8).contains(&end_position) {
            new_board[end_position] = Square::MyQueen
        } else if displacement == 2 * Direction::NORTH {
            en_passant_position = Some((start_position as i32 + Direction::NORTH) as usize)
        }

        // An en passant capture moves diagonally to an empty square.
        if board_state.en_passant_position == Some(end_position) {
            new_board[end_position + Direction::SOUTH as usize] = Square::Empty;
        }
    }

    let mut new_board_state = BoardState {
        board: new_board,
        score: board_state.score + move_value(board_state, chess_move),
        my_castling_rights,
        opponent_castling_rights,
        king_passant_position,
        en_passant_position,
    };
    rotate(&mut new_board_state);
    new_board_state
}

pub fn move_value(board_state: &BoardState, chess_move: &ChessMove) -> i32 {
    let (start_position, end_position) = *chess_move;
    let moving_piece = board_state.board[start_position];
    if !moving_piece.is_my_piece() {
        panic!();
    }

    // Actual move
    let mut score_delta =
        moving_piece.midgame_value(end_position) - moving_piece.midgame_value(start_position);

    // Score for captures
    if board_state.board[end_position].is_opponent_piece() {
        // Add to the board score the value of the captured piece in the rotated board
        score_delta += board_state.board[end_position]
            .swap_color()
            .midgame_value(BOARD_SIZE - 1 - end_position);
    }

    // Castling check detection
    if let Some(position) = board_state.king_passant_position
        && (end_position as i32 - position as i32).abs() < 2
    {
        // If I'm moving to a position the opponent king just passed through while castling, I can capture it
        // E.g. any of E1, F1, G1 for white short castling, the king_passant_position would be F1
        score_delta += Square::MyKing.midgame_value(BOARD_SIZE - 1 - end_position);
    }

    // Score the special effects of king and pawn moves.
    match moving_piece {
        Square::MyKing => {
            // Castling, update the score with the new rook position
            if (end_position as i32 - start_position as i32).abs() == 2 {
                score_delta += Square::MyRook.midgame_value((start_position + end_position) / 2);
                score_delta -= Square::MyRook.midgame_value(if end_position < start_position {
                    A1
                } else {
                    H1
                });
            } else if board_state.my_castling_rights.0 || board_state.my_castling_rights.1 {
                score_delta -= 70;
            }
        }
        Square::MyPawn => {
            let second_rank = A1 - BOARD_SIDE;
            let opens_king = (board_state.board[A1 + 6] == Square::MyKing
                && (second_rank + 5..=second_rank + 7).contains(&start_position))
                || (board_state.board[A1 + 2] == Square::MyKing
                    && (second_rank..=second_rank + 2).contains(&start_position));
            if opens_king {
                score_delta -= 40;
            }
            if (A8..=H8).contains(&end_position) {
                //Promotion
                score_delta += Square::MyQueen.midgame_value(end_position)
                    - Square::MyPawn.midgame_value(end_position) //Always promote to queen
            } else if board_state.en_passant_position == Some(end_position) {
                // The captured pawn sits one rank behind the destination.
                score_delta +=
                    Square::MyPawn.midgame_value(BOARD_SIZE - 1 - (end_position + BOARD_SIDE))
            }
        }
        _ => {}
    }
    score_delta
}

pub fn static_score(board: [Square; BOARD_SIZE]) -> i32 {
    board
        .iter()
        .enumerate()
        .map(|(index, piece)| {
            if piece.is_my_piece() {
                piece.midgame_value(index)
            } else if piece.is_opponent_piece() {
                -piece.swap_color().midgame_value(BOARD_SIZE - 1 - index)
            } else {
                0
            }
        })
        .sum()
}

const INITIAL_BOARD: [Square; BOARD_SIZE] = [
    // Padding
    Square::Wall,
    Square::Wall,
    Square::Wall,
    Square::Wall,
    Square::Wall,
    Square::Wall,
    Square::Wall,
    Square::Wall,
    Square::Wall,
    Square::Wall,
    Square::Wall,
    Square::Wall,
    Square::Wall,
    Square::Wall,
    Square::Wall,
    Square::Wall,
    Square::Wall,
    Square::Wall,
    Square::Wall,
    Square::Wall,
    Square::Wall,
    Square::Wall,
    Square::Wall,
    Square::Wall,
    // Eigth rank
    Square::Wall,
    Square::Wall,
    Square::OpponentRook,
    Square::OpponentKnight,
    Square::OpponentBishop,
    Square::OpponentQueen,
    Square::OpponentKing,
    Square::OpponentBishop,
    Square::OpponentKnight,
    Square::OpponentRook,
    Square::Wall,
    Square::Wall,
    // Seventh rank
    Square::Wall,
    Square::Wall,
    Square::OpponentPawn,
    Square::OpponentPawn,
    Square::OpponentPawn,
    Square::OpponentPawn,
    Square::OpponentPawn,
    Square::OpponentPawn,
    Square::OpponentPawn,
    Square::OpponentPawn,
    Square::Wall,
    Square::Wall,
    // Sixth rank
    Square::Wall,
    Square::Wall,
    Square::Empty,
    Square::Empty,
    Square::Empty,
    Square::Empty,
    Square::Empty,
    Square::Empty,
    Square::Empty,
    Square::Empty,
    Square::Wall,
    Square::Wall,
    // Fifth rank
    Square::Wall,
    Square::Wall,
    Square::Empty,
    Square::Empty,
    Square::Empty,
    Square::Empty,
    Square::Empty,
    Square::Empty,
    Square::Empty,
    Square::Empty,
    Square::Wall,
    Square::Wall,
    // Fourth rank
    Square::Wall,
    Square::Wall,
    Square::Empty,
    Square::Empty,
    Square::Empty,
    Square::Empty,
    Square::Empty,
    Square::Empty,
    Square::Empty,
    Square::Empty,
    Square::Wall,
    Square::Wall,
    // Third rank
    Square::Wall,
    Square::Wall,
    Square::Empty,
    Square::Empty,
    Square::Empty,
    Square::Empty,
    Square::Empty,
    Square::Empty,
    Square::Empty,
    Square::Empty,
    Square::Wall,
    Square::Wall,
    // Second rank
    Square::Wall,
    Square::Wall,
    Square::MyPawn,
    Square::MyPawn,
    Square::MyPawn,
    Square::MyPawn,
    Square::MyPawn,
    Square::MyPawn,
    Square::MyPawn,
    Square::MyPawn,
    Square::Wall,
    Square::Wall,
    // First rank
    Square::Wall,
    Square::Wall,
    Square::MyRook,
    Square::MyKnight,
    Square::MyBishop,
    Square::MyQueen,
    Square::MyKing,
    Square::MyBishop,
    Square::MyKnight,
    Square::MyRook,
    Square::Wall,
    Square::Wall,
    // Padding
    Square::Wall,
    Square::Wall,
    Square::Wall,
    Square::Wall,
    Square::Wall,
    Square::Wall,
    Square::Wall,
    Square::Wall,
    Square::Wall,
    Square::Wall,
    Square::Wall,
    Square::Wall,
    Square::Wall,
    Square::Wall,
    Square::Wall,
    Square::Wall,
    Square::Wall,
    Square::Wall,
    Square::Wall,
    Square::Wall,
    Square::Wall,
    Square::Wall,
    Square::Wall,
    Square::Wall,
];

pub const INITIAL_BOARD_STATE: BoardState = BoardState {
    board: INITIAL_BOARD,
    score: 38,
    my_castling_rights: (true, true),
    opponent_castling_rights: (true, true),
    en_passant_position: None,
    king_passant_position: None,
};
