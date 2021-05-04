use crate::font::{Font, FontWriter};
use core::fmt;

pub struct ConsoleWriter {
    screen: [[char; ConsoleWriter::MAX_ROWS]; ConsoleWriter::MAX_COLUMNS],
    cursor_row: usize,
    cursor_column: usize,
    characters: [Font; Font::MAX],
    error_character: Font,
    writer: FontWriter,
}

impl ConsoleWriter {
    const MAX_ROWS: usize = 25;
    const MAX_COLUMNS: usize = 80;

    pub fn new(writer: FontWriter) -> ConsoleWriter {
        let screen = [['\0'; ConsoleWriter::MAX_ROWS]; ConsoleWriter::MAX_COLUMNS];
        let characters = Font::all();
        let error_character = Font::new('■');

        ConsoleWriter {
            screen: screen,
            cursor_row: 0,
            cursor_column: 0,
            characters: characters,
            error_character: error_character,
            writer: writer,
        }
    }

    pub fn write(&mut self, string: &str) {
        for c in string.chars() {
            self.write_character(c)
        }
    }

    fn write_character(&mut self, c: char) {
        match c {
            '\n' => self.new_line(),
            _ => {
                if self.cursor_column >= ConsoleWriter::MAX_COLUMNS {
                    self.new_line();
                }
                self.screen[self.cursor_column][self.cursor_row] = c;
                //へんなキャストだけど他にいい方法を知らない
                let code = c as u32 as usize;
                //範囲エラーが怖いのでget
                let font = self.characters.get(code).unwrap_or(&self.error_character);
                self.writer.write(self.cursor_column, self.cursor_row, font);
                self.cursor_column += 1;
            }
        }
    }

    fn new_line(&mut self) {
        self.cursor_column = 0;
        if self.cursor_row < ConsoleWriter::MAX_ROWS - 1 {
            self.cursor_row += 1;
        } else {
            for x in 0..ConsoleWriter::MAX_COLUMNS {
                for y in 0..ConsoleWriter::MAX_ROWS {
                    self.writer.clear(x, y);
                }
            }
            for y in 0..(ConsoleWriter::MAX_ROWS - 1 ) {
                for x in 0..ConsoleWriter::MAX_COLUMNS {
                    self.screen[x][y] = self.screen[x][y+1];
                    let character = self.screen[x][y];
                    //へんなキャストだけど他にいい方法を知らない
                    let code = character as u32 as usize;
                    //範囲エラーが怖いのでget
                    let font = self.characters.get(code).unwrap_or(&self.error_character);
                    self.writer.write(x, y, font);
                }
            }
            for x in 0..ConsoleWriter::MAX_COLUMNS {
                self.screen[x][ConsoleWriter::MAX_ROWS -1] = '\0';
            }
        }
    }
}

impl fmt::Write for ConsoleWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write(s);
        Ok(())
    }
}