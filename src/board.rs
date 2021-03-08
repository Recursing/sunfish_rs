use crate::pieces::{Direction, Square};
use std::fmt::Debug;

pub const PADDING: usize = 2;
pub const BOARD_SIDE: usize = 8 + 2 * PADDING;
pub const BOARD_SIZE: usize = BOARD_SIDE * BOARD_SIDE;

pub const A8: usize = BOARD_SIDE * PADDING + PADDING;
pub const H8: usize = A8 + 7;
pub const A1: usize = A8 + 7 * BOARD_SIDE;
const H1: usize = A1 + 7;

#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug)]
pub struct BoardState {
    pub board: [Square; BOARD_SIZE],
    pub score: i32,
    pub my_castling_rights: (bool, bool), // first west, second east
    pub opponent_castling_rights: (bool, bool), // first west, second east
    pub en_passant_position: Option<usize>, // square where I can en passant
    pub king_passant_position: Option<usize>, // square where I could capture the king, used to treat castling as en passant
}

pub fn piece_moves(
    board_state: &BoardState,
    piece_moving: Square,
    start_position: usize,
) -> Vec<usize> {
    let mut reachable_squares: Vec<usize> = Vec::with_capacity(20);
    for move_direction in piece_moving.moves() {
        for end_position in (1..).map(|k| (start_position as i32 + move_direction * k) as usize) {
            let destination_square = board_state.board[end_position];
            // Illegal moves

            // Hit board bounds or one of my pieces
            if destination_square == Square::Wall || destination_square.is_my_piece() {
                break;
            }

            // Illegal pawn moves TODO write explanations
            if piece_moving == Square::MyPawn {
                if (*move_direction == Direction::NORTH
                    || *move_direction == Direction::NORTH + Direction::NORTH)
                    && destination_square != Square::Empty
                {
                    break;
                }
                if (*move_direction == Direction::NORTH + Direction::WEST
                    || *move_direction == Direction::NORTH + Direction::EAST)
                    && destination_square == Square::Empty
                    && board_state.en_passant_position != Some(end_position)
                    && board_state.king_passant_position != Some(end_position)
                {
                    break;
                }
                if *move_direction == Direction::NORTH + Direction::NORTH
                    && (start_position < (A1 as i32 + Direction::NORTH) as usize
                        || board_state.board[(start_position as i32 + Direction::NORTH) as usize]
                            != Square::Empty)
                {
                    break;
                }
            }

            // Move is probably fine (TODO except king stuff)
            reachable_squares.push(end_position);

            // Stop pieces that don't slide
            if piece_moving == Square::MyPawn
                || piece_moving == Square::MyKnight
                || piece_moving == Square::MyKing
            {
                break;
            }

            // Stop sliding after capture
            if destination_square != Square::Empty {
                break;
            }
        }
    }
    reachable_squares
}

pub fn gen_moves(board_state: &BoardState) -> Vec<(usize, usize)> {
    let mut moves: Vec<(usize, usize)> = Vec::with_capacity(42);
    for (start_position, start_square) in board_state.board.iter().enumerate() {
        if !start_square.is_my_piece() {
            continue;
        }
        let piece_moving = start_square;
        for end_position in piece_moves(board_state, *piece_moving, start_position) {
            // Add castling if the rook can move to the king, east castling (long or short depending on color)
            moves.push((start_position, end_position));
            if start_position == A1
                && board_state.board[(end_position as i32 + Direction::EAST) as usize]
                    == Square::MyKing
                && board_state.my_castling_rights.0
            {
                moves.push((
                    (end_position as i32 + Direction::EAST) as usize,
                    (end_position as i32 + Direction::WEST) as usize,
                ))
            }
            // Add castling if the rook can move to the king, west castling (long or short depending on color)
            else if start_position == H1
                && board_state.board[(end_position as i32 + Direction::WEST) as usize]
                    == Square::MyKing
                && board_state.my_castling_rights.1
            {
                moves.push((
                    (end_position as i32 + Direction::WEST) as usize,
                    (end_position as i32 + Direction::EAST) as usize,
                ))
            }
        }
    }
    moves
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
        .map(|ep| BOARD_SIZE - 1 - ep);
    board_state.king_passant_position = board_state
        .king_passant_position
        .map(|kp| BOARD_SIZE - 1 - kp);
}

// Like rotate, but clears ep and kp
pub fn nullmove(board_state: &BoardState) -> BoardState {
    let mut new_board = [Square::Empty; BOARD_SIZE];
    for (coordinate, square) in new_board.iter_mut().enumerate() {
        *square = board_state.board[BOARD_SIZE - 1 - coordinate].swap_color();
    }
    BoardState {
        board: new_board,
        score: -board_state.score,
        my_castling_rights: board_state.opponent_castling_rights,
        opponent_castling_rights: board_state.my_castling_rights,
        en_passant_position: None,
        king_passant_position: None,
    }
}

pub fn after_move(board_state: &BoardState, move_: &(usize, usize)) -> BoardState {
    let (start_position, end_position) = *move_;
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
        let move_type = end_position as i32 - start_position as i32;
        if (A8 <= end_position) && (end_position <= H8) {
            new_board[end_position] = Square::MyQueen
        } else if move_type == 2 * Direction::NORTH {
            en_passant_position = Some((start_position as i32 + Direction::NORTH) as usize)
        }

        // en passant capture (diagonal move to empty position)
        if board_state.en_passant_position == Some(end_position) {
            new_board[end_position + Direction::SOUTH as usize] = Square::Empty;
        }
    }

    let mut new_board_state = BoardState {
        board: new_board,
        score: board_state.score + move_value(board_state, &move_),
        my_castling_rights,
        opponent_castling_rights,
        king_passant_position,
        en_passant_position,
    };
    rotate(&mut new_board_state);
    new_board_state
}

pub fn can_check(board_state: &BoardState, move_: &(usize, usize)) -> bool {
    let (start_position, end_position) = *move_;
    let moved_piece = board_state.board[start_position];
    if !moved_piece.is_my_piece() {
        panic!();
    }
    for reachable_square in piece_moves(board_state, moved_piece, end_position) {
        if board_state.board[reachable_square] == Square::OpponentKing {
            return true;
        }
    }
    false
}

pub fn move_value(board_state: &BoardState, move_: &(usize, usize)) -> i32 {
    let (start_position, end_position) = *move_;
    let moving_piece = board_state.board[start_position];
    if !moving_piece.is_my_piece() {
        panic!();
    }

    // Actual move
    let mut temp_score =
        moving_piece.midgame_value(end_position) - moving_piece.midgame_value(start_position);

    // Score for captures
    if board_state.board[end_position].is_opponent_piece() {
        // Add to the board score the value of the captured piece in the rotated board
        temp_score += board_state.board[end_position]
            .swap_color()
            .midgame_value(BOARD_SIZE - 1 - end_position);
    }

    // Castling check detection
    match board_state.king_passant_position {
        None => {}
        Some(position) => {
            // If I'm moving to a position the opponent king just passed through while castling, I can capture it
            // E.g. any of E1, F1, G1 for white short castling, the king_passant_position would be F1
            if (end_position as i32 - position as i32).abs() < 2 {
                temp_score += Square::MyKing.midgame_value(BOARD_SIZE - 1 - end_position);
            }
        }
    }

    // Wierd pawn and king stuff (castling, promotions and en passant)
    match moving_piece {
        Square::MyKing => {
            // Castling, update the score with the new rook position
            if (end_position as i32 - start_position as i32).abs() == 2 {
                temp_score += Square::MyRook.midgame_value((start_position + end_position) / 2);
                temp_score -= Square::MyRook.midgame_value(if end_position < start_position {
                    A1
                } else {
                    H1
                });
            }
        }
        Square::MyPawn => {
            if A8 <= end_position && end_position <= H8 {
                //Promotion
                temp_score += Square::MyQueen.midgame_value(end_position)
                    - Square::MyPawn.midgame_value(end_position) //Always promote to queen
            } else if board_state.en_passant_position == Some(end_position) {
                //Capture a pawn en passant
                // TODO explain
                temp_score +=
                    Square::MyPawn.midgame_value(BOARD_SIZE - 1 - (end_position + BOARD_SIDE))
            }
        }
        _ => {}
    }
    temp_score
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
    score: 0,
    my_castling_rights: (true, true),
    opponent_castling_rights: (true, true),
    en_passant_position: None,
    king_passant_position: None,
};
