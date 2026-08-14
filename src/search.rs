use log::info;
use schnellru::{ByLength, LruMap};
use std::cmp::max;
use std::time::{Duration, Instant};

use crate::board::{
    BoardState, ChessMove, after_move, gen_moves, move_value, nullmove, zobrist_hash,
};
use crate::pieces::Square;

// Keep mate scores outside every score reachable through normal evaluation.
pub const MATE_UPPER: i32 = 32_000 + 8 * 2529;
pub const MATE_LOWER: i32 = 32_000 - 8 * 2529;
const SCORE_TRANSPOSITION_TABLE_SIZE: u32 = 6_000_000;
const MOVE_TRANSPOSITION_TABLE_SIZE: u32 = 1_000_000;
const QUIESCENCE_SEARCH_LIMIT: i32 = 40;
const EVAL_ROUGHNESS: i32 = 15;
const STOP_SEARCH: i32 = MATE_UPPER * 101;

fn can_capture_king(board_state: &BoardState) -> bool {
    gen_moves(board_state)
        .iter()
        .any(|chess_move| move_value(board_state, chess_move) >= MATE_LOWER)
}

fn gives_check(board_state: &BoardState, chess_move: &ChessMove) -> bool {
    let opponent_to_move = after_move(board_state, chess_move);
    can_capture_king(&nullmove(&opponent_to_move))
}

#[derive(Clone, Copy)]
pub struct ScoreBounds {
    lower: i32,
    upper: i32,
}

const UNSEARCHED_BOUNDS: ScoreBounds = ScoreBounds {
    lower: -MATE_UPPER,
    upper: MATE_UPPER,
};

pub struct Searcher {
    pub score_transposition_table: LruMap<(u64, i32), ScoreBounds>,
    pub move_transposition_table: LruMap<u64, ChessMove>,
    pub nodes: u32,
    started_at: Instant,
    time_limit: Duration,
}

impl Default for Searcher {
    fn default() -> Self {
        Searcher {
            score_transposition_table: LruMap::new(ByLength::new(SCORE_TRANSPOSITION_TABLE_SIZE)),
            move_transposition_table: LruMap::new(ByLength::new(MOVE_TRANSPOSITION_TABLE_SIZE)),
            nodes: 0,
            started_at: Instant::now(),
            time_limit: Duration::from_secs(4),
        }
    }
}

impl Searcher {
    fn bound(&mut self, board_state: &BoardState, gamma: i32, depth: i32, root: bool) -> i32 {
        self.nodes += 1;

        // Sunfish is a king-capture engine, so we should always check if we
        // still have a king. Notice since this is the only termination check,
        // the remaining code has to be comfortable with being mated, stalemated
        // or able to capture the opponent king.
        if board_state.score <= -MATE_LOWER {
            return -MATE_UPPER;
        }
        let hash = zobrist_hash(board_state);

        // Look into the table if we have already searched this position before.
        // Bounds are reusable only when they came from the same search depth.
        // Quiescence search uses depth zero because it continues until the
        // position is quiet rather than to a fixed negative depth.

        let entry = if root {
            UNSEARCHED_BOUNDS
        } else {
            self.score_transposition_table
                .get(&(hash, max(depth, 0)))
                .copied()
                .unwrap_or(UNSEARCHED_BOUNDS)
        };

        if !root {
            if entry.lower >= gamma {
                return entry.lower;
            } else if entry.upper < gamma {
                return entry.upper;
            }
        }

        if self.started_at.elapsed() > self.time_limit {
            return STOP_SEARCH;
        }

        let mut best = -MATE_UPPER;
        if depth > 2
            && !root
            && board_state.score.abs() < 500
            && board_state.board.iter().any(|&square| {
                matches!(
                    square,
                    Square::MyRook | Square::MyKnight | Square::MyBishop | Square::MyQueen
                )
            })
        {
            let score = -self.bound(&nullmove(board_state), 1 - gamma, depth - 3, false);
            if score == -STOP_SEARCH {
                return STOP_SEARCH;
            }
            best = std::cmp::max(best, score.min(board_state.score + EVAL_ROUGHNESS));
        } else if depth <= 0 {
            // For QSearch we have a different kind of null-move
            let score = board_state.score;
            best = std::cmp::max(best, score);
        }

        let move_value_limit = if depth > 0 {
            -MATE_UPPER
        } else {
            QUIESCENCE_SEARCH_LIMIT
        };
        let mut preferred_move = self.move_transposition_table.get(&hash).copied();
        if preferred_move.is_none() && depth > 3 {
            if self.bound(board_state, gamma, depth - 3, true) == STOP_SEARCH {
                return STOP_SEARCH;
            }
            preferred_move = self.move_transposition_table.get(&hash).copied();
        }
        if best < gamma
            && let Some(preferred_move) = preferred_move
        {
            // Search the best move from the previous iteration first. The
            // transposition table makes the second visit inexpensive.
            if move_value(board_state, &preferred_move) >= move_value_limit {
                let score = -self.bound(
                    &after_move(board_state, &preferred_move),
                    1 - gamma,
                    depth - 1,
                    false,
                );
                if score == -STOP_SEARCH {
                    return STOP_SEARCH;
                }
                best = std::cmp::max(best, score);
            }
        }

        if best < gamma {
            let generated_moves = gen_moves(board_state);
            let mut ordered_moves = Vec::with_capacity(generated_moves.len());
            for chess_move in &generated_moves {
                let negated_move_value = -move_value(board_state, chess_move);
                if -negated_move_value >= move_value_limit {
                    // At the root, forcing checks are usually the most useful
                    // moves to establish a bound first.
                    ordered_moves.push((
                        !root || !gives_check(board_state, chess_move),
                        negated_move_value,
                        chess_move,
                    ));
                }
            }
            ordered_moves.sort_unstable_by_key(|(not_check, value, _)| (*not_check, *value));
            for (_, negated_move_value, chess_move) in ordered_moves {
                if depth <= 1
                    && board_state.score - negated_move_value < gamma
                    && -negated_move_value < MATE_LOWER
                {
                    best = std::cmp::max(best, board_state.score - negated_move_value);
                    break;
                }
                let score = -self.bound(
                    &after_move(board_state, chess_move),
                    1 - gamma,
                    depth - 1,
                    false,
                );
                if score == -STOP_SEARCH {
                    return STOP_SEARCH;
                }
                if score > best {
                    best = score;
                    // Reuse good moves for principal variations and move ordering.
                    if depth > 0 && (root || best >= gamma) {
                        self.move_transposition_table.insert(hash, *chess_move);
                    }
                }
                if best >= gamma {
                    break;
                }
            }
        }

        // Stalemate checking is a bit tricky: Say we failed low, because
        // we can't (legally) move and so the (real) score is -infty.
        // At the next depth we are allowed to just return r, -infty <= r < gamma,
        // which is normally fine.
        // However, what if gamma = -10 and we don't have any legal moves?
        // Then the score is actually a draw and we should fail high!
        // Thus, if best < gamma and best < 0 we need to double check what we are doing.
        // This doesn't prevent sunfish from making a move that results in stalemate,
        // but only if depth == 1, so that's probably fair enough.
        // (Btw, at depth 1 we can also mate without realizing.)
        if best < gamma
            && best < 0
            && depth > 0
            && gen_moves(board_state)
                .iter()
                .all(|chess_move| can_capture_king(&after_move(board_state, chess_move)))
        {
            let in_check = can_capture_king(&nullmove(board_state));
            best = if in_check { -MATE_UPPER } else { 0 };
        }

        // Update score_transposition_table
        if !root {
            let entry = if best >= gamma {
                ScoreBounds {
                    lower: best,
                    upper: entry.upper,
                }
            } else {
                ScoreBounds {
                    lower: entry.lower,
                    upper: best,
                }
            };
            self.score_transposition_table
                .insert((hash, max(depth, 0)), entry);
        }

        best
    }

    // Iterative deepening MTD-bi search
    pub fn search(
        &mut self,
        board_state: BoardState,
        time_limit: Duration,
    ) -> (ChessMove, i32, i32) {
        self.nodes = 0;
        self.started_at = Instant::now();
        self.time_limit = time_limit;
        let mut last_result = ((0, 0), 0, 0);
        let hash = zobrist_hash(&board_state);
        self.mark_as_draw(&board_state);

        // Bound depth to avoid infinite recursion in finished games
        for depth in 1..99 {
            // Realistically will reach depths around 6-12, except endgames
            let mut lower = -MATE_UPPER;
            let mut upper = MATE_UPPER;
            while lower < upper - EVAL_ROUGHNESS {
                let gamma = (lower + upper + 1) / 2;
                let score = self.bound(&board_state, gamma, depth, true);
                if score == STOP_SEARCH {
                    lower = STOP_SEARCH;
                    break;
                }
                if score >= gamma {
                    lower = score;
                } else {
                    upper = score;
                }
            }
            if lower == STOP_SEARCH {
                break;
            }
            let score = self.bound(&board_state, lower, depth, true);
            if score == STOP_SEARCH {
                break;
            }
            info!(
                "Reached depth {: <2} score {: <5} nodes {: <7} time {:?}",
                depth,
                score,
                self.nodes,
                self.started_at.elapsed()
            );

            // If the game hasn't finished we can retrieve our move from the
            // transposition table.

            last_result = (
                *self
                    .move_transposition_table
                    .get(&hash)
                    .expect("move not in table"),
                score,
                depth,
            );

            if self.started_at.elapsed() > self.time_limit || score > MATE_LOWER {
                // Don't waste time if a mate is found
                break;
            }
        }

        // The search uses pseudo-legal moves and scores a forced mate like an
        // illegal move. If they tie, still return a move that evades check.
        if can_capture_king(&after_move(&board_state, &last_result.0)) {
            last_result.0 = gen_moves(&board_state)
                .into_iter()
                .find(|chess_move| !can_capture_king(&after_move(&board_state, chess_move)))
                .expect("position has no legal move");
        }

        last_result
    }

    pub fn mark_as_draw(&mut self, board_state: &BoardState) {
        // A repeated position is a draw regardless of the remaining depth.
        let hash = zobrist_hash(board_state);
        for depth in 1..30 {
            self.score_transposition_table
                .insert((hash, depth), ScoreBounds { lower: 0, upper: 0 });
        }
    }
}
