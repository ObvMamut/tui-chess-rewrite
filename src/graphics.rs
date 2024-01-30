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



mod graphics {
    pub const SEPARATOR_VERTICAL: &'static str = "║";
    pub const CORNER_LEFT_TOP: &'static str = "╔";
    pub const CORNER_LEFT_BOTTOM: &'static str = "╚";
    pub const CORNER_RIGHT_TOP: &'static str = "╗";
    pub const CORNER_RIGHT_BOTTOM: &'static str = "╝";
    pub const LINE: &'static str = "═";
    pub const PIECES: [&str;13] = [" ", "♖", "♘", "♗", "♕", "♔", "♙", "♜", "♞", "♝", "♛", "♚", "♟︎"];
    pub const START_SCREEN: &'static [&'static str] = &[
        "╔══════════════════════════════╗",
        "║───────CHESS - Mamut──────────║",
        "║──────────────────────────────║",
        "║ h ┆ help                     ║",
        "║ o ┆ mode screen              ║",
        "║ r ┆ restart / new game       ║",
        "║ m ┆ move                     ║",
        "║ q ┆ quit                     ║",
        "║ d ┆ debug                    ║",
        "║ n ┆ switch modes             ║",
        "╚═══╧══════════════════════════╝"
    ];
    pub const HELP_SCREEN: &'static [&'static str] = &[
        "╔═══════════════════════════════════════════════════════════════════╗",
        "║───────────────────────────HELP SCREEN─────────────────────────────║",
        "║───────────────────────────────────────────────────────────────────║",
        "║ X:Y>H:Z    ┆ move piece                                           ║",
        "║            ┆ {X:Y = Row:Column of your piece}                     ║",
        "║            ┆ {H:Z = Row:Column of the square you want to move to} ║",
        "║ s          ┆ start screen                                         ║",
        "╚═══════════════════════════════════════════════════════════════════╝"
    ];

    pub const MODE_SCREEN: &'static [&'static str] = &[
        "╔════════════════════════════════════════╗",
        "║─────────SWITCH MODES───────────────────║",
        "║────────────────────────────────────────║",
        "║ Player vs. Player          ┆           ║",
        "║ Player vs. AI  (Stockfish) ┆           ║",
        "╚════════════════════════════════════════╝"
];
}

pub fn clear_screen() {

    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();

    write!(stdout,
           "{}{}{}",
           termion::clear::All,
           termion::cursor::Goto(1, 1),
           termion::cursor::Hide)
        .unwrap();

    stdout.flush().unwrap();
}

pub fn draw(x: u32, y: u32, content: String, color: &str) {

    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();


    match color {
        "red" => {

            write!(stdout, "{}{}{}{}",
                   termion::cursor::Goto(x as u16, y as u16),
                   termion::color::Fg(termion::color::Red),
                   content,
                   termion::cursor::Hide).unwrap();
            stdout.flush().unwrap();
        },
        "green" => {

            write!(stdout, "{}{}{}{}",
                   termion::cursor::Goto(x as u16, y as u16),
                   termion::color::Fg(termion::color::Green),
                   content,
                   termion::cursor::Hide).unwrap();
            stdout.flush().unwrap();
        },
        _ => {

            write!(stdout, "{}{}{}{}",
                   termion::cursor::Goto(x as u16, y as u16),
                   termion::color::Fg(termion::color::White),
                   content,
                   termion::cursor::Hide).unwrap();
            stdout.flush().unwrap();
        }
    }

}

pub fn start_screen() {

    clear_screen();

    let x: u32 = 55;
    let mut y: u32 = 15;

    for row in graphics::START_SCREEN {
        draw(x, y, row.to_string(), "white");
        y += 1;
    }
}


pub fn help_screen() {
    clear_screen();

    let x: u32 = 35;
    let mut y: u32 = 15;

    for row in graphics::HELP_SCREEN {
        draw(x, y, row.to_string(), "white");
        y += 1;
    }
}

pub fn display_board(board: [[usize;8];8]) {

    let x_left_corner: u32 = 50;
    let y_top: u32 = 15;

    for index in 0..8 {
        for scnd_index in 0..8 {

            // Left wall
            draw((x_left_corner + (scnd_index * 5)), (y_top + (index * 3)), graphics::CORNER_LEFT_TOP.to_string(), "white");
            draw((x_left_corner + (scnd_index * 5)), (y_top + (index * 3) + 1), graphics::SEPARATOR_VERTICAL.to_string(), "white");
            draw((x_left_corner + (scnd_index * 5)), (y_top + (index * 3) + 2), graphics::CORNER_LEFT_BOTTOM.to_string(), "white");

            // Right wall
            draw((x_left_corner + (scnd_index * 5) + 4), (y_top + (index * 3)), graphics::CORNER_RIGHT_TOP.to_string(), "white");
            draw((x_left_corner + (scnd_index * 5) + 4), (y_top + (index * 3) + 1), graphics::SEPARATOR_VERTICAL.to_string(), "white");
            draw((x_left_corner + (scnd_index * 5) + 4), (y_top + (index * 3) + 2), graphics::CORNER_RIGHT_BOTTOM.to_string(), "white");

            // Line
            draw((x_left_corner + (scnd_index * 5) + 1), (y_top + (index * 3)), graphics::LINE.to_string(), "white");
            draw((x_left_corner + (scnd_index * 5) + 2), (y_top + (index * 3)), graphics::LINE.to_string(), "white");
            draw((x_left_corner + (scnd_index * 5) + 3), (y_top + (index * 3)), graphics::LINE.to_string(), "white");

            draw((x_left_corner + (scnd_index * 5) + 1), (y_top + (index * 3) + 2), graphics::LINE.to_string(), "white");
            draw((x_left_corner + (scnd_index * 5) + 2), (y_top + (index * 3) + 2), graphics::LINE.to_string(), "white");
            draw((x_left_corner + (scnd_index * 5) + 3), (y_top + (index * 3) + 2), graphics::LINE.to_string(), "white");

            // Piece

            let piece: usize = board[index as usize][scnd_index as usize];
            let mut color = "";

            match piece {
                1 | 2 | 3 | 4 | 5 | 6 => {
                    color = "red"
                },

                7 | 8 | 9 | 10 | 11 | 12 => {
                    color = "green"
                },
                _ => {}
            }

            draw(x_left_corner + (scnd_index * 5) + 2, (y_top + (index * 3) + 1), graphics::PIECES[piece].to_string(), color);
        }

        // Indicators


        // Horizontal
        draw((x_left_corner + 2 + index * 5), (y_top - 1), index.to_string(), "white");

        // Vertical
        draw((x_left_corner - 2), (y_top + 1 + index * 3), index.to_string(), "white");
    }
}

pub fn display_all(board: [[usize;8];8]) {
    clear_screen();

    display_board(board);

}

pub fn display_move(command: String) {
    let content = format!("MOVE : {}", command);
    draw(50, 11, content, "white");
}