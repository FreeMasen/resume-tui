use ratatui::buffer::Cell;
use ratatui::layout::Size;
use ratatui::style::{Color, Modifier};
use wasm_bindgen::prelude::*;

pub struct Terminal;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen]
    fn writeToCanvas(s: String, x: u16, y: u16, bold: bool, italic: bool);
    #[wasm_bindgen]
    fn writeToTerminal(s: String);
    #[wasm_bindgen]
    fn canvasSizeW() -> u16;
    #[wasm_bindgen]
    fn canvasSizeH() -> u16;
}

impl ratatui::backend::Backend for Terminal {
    fn draw<'a, I>(&mut self, content: I) -> std::io::Result<()>
    where
        I: Iterator<Item = (u16, u16, &'a ratatui::prelude::buffer::Cell)>,
    {
        writeToTerminal(
            content
                .into_iter()
                .map(|(x, y, c)| ansiify_cell(x, y, c))
                .collect(),
        );
        Ok(())
    }

    fn hide_cursor(&mut self) -> std::io::Result<()> {
        Ok(())
    }

    fn show_cursor(&mut self) -> std::io::Result<()> {
        Ok(())
    }

    fn get_cursor(&mut self) -> std::io::Result<(u16, u16)> {
        Ok((0, 0))
    }

    fn set_cursor(&mut self, _x: u16, _y: u16) -> std::io::Result<()> {
        Ok(())
    }

    fn clear(&mut self) -> std::io::Result<()> {
        Ok(())
    }

    fn size(&self) -> std::io::Result<ratatui::layout::Size> {
        let w = canvasSizeW();
        let h = canvasSizeH();
        Ok(ratatui::layout::Size::new(w, h))
    }

    fn window_size(&mut self) -> std::io::Result<ratatui::prelude::backend::WindowSize> {
        let height = canvasSizeH();
        let width = canvasSizeW();
        Ok(ratatui::prelude::backend::WindowSize {
            columns_rows: Size { height, width },
            pixels: Size {
                height: 0,
                width: 0,
            },
        })
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }

    fn get_cursor_position(&mut self) -> Result<ratatui::layout::Position, std::io::Error> {
        Ok((0, 0).into())
    }
    fn set_cursor_position<P>(&mut self, _: P) -> Result<(), std::io::Error>
    where
        P: Into<ratatui::layout::Position>,
    {
        Ok(())
    }
}

fn ansiify_cell(x: u16, y: u16, cell: &Cell) -> String {
    const BOLD: &str = "\x1B[1m";
    const DIM: &str = "\x1B[2m";
    const UNDERLINE: &str = "\x1B[4m";
    const SLOW_BLINK: &str = "\x1B[5m";
    const FAST_BLINK: &str = "\x1B[6m";
    const REVERSED: &str = "\x1B[7m";
    const HIDDEN: &str = "\x1B[8m";
    const STRIKE_THROUGH: &str = "\x1B[9m";
    let mut ret = format!("\x1B[{};{}H", y + 1, x + 1);
    ret.push_str(ansiify_color_fg(cell.fg));
    ret.push_str(ansiify_color_bg(cell.bg));
    if cell.modifier.contains(Modifier::BOLD) {
        ret.push_str(BOLD);
    }
    if cell.modifier.contains(Modifier::DIM) {
        ret.push_str(DIM);
    }
    if cell.modifier.contains(Modifier::UNDERLINED) {
        ret.push_str(UNDERLINE);
    }
    if cell.modifier.contains(Modifier::SLOW_BLINK) {
        ret.push_str(SLOW_BLINK);
    }
    if cell.modifier.contains(Modifier::RAPID_BLINK) {
        ret.push_str(FAST_BLINK);
    }
    if cell.modifier.contains(Modifier::CROSSED_OUT) {
        ret.push_str(STRIKE_THROUGH);
    }
    if cell.modifier.contains(Modifier::HIDDEN) {
        ret.push_str(HIDDEN);
    }
    if cell.modifier.contains(Modifier::REVERSED) {
        ret.push_str(REVERSED);
    }

    ret.push_str(cell.symbol());
    ret.push_str("\x1B[0m");
    ret
}

fn ansiify_color_fg(color: Color) -> &'static str {
    match color {
        Color::Reset => "\x1B[0m",
        Color::Black => "\x1B[30m",
        Color::Red => "\x1B[31m",
        Color::Green => "\x1B[32m",
        Color::Yellow => "\x1B[33m",
        Color::Blue => "\x1B[34m",
        Color::Magenta => "\x1B[35m",
        Color::Cyan => "\x1B[36m",
        Color::Gray => "\x1B[37m",
        Color::DarkGray => "\x1B[90m",
        Color::LightRed => "\x1B[91m",
        Color::LightGreen => "\x1B[92m",
        Color::LightYellow => "\x1B[93m",
        Color::LightBlue => "\x1B[94m",
        Color::LightMagenta => "\x1B[95m",
        Color::LightCyan => "\x1B[96m",
        Color::White => "\x1B[97m",
        _ => "",
    }
}
fn ansiify_color_bg(color: Color) -> &'static str {
    match color {
        Color::Reset => "\x1B[0m",
        Color::Black => "\x1B[40m",
        Color::Red => "\x1B[41m",
        Color::Green => "\x1B[42m",
        Color::Yellow => "\x1B[43m",
        Color::Blue => "\x1B[44m",
        Color::Magenta => "\x1B[45m",
        Color::Cyan => "\x1B[46m",
        Color::Gray => "\x1B[47m",
        Color::DarkGray => "\x1B[100m",
        Color::LightRed => "\x1B[101m",
        Color::LightGreen => "\x1B[102m",
        Color::LightYellow => "\x1B[103m",
        Color::LightBlue => "\x1B[104m",
        Color::LightMagenta => "\x1B[105m",
        Color::LightCyan => "\x1B[106m",
        Color::White => "\x1B[107m",
        _ => "",
    }
}
