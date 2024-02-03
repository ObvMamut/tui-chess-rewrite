#![feature(exclusive_range_pattern)]
#![allow(unused)]
#![feature(const_trait_impl)]
#![feature(let_chains)]
#![feature(ascii_char)]

mod graphics;

extern crate termion;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use std::io::{Write, stdout, stdin, Stdout, Stdin};
use std::{io, thread, time};
use std::ascii::Char;
use extra::rand::Randomizer;
use std::io::Read;
use std::thread::current;
use termion::async_stdin;
use run_script::ScriptOptions;
use crate::graphics::{display_all, display_move, draw};


enum Mode {
    PvP,
    AI
}

enum GameStatus {
    Started,
    Ended,
    NotStarted
}

enum CommandDebug {
    Valid,
    InValid
}

enum Round {
    White,
    Black
}

struct Game {
    mode: Mode,
    status: GameStatus,
    board: Board,
    cmd_debug: CommandDebug,
    round: Round
}


struct Board {
    board: [[usize; 8]; 8],
}

impl Board {
    fn new() -> Self {
        Board {
            board : [
                [1, 2, 3, 4, 0, 3, 2, 1],
                [6, 6, 6, 6, 0, 6, 6, 6],
                [0, 0, 12, 0, 0, 0, 0, 0],
                [0, 0, 0, 0, 11, 0, 0, 0],
                [0, 0, 0, 0, 5, 0, 0, 0],
                [0, 0, 0, 0, 0, 0, 0, 0],
                [0, 12, 12, 12, 0, 12, 12, 12],
                [7, 8, 9, 10, 0, 9, 8, 7]
            ],
        }
    }
}

impl Game {

    fn new() -> Self {
        Game {
            mode: Mode::PvP,
            status: GameStatus::Started,
            board: Board::new(),
            cmd_debug: CommandDebug::InValid,
            round: Round::White
        }
    }

    fn init(&mut self) {
        let stdin: Stdin = stdin();
        let mut stdout: RawTerminal<Stdout> = stdout().into_raw_mode().unwrap();

        graphics::clear_screen();
        graphics::start_screen();
        // key catching

        for c in stdin.keys() {
            match c.unwrap() {
                Key::Char('q') => break,
                Key::Char('h') => graphics::help_screen(),
                Key::Char('s') => graphics::start_screen(),
                Key::Char('m') => {

                    match self.mode {
                        Mode::PvP => {
                            if Game::move_piece(self) == true {
                                Game::update(self);
                            }
                        },
                        Mode::AI => {

                        },
                        _ => {}
                    }},
                Key::Char('r') => {
                    display_all(self.board.board);
                }
                _ => {}
            }
            stdout.flush().unwrap();
        }

        write!(stdout, "{}", termion::cursor::Show).unwrap();
        Game::update(self);

    }

    fn update(&mut self) {

        match self.round {
            Round::White => {
                self.round = Round::Black;
            }
            Round::Black => {
                self.round = Round::White;
            }
            _ => {}
        }

        display_all(self.board.board);
    }

    fn is_valid_move(&mut self, command: Vec<Vec<Option<u32>>>) -> bool {

        let original_position = &command[0];
        let future_position = &command[1];
        let piece_to_move = self.board.board[original_position[0].unwrap() as usize][original_position[1].unwrap() as usize];
        let mut next_position = vec![future_position[0].unwrap(), future_position[1].unwrap()];

        // Check if the original position is a position where a piece exists
        match self.round {
            Round::White => {
                match self.board.board[original_position[0].unwrap() as usize][original_position[1].unwrap() as usize] {
                    7 | 8 | 9 | 10 | 11 | 12 => {
                    }
                    _ => {
                        return false
                    }
                }
            }
            Round::Black => {
                match self.board.board[original_position[0].unwrap() as usize][original_position[1].unwrap() as usize] {
                    1 | 2 | 3 | 4 | 5 | 6 => {}
                    _ => {return false}
                }
            }
            _ => {}
        }

        // Check for Castling

        match piece_to_move {

            // White King
            11 =>  {
                // Check if future position is a White Tower
                if self.board.board[future_position[0].unwrap() as usize][future_position[1].unwrap() as usize] == 7 {
                    if Game::castling(self, original_position, future_position) == true {
                        return true
                    }
                }
            }

            // Black King
            5 => {
                // Check if future position is a Black Tower
                if self.board.board[future_position[0].unwrap() as usize][future_position[1].unwrap() as usize] == 1 {
                    if Game::castling(self, original_position, future_position) == true {
                        return true
                    }
                }
            }

            _ => {}
        }

        let possible_moves: Vec<Vec<u32>> = Game::get_possible_moves(self, piece_to_move, original_position);

        if possible_moves.is_empty() {
            return false
        }


        return if possible_moves.contains(&next_position) {
            println!("{:?}", possible_moves);
            true
        } else {
            false
        }

    }

    fn get_possible_moves(&mut self, piece: usize, position: &Vec<Option<u32>>) -> Vec<Vec<u32>> {
        let mut positions: Vec<Vec<u32>> = vec![];

        let pos_y: u32 = position[0].unwrap(); // ROWS
        let pos_x: u32 = position[1].unwrap(); // COLUMNS

        match piece {

            // White pawn

            12 => {
                // Check if you could move a square in front
                if pos_y as i32 - 1 >= 0 {

                    // Check if the position in front of the pawn is empty
                    if self.board.board[pos_y as usize - 1][pos_x as usize] == 0 {
                        positions.push(vec![pos_y - 1, pos_x])
                    }

                    // Capture to the left
                    if pos_x as i32 - 1 >= 0 {
                        // Check if the position up-left has enemy
                        match self.board.board[pos_y as usize - 1][pos_x as usize - 1] {
                            1 | 2 | 3 | 4 | 5 | 6 => {
                                positions.push(vec![pos_y - 1, pos_x - 1])
                            }
                            _ => {}
                        }
                    }

                    // Capture to the right
                    if pos_x as i32 - 1 <= 7 {
                        // Check if the position up-right has enemy
                        match self.board.board[pos_y as usize - 1][pos_x as usize + 1] {
                            1 | 2 | 3 | 4 | 5 | 6 => {
                                positions.push(vec![pos_y - 1, pos_x + 1])
                            }
                            _ => {}
                        }
                    }

                }

                if pos_y == 6 {
                    // Pawn in original place
                    if self.board.board[pos_y as usize - 2][pos_x as usize] == 0 {
                        // Position 2x in-front of Pawn empty
                        positions.push(vec![pos_y - 2, pos_x])
                    }
                }



            },

            // Black pawn

            6 => {
                // Check if you could move a square in front
                if pos_y as i32 - 1 <= 7 {

                    // Check if the position in front of the pawn is empty
                    if self.board.board[pos_y as usize + 1][pos_x as usize] == 0 {
                        positions.push(vec![pos_y + 1, pos_x])
                    }

                    // Capture to the left
                    if pos_x as i32 - 1 >= 0 {
                        // Check if the position up-left has enemy
                        match self.board.board[pos_y as usize + 1][pos_x as usize - 1] {
                            1 | 2 | 3 | 4 | 5 | 6 => {
                                positions.push(vec![pos_y + 1, pos_x - 1])
                            }
                            _ => {}
                        }
                    }

                    // Capture to the right
                    if pos_x as i32 + 1 <= 7 {
                        // Check if the position up-right has enemy
                        match self.board.board[pos_y as usize + 1][pos_x as usize + 1] {
                            1 | 2 | 3 | 4 | 5 | 6 => {
                                positions.push(vec![pos_y + 1, pos_x + 1])
                            }
                            _ => {}
                        }
                    }
                }

                if pos_y == 1 {
                    // Pawn in original place
                    if self.board.board[pos_y as usize + 2][pos_x as usize] == 0 {
                        // Position 2x in-front of Pawn empty
                        positions.push(vec![pos_y + 2, pos_x])
                    }
                }
            }

            // Towers

            7 | 1 => {

                let mut token = 1;

                // Top

                let mut top_token: bool = true;

                while top_token == true {
                    if (pos_y + 10) - token <= 9 {
                        top_token = false
                    } else {
                        match self.board.board[pos_y as usize - token as usize][pos_x as usize] {
                            // Checks if the position up token is White or Black

                            // Black => Pushes that position and closes loop

                            1 | 2 | 3 | 4 | 5 | 6 => {
                                match piece {
                                    7 => {
                                        positions.push(vec![pos_y - token, pos_x]);
                                        top_token = false;
                                    }
                                    1 => {
                                        top_token = false;
                                    }
                                    _ => {}
                                }

                            }

                            // White => Closes loop

                            7 | 8 | 9 | 10 | 11 | 12 => {
                                match piece {
                                    1 => {
                                        positions.push(vec![pos_y - token, pos_x]);
                                        top_token = false;
                                    }
                                    7 => {
                                        top_token = false;
                                    }
                                    _ => {}
                                }
                            }

                            _ => {
                                positions.push(vec![pos_y - token, pos_x]);
                                token += 1
                            }
                        }
                    }
                }

                // Down

                token = 1;

                let mut down_token: bool = true;

                while down_token == true {

                    if (pos_y + 10) + token >= 18  {
                        down_token = false
                    } else {
                        match self.board.board[pos_y as usize + token as usize][pos_x as usize] {
                            // Checks if the position up token is White or Black

                            // Black => Pushes that position and closes loop

                            1 | 2 | 3 | 4 | 5 | 6 => {
                                match piece {
                                    7 => {
                                        positions.push(vec![pos_y + token, pos_x]);
                                        down_token = false;
                                    }
                                    1 => {
                                        down_token = false;
                                    }
                                    _ => {}
                                }
                            }

                            // White => Closes loop

                            7 | 8 | 9 | 10 | 11 | 12 => {
                                match piece {
                                    1 => {
                                        positions.push(vec![pos_y + token, pos_x]);
                                        down_token = false;
                                    }
                                    7 => {
                                        down_token = false;
                                    }
                                    _ => {}
                                }
                            }

                            _ => {
                                positions.push(vec![pos_y + token, pos_x]);
                                token += 1
                            }
                        }
                    }
                }

                // Left

                token = 1;

                let mut left_token: bool = true;

                while left_token == true {

                    if (pos_x + 10) - token <= 9  {
                        left_token = false
                    } else {
                        match self.board.board[pos_y as usize][pos_x as usize - token as usize] {
                            // Checks if the position up token is White or Black

                            // Black => Pushes that position and closes loop

                            1 | 2 | 3 | 4 | 5 | 6 => {
                                match piece {
                                    7 => {
                                        positions.push(vec![pos_y, pos_x - token]);
                                        left_token = false;
                                    }
                                    1 => {
                                        left_token = false;
                                    }
                                    _ => {}
                                }
                            }

                            // White => Closes loop

                            7 | 8 | 9 | 10 | 11 | 12 => {
                                match piece {
                                    1 => {
                                        positions.push(vec![pos_y, pos_x - token]);
                                        left_token = false;
                                    }
                                    7 => {
                                        left_token = false;
                                    }
                                    _ => {}
                                }
                            }

                            _ => {
                                positions.push(vec![pos_y, pos_x - token]);
                                token += 1
                            }
                        }
                    }
                }

                // Right

                token = 1;

                let mut right_token: bool = true;

                while right_token == true {
                    if (pos_x + 10) + token >= 18 {
                        right_token = false
                    } else {
                        match self.board.board[pos_y as usize][pos_x as usize + token as usize] {
                            // Checks if the position up token is White or Black

                            // Black => Pushes that position and closes loop

                            1 | 2 | 3 | 4 | 5 | 6 => {
                                match piece {
                                    7 => {
                                        positions.push(vec![pos_y, pos_x + token]);
                                        right_token = false;
                                    }
                                    1 => {
                                        right_token = false;
                                    }
                                    _ => {}
                                }
                            }

                            // White => Closes loop

                            7 | 8 | 9 | 10 | 11 | 12 => {
                                match piece {
                                    1 => {
                                        positions.push(vec![pos_y, pos_x + token]);
                                        right_token = false;
                                    }
                                    7 => {
                                        right_token = false;
                                    }
                                    _ => {}
                                }
                            }

                            _ => {
                                positions.push(vec![pos_y, pos_x + token]);
                                token += 1
                            }
                        }
                    }
                }


            }
            
            // Knights

            8 | 2 => {

                // Up 2 positions

                if (pos_y + 10) - 2 >= 10 {

                    // Left

                    if (pos_x + 10) - 1 >= 10 {
                        match piece {
                            8 => {
                                match self.board.board[pos_y as usize - 2][pos_x as usize - 1] {
                                    0 | 1 | 2 | 3 | 4 | 5 | 6 => {
                                        positions.push(vec![pos_y - 2, pos_x - 1])
                                    }
                                    _ => {}
                                }
                            }
                            2 => {
                                match self.board.board[pos_y as usize - 2][pos_x as usize - 1] {
                                    0 | 7 | 8 | 9 | 10 | 11 | 12 => {
                                        positions.push(vec![pos_y - 2, pos_x - 1])
                                    }
                                    _ => {}
                                }
                            }
                            _ => {}
                        }
                    }

                    // Right

                    if pos_x + 1 <= 7 {

                        match piece {
                            8 => {
                                match self.board.board[pos_y as usize - 2][pos_x as usize + 1] {
                                    0 | 1 | 2 | 3 | 4 | 5 | 6 => {
                                        positions.push(vec![pos_y - 2, pos_x + 1])
                                    }
                                    _ => {}
                                }
                            }
                            2 => {
                                match self.board.board[pos_y as usize - 2][pos_x as usize + 1] {
                                    0 | 7 | 8 | 9 | 10 | 11 | 12 => {
                                        positions.push(vec![pos_y - 2, pos_x + 1])
                                    }
                                    _ => {}
                                }
                            }
                            _ => {}
                        }

                    }

                }

                // Up 1 position

                if (pos_y + 10) - 1 >= 10 {

                    // Left

                    if (pos_x + 10) - 2 >= 10 {

                        match piece {
                            8 => {
                                match self.board.board[pos_y as usize - 1][pos_x as usize - 2] {
                                    0 | 1 | 2 | 3 | 4 | 5 | 6 => {
                                        positions.push(vec![pos_y - 1, pos_x - 2])
                                    }
                                    _ => {}
                                }
                            }
                            2 => {
                                match self.board.board[pos_y as usize - 1][pos_x as usize - 2] {
                                    0 | 7 | 8 | 9 | 10 | 11 | 12 => {
                                        positions.push(vec![pos_y - 1, pos_x - 2])
                                    }
                                    _ => {}
                                }
                            }
                            _ => {}
                        }

                    }

                    // Right

                    if pos_x + 2 <= 7 {

                        match piece {
                            8 => {
                                match self.board.board[pos_y as usize - 1][pos_x as usize + 2] {
                                    0 | 1 | 2 | 3 | 4 | 5 | 6 => {
                                        positions.push(vec![pos_y - 1, pos_x + 2])
                                    }
                                    _ => {}
                                }
                            }
                            2 => {
                                match self.board.board[pos_y as usize - 1][pos_x as usize + 2] {
                                    0 | 7 | 8 | 9 | 10 | 11 | 12 => {
                                        positions.push(vec![pos_y - 1, pos_x + 2])
                                    }
                                    _ => {}
                                }
                            }
                            _ => {}
                        }

                    }

                }

                // Down 2 positions

                if pos_y + 2 <= 7 {

                    // Left

                    if (pos_x + 10) - 1 >= 10 {

                        match piece {
                            8 => {
                                match self.board.board[pos_y as usize + 2][pos_x as usize - 1] {
                                    0 | 1 | 2 | 3 | 4 | 5 | 6 => {
                                        positions.push(vec![pos_y + 2, pos_x - 1])
                                    }
                                    _ => {}
                                }
                            }
                            2 => {
                                match self.board.board[pos_y as usize + 2][pos_x as usize - 1] {
                                    0 | 7 | 8 | 9 | 10 | 11 | 12 => {
                                        positions.push(vec![pos_y + 2, pos_x - 1])
                                    }
                                    _ => {}
                                }
                            }
                            _ => {}
                        }

                    }

                    // Right

                    if pos_x + 1 <= 7 {

                        match piece {
                            8 => {
                                match self.board.board[pos_y as usize + 2][pos_x as usize + 1] {
                                    0 | 1 | 2 | 3 | 4 | 5 | 6 => {
                                        positions.push(vec![pos_y + 2, pos_x + 1])
                                    }
                                    _ => {}
                                }
                            }
                            2 => {
                                match self.board.board[pos_y as usize + 2][pos_x as usize + 1] {
                                    0 | 7 | 8 | 9 | 10 | 11 | 12 => {
                                        positions.push(vec![pos_y + 2, pos_x + 1])
                                    }
                                    _ => {}
                                }
                            }
                            _ => {}
                        }

                    }

                }

                // Down 1 position

                if pos_y + 1 <= 7 {

                    // Left

                    if (pos_x + 10) - 2 >= 10 {

                        match piece {
                            8 => {
                                match self.board.board[pos_y as usize + 1][pos_x as usize - 2] {
                                    0 | 1 | 2 | 3 | 4 | 5 | 6 => {
                                        positions.push(vec![pos_y + 1, pos_x - 2])
                                    }
                                    _ => {}
                                }
                            }
                            2 => {
                                match self.board.board[pos_y as usize + 1][pos_x as usize - 2] {
                                    0 | 7 | 8 | 9 | 10 | 11 | 12 => {
                                        positions.push(vec![pos_y + 1, pos_x - 2])
                                    }
                                    _ => {}
                                }
                            }
                            _ => {}
                        }

                    }

                    // Right

                    if pos_x + 2 <= 7 {

                        match piece {
                            8 => {
                                match self.board.board[pos_y as usize + 1][pos_x as usize + 2] {
                                    0 | 1 | 2 | 3 | 4 | 5 | 6 => {
                                        positions.push(vec![pos_y + 1, pos_x + 2])
                                    }
                                    _ => {}
                                }
                            }
                            2 => {
                                match self.board.board[pos_y as usize + 1][pos_x as usize + 2] {
                                    0 | 7 | 8 | 9 | 10 | 11 | 12 => {
                                        positions.push(vec![pos_y + 1, pos_x + 2])
                                    }
                                    _ => {}
                                }
                            }
                            _ => {}
                        }

                    }

                }

            }

            // Bishops

            9 | 3 => {
                let mut token = 1;

                // Up-Left

                let mut up_left_token = true;

                while up_left_token == true {
                    if (pos_y + 10) - token >= 10 && (pos_x + 10) - token >= 10 {
                            match piece {
                                9 => {
                                    match self.board.board[pos_y as usize - token as usize][pos_x as usize - token as usize] {
                                        1 | 2 | 3 | 4 | 5 | 6 => {
                                            positions.push(vec![pos_y - token, pos_x - token]);
                                            up_left_token = false
                                        }
                                        7 | 8 | 9 | 10 | 11 | 12 => {
                                            up_left_token = false
                                        }
                                        _ => {
                                            positions.push(vec![pos_y - token, pos_x - token]);
                                            //println!("{:?}", positions);
                                            token += 1;
                                        }
                                    }
                                }
                                3 => {
                                    match self.board.board[pos_y as usize - token as usize][pos_x as usize - token as usize] {
                                        7 | 8 | 9 | 10 | 11 | 12 => {
                                            positions.push(vec![pos_y - token, pos_x - token]);
                                            up_left_token = false
                                        }
                                        1 | 2 | 3 | 4 | 5 | 6 => {
                                            up_left_token = false
                                        }
                                        _ => {
                                            positions.push(vec![pos_y - token, pos_x - token]);
                                            token += 1;
                                        }
                                    }
                                }
                                _ => {}
                            }
                    } else {
                        up_left_token = false;
                    }
                }

                // Up-Right

                token = 1;

                let mut up_right_token = true;

                while up_right_token == true {
                    if (pos_y + 10) - token >= 10 && pos_x + token <= 7 {
                        match piece {
                            9 => {
                                match self.board.board[pos_y as usize - token as usize ][pos_x as usize + token as usize ] {
                                    1 | 2 | 3 | 4 | 5 | 6 => {
                                        positions.push(vec![pos_y - token, pos_x + token]);
                                        up_right_token = false
                                    }
                                    7 | 8 | 9 | 10 | 11 | 12 => {
                                        up_right_token = false
                                    }
                                    _ => {
                                        positions.push(vec![pos_y - token, pos_x + token]);
                                        token += 1;
                                    }
                                }
                            }
                            3 => {
                                match self.board.board[pos_y as usize - token as usize ][pos_x as usize + token as usize ] {
                                    7 | 8 | 9 | 10 | 11 | 12 => {
                                        positions.push(vec![pos_y - token, pos_x + token]);
                                        up_right_token = false
                                    }
                                    1 | 2 | 3 | 4 | 5 | 6 => {
                                        up_right_token = false
                                    }
                                    _ => {
                                        positions.push(vec![pos_y - token, pos_x + token]);
                                        token += 1;
                                    }
                                }
                            }
                            _ => {}
                        }
                    } else {
                        up_right_token = false;
                    }
                }

                token = 1;

                // Down-Left

                let mut down_left_token = true;

                while down_left_token == true {
                    if pos_y + token <= 7 && (pos_x + 10) - token >= 10 {
                        match piece {
                            9 => {
                                match self.board.board[pos_y as usize + token as usize][pos_x as usize - token as usize] {
                                    1 | 2 | 3 | 4 | 5 | 6 => {
                                        positions.push(vec![pos_y + token, pos_x - token]);
                                        down_left_token = false
                                    }
                                    7 | 8 | 9 | 10 | 11 | 12 => {
                                        down_left_token = false
                                    }
                                    _ => {
                                        positions.push(vec![pos_y + token, pos_x - token]);
                                        //println!("{:?}", positions);
                                        token += 1;
                                    }
                                }
                            }
                            3 => {
                                match self.board.board[pos_y as usize + token as usize][pos_x as usize - token as usize] {
                                    7 | 8 | 9 | 10 | 11 | 12 => {
                                        positions.push(vec![pos_y + token, pos_x - token]);
                                        down_left_token = false
                                    }
                                    1 | 2 | 3 | 4 | 5 | 6 => {
                                        down_left_token = false
                                    }
                                    _ => {
                                        positions.push(vec![pos_y + token, pos_x - token]);
                                        token += 1;
                                    }
                                }
                            }
                            _ => {}
                        }
                    } else {
                        down_left_token = false;
                    }
                }

                // Down-Right

                token = 1;

                let mut down_right_token = true;

                while down_right_token == true {
                    if pos_y + token <= 7 && pos_x + token <= 7 {
                        match piece {
                            9 => {
                                match self.board.board[pos_y as usize + token as usize ][pos_x as usize + token as usize ] {
                                    1 | 2 | 3 | 4 | 5 | 6 => {
                                        positions.push(vec![pos_y + token, pos_x + token]);
                                        down_right_token = false
                                    }
                                    7 | 8 | 9 | 10 | 11 | 12 => {
                                        down_right_token = false
                                    }
                                    _ => {
                                        positions.push(vec![pos_y + token, pos_x + token]);
                                        token += 1;
                                    }
                                }
                            }
                            3 => {
                                match self.board.board[pos_y as usize + token as usize ][pos_x as usize + token as usize ] {
                                    7 | 8 | 9 | 10 | 11 | 12 => {
                                        positions.push(vec![pos_y + token, pos_x + token]);
                                        down_right_token = false
                                    }
                                    1 | 2 | 3 | 4 | 5 | 6 => {
                                        down_right_token = false
                                    }
                                    _ => {
                                        positions.push(vec![pos_y + token, pos_x + token]);
                                        token += 1;
                                    }
                                }
                            }
                            _ => {}
                        }
                    } else {
                        down_right_token = false;
                    }
                }

            }

            // Queens

            10 | 4 => {
                let bishop_numeric_value: usize;
                let tower_numeric_value: usize;

                match piece {
                    10 => {
                        bishop_numeric_value = 9;
                        tower_numeric_value = 7;
                    }
                    4 => {
                        bishop_numeric_value = 3;
                        tower_numeric_value = 1;
                    }

                    _ => {
                        bishop_numeric_value = 0;
                        tower_numeric_value = 0;
                    }
                }

                // Straights

                let mut straights = Game::get_possible_moves(self, tower_numeric_value, position);

                // Diagonals

                let mut diagonals = Game::get_possible_moves(self, bishop_numeric_value, position);

                // Concatenating

                for p in straights {
                    positions.push(p);
                }

                for p in diagonals {
                    positions.push(p);
                }

            }

            // Kings

            11 | 5 => {

                // Up

                if (pos_y + 10) - 1 >= 10 {

                    // Straight

                    match piece {
                        11 => {
                            match self.board.board[pos_y as usize - 1][pos_x as usize] {
                                0 | 1 | 2 | 3 | 4 | 5 | 6 => {
                                    positions.push(vec![pos_y - 1, pos_x])
                                }
                                _ => {}
                            }
                        }

                        5 => {
                            match self.board.board[pos_y as usize - 1][pos_x as usize] {
                                0 | 7 | 8 | 9 | 10 | 11 | 12 => {
                                    positions.push(vec![pos_y - 1, pos_x])
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }

                    // Left

                    if (pos_x + 10) - 1 >= 10 {
                        match piece {
                            11 => {
                                match self.board.board[pos_y as usize - 1][pos_x as usize - 1] {
                                    0 | 1 | 2 | 3 | 4 | 5 | 6 => {
                                        positions.push(vec![pos_y - 1, pos_x - 1])
                                    }
                                    _ => {}
                                }
                            }

                            5 => {
                                match self.board.board[pos_y as usize - 1][pos_x as usize - 1] {
                                    0 | 7 | 8 | 9 | 10 | 11 | 12 => {
                                        positions.push(vec![pos_y - 1, pos_x - 1])
                                    }
                                    _ => {}
                                }
                            }
                            _ => {}
                        }
                    }

                    // Right

                    if pos_x + 1 <= 7 {
                        match piece {
                            11 => {
                                match self.board.board[pos_y as usize - 1][pos_x as usize + 1] {
                                    0 | 1 | 2 | 3 | 4 | 5 | 6 => {
                                        positions.push(vec![pos_y - 1, pos_x + 1])
                                    }
                                    _ => {}
                                }
                            }

                            5 => {
                                match self.board.board[pos_y as usize - 1][pos_x as usize + 1] {
                                    0 | 7 | 8 | 9 | 10 | 11 | 12 => {
                                        positions.push(vec![pos_y - 1, pos_x + 1])
                                    }
                                    _ => {}
                                }
                            }
                            _ => {}
                        }
                    }

                }

                // Down

                if pos_y + 1 <= 7 {

                    // Straight

                    match piece {
                        11 => {
                            match self.board.board[pos_y as usize + 1][pos_x as usize] {
                                0 | 1 | 2 | 3 | 4 | 5 | 6 => {
                                    positions.push(vec![pos_y + 1, pos_x])
                                }
                                _ => {}
                            }
                        }

                        5 => {
                            match self.board.board[pos_y as usize + 1][pos_x as usize] {
                                0 | 7 | 8 | 9 | 10 | 11 | 12 => {
                                    positions.push(vec![pos_y + 1, pos_x])
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }

                    // Left

                    if (pos_x + 10) - 1 >= 10 {
                        match piece {
                            11 => {
                                match self.board.board[pos_y as usize + 1][pos_x as usize - 1] {
                                    0 | 1 | 2 | 3 | 4 | 5 | 6 => {
                                        positions.push(vec![pos_y + 1, pos_x - 1])
                                    }
                                    _ => {}
                                }
                            }

                            5 => {
                                match self.board.board[pos_y as usize + 1][pos_x as usize - 1] {
                                    0 | 7 | 8 | 9 | 10 | 11 | 12 => {
                                        positions.push(vec![pos_y + 1, pos_x - 1])
                                    }
                                    _ => {}
                                }
                            }
                            _ => {}
                        }
                    }

                    // Right

                    if pos_x + 1 <= 7 {
                        match piece {
                            11 => {
                                match self.board.board[pos_y as usize + 1][pos_x as usize + 1] {
                                    0 | 1 | 2 | 3 | 4 | 5 | 6 => {
                                        positions.push(vec![pos_y + 1, pos_x + 1])
                                    }
                                    _ => {}
                                }
                            }

                            5 => {
                                match self.board.board[pos_y as usize + 1][pos_x as usize + 1] {
                                    0 | 7 | 8 | 9 | 10 | 11 | 12 => {
                                        positions.push(vec![pos_y + 1, pos_x + 1])
                                    }
                                    _ => {}
                                }
                            }
                            _ => {}
                        }
                    }

                }

                // Left

                if (pos_x + 10) - 1 >= 10 {
                    match piece {
                        11 => {
                            match self.board.board[pos_y as usize][pos_x as usize - 1] {
                                0 | 1 | 2 | 3 | 4 | 5 | 6 => {
                                    positions.push(vec![pos_y, pos_x - 1])
                                }
                                _ => {}
                            }
                        }

                        5 => {
                            match self.board.board[pos_y as usize][pos_x as usize - 1] {
                                0 | 7 | 8 | 9 | 10 | 11 | 12 => {
                                    positions.push(vec![pos_y, pos_x - 1])
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }

                // Right

                if pos_x + 1 <= 7 {
                    match piece {
                        11 => {
                            match self.board.board[pos_y as usize][pos_x as usize + 1] {
                                0 | 1 | 2 | 3 | 4 | 5 | 6 => {
                                    positions.push(vec![pos_y, pos_x + 1])
                                }
                                _ => {}
                            }
                        }

                        5 => {
                            match self.board.board[pos_y as usize][pos_x as usize+ 1] {
                                0 | 7 | 8 | 9 | 10 | 11 | 12 => {
                                    positions.push(vec![pos_y, pos_x + 1])
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }

            }

            _ => {}
        }


        draw(1, 1, "".to_string(), "white");
        println!("{:?}", positions);
        return positions;
    }

    fn move_piece(&mut self) -> bool {
        let mut pos = Game::get_command(self);

        if pos.is_empty() {
            return false
        };


        let original_position = &pos[0];
        let future_position = &pos[1];
        let piece_that_was_taken = self.board.board[future_position[0].unwrap() as usize][future_position[1].unwrap() as usize];
        let piece_to_move = self.board.board[original_position[0].unwrap() as usize][original_position[1].unwrap() as usize];

        

        self.board.board[original_position[0].unwrap() as usize][original_position[1].unwrap() as usize] = 0;
        self.board.board[future_position[0].unwrap() as usize][future_position[1].unwrap() as usize] = piece_to_move;


        match self.round {
            Round::White => {
                if Game::is_check("white") == true {
                    // Checks when White moved if there is a check on the white king if YES it returns board to original

                    self.board.board[original_position[0].unwrap() as usize][original_position[1].unwrap() as usize] = piece_to_move;
                    self.board.board[future_position[0].unwrap() as usize][future_position[1].unwrap() as usize] = piece_that_was_taken;

                }
            }
            _ => {}
        }

        return true

    }

    fn is_check(king: &str) -> bool {
        return false
    }

    fn check_correct_cmd(command: String) -> bool {

        if command.len() != 7 {
            return false
        }

        for char in command.chars() {
            match char {
                ':' | '>' | '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' => {}
                _ => {
                    return false
                }

            }
        }

        match command.chars().nth(0).unwrap() {
            '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' => {}
            _ => {
                return false
            }
        }

        match command.chars().nth(1).unwrap() {
            ':' => {}
            _ => {
                return false
            }
        }

        match command.chars().nth(2).unwrap() {
            '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' => {}
            _ => {
                return false
            }
        }

        match command.chars().nth(3).unwrap() {
            '>' => {}
            _ => {
                return false
            }
        }

        match command.chars().nth(4).unwrap() {
            '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' => {}
            _ => {
                return false
            }
        }

        match command.chars().nth(5).unwrap() {
            ':' => {}
            _ => {
                return false
            }
        }

        match command.chars().nth(6).unwrap() {
            '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' => {}
            _ => {
                return false
            }
        }


        return true
    }

    fn get_command(&mut self) -> Vec<Vec<Option<u32>>> {

        draw(50, 11, "MOVE : ".to_string(), "white");

        let mut command = vec![];
        let mut cmd = String::from("");

        let stdin = stdin();
        let mut stdout = stdout().into_raw_mode().unwrap();

        for c in stdin.keys() {

            write!(stdout,
                   "{}",
                   termion::cursor::Hide)
                .unwrap();

            match c.unwrap() {
                Key::Char('q') => {
                    break
                },
                Key::Char('e') => {
                    break
                }

                Key::Char(c) => {
                    cmd.push(c);
                    display_move(cmd.clone());
                },


                _ => {}
            }
            stdout.flush().unwrap();
        }

        write!(stdout, "{}", termion::cursor::Hide).unwrap();

        display_all(self.board.board);

        if Game::check_correct_cmd(cmd.clone()) == false {
            command = vec![];
            return command
        }


        let pos1 = vec![cmd.clone().chars().nth(0).unwrap().to_digit(10), cmd.clone().chars().nth(2).unwrap().to_digit(10)];
        let pos2 = vec![cmd.clone().chars().nth(4).unwrap().to_digit(10), cmd.clone().chars().nth(6).unwrap().to_digit(10)];


        command.push(pos1);
        command.push(pos2);

        if Game::is_valid_move(self, command.clone()) == false {
            command = vec![];
            return command
        }

        return command
    }

    fn castling(&mut self, original_position: &Vec<Option<u32>>, future_position: &Vec<Option<u32>>) -> bool {
        return true
    }

}



fn main() {
    let mut game = Game::new();
    Game::init(&mut game);
}
