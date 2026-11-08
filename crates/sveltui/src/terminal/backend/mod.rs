use crate::terminal::buffer::Buffer;
use crate::terminal::cell::Cell;
use crate::terminal::image::{cell_pixels, plan_images};
use crate::terminal::pointer::{PointerDialect, PointerShape, write_pointer_shape};
use crossterm::{
    cursor::{self, MoveTo},
    event::{DisableMouseCapture, EnableMouseCapture},
    execute, queue,
    style::{
        Attribute, Color, Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor,
    },
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, Stdout, Write};
use std::sync::atomic::{AtomicBool, Ordering};

static CLEANED_UP: AtomicBool = AtomicBool::new(false);
// Set once a non-default pointer shape is sent, so exit resets it only then.
static POINTER_SHAPE_USED: AtomicBool = AtomicBool::new(false);

pub struct Terminal {
    stdout: Stdout,
    pub curr_buffer: Buffer,
    pub prev_buffer: Buffer,
    width: u16,
    height: u16,
    needs_full_repaint: bool,
    pointer_shape: PointerShape,
    pointer_shapes: bool,
    pointer_dialect: PointerDialect,
}

impl Terminal {
    pub fn new() -> io::Result<Self> {
        Self::new_with_mouse(true)
    }

    pub fn new_with_mouse(enable_mouse: bool) -> io::Result<Self> {
        let (width, height) = terminal::size()?;
        let mut stdout = io::stdout();

        terminal::enable_raw_mode()?;
        if enable_mouse {
            execute!(
                stdout,
                EnterAlternateScreen,
                cursor::Hide,
                EnableMouseCapture
            )?;
        } else {
            execute!(stdout, EnterAlternateScreen, cursor::Hide)?;
        }

        Self::set_cleanup_hook();

        refresh_cell_pixels();
        Ok(Self {
            stdout,
            curr_buffer: Buffer::new(width, height),
            prev_buffer: Buffer::new(width, height),
            width,
            height,
            needs_full_repaint: true,
            pointer_shape: PointerShape::Default,
            pointer_dialect: PointerDialect::detect(),
            pointer_shapes: std::env::var("SVELTUI_POINTER_SHAPES")
                .map(|v| v != "0")
                .unwrap_or(true),
        })
    }

    fn set_cleanup_hook() {
        let default_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            Self::restore_terminal();
            default_hook(panic_info);
        }));
    }

    /// Turns mouse pointer shapes on or off. They are on by default;
    /// `SVELTUI_POINTER_SHAPES=0` starts them off.
    pub fn set_pointer_shapes(&mut self, enabled: bool) {
        self.pointer_shapes = enabled;
    }

    /// Overrides the pointer shape names detected for this terminal.
    pub fn set_pointer_dialect(&mut self, dialect: PointerDialect) {
        self.pointer_dialect = dialect;
    }

    /// Shows `shape` over the terminal, writing only when it changes.
    pub fn set_pointer_shape(&mut self, shape: PointerShape) -> io::Result<()> {
        if !self.pointer_shapes || shape == self.pointer_shape {
            return Ok(());
        }
        self.pointer_shape = shape;
        if shape != PointerShape::Default {
            POINTER_SHAPE_USED.store(true, Ordering::SeqCst);
        }
        write_pointer_shape(&mut self.stdout, shape, self.pointer_dialect)
    }

    pub fn size(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
        self.curr_buffer.resize(width, height);
        self.prev_buffer.resize(width, height);
        refresh_cell_pixels();
        self.needs_full_repaint = true;
    }

    pub fn flush(&mut self) -> io::Result<()> {
        let plan = plan_images(
            &self.prev_buffer.images,
            &self.curr_buffer.images,
            |rect| self.curr_buffer.differs_in(&self.prev_buffer, rect),
            self.needs_full_repaint,
        );
        for rect in &plan.repaint {
            self.prev_buffer.invalidate(*rect);
        }
        for bytes in &plan.erase {
            self.stdout.write_all(bytes)?;
        }
        write_buffer_diff(
            &mut self.stdout,
            &self.curr_buffer,
            &self.prev_buffer,
            self.needs_full_repaint,
        )?;
        write_images(
            &mut self.stdout,
            &self.curr_buffer.images,
            &plan.draw,
            cell_pixels().unwrap_or((8, 16)),
        )?;
        self.needs_full_repaint = false;
        self.prev_buffer.cells.clone_from(&self.curr_buffer.cells);
        self.prev_buffer.images.clone_from(&self.curr_buffer.images);
        Ok(())
    }

    fn restore_terminal() {
        if !CLEANED_UP.swap(true, Ordering::SeqCst) {
            let mut stdout = io::stdout();
            let _ = execute!(
                stdout,
                ResetColor,
                SetAttribute(Attribute::Reset),
                cursor::SetCursorStyle::DefaultUserShape,
                cursor::Show,
                DisableMouseCapture,
                LeaveAlternateScreen
            );
            if POINTER_SHAPE_USED.load(Ordering::SeqCst) {
                let _ = write_pointer_shape(
                    &mut stdout,
                    PointerShape::Default,
                    PointerDialect::detect(),
                );
            }
            let _ = stdout.flush();
            let _ = terminal::disable_raw_mode();
        }
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        Self::restore_terminal();
    }
}

/// Draws the placements at `draw`, each at its own cell.
/// Asks the terminal how big a cell is, in pixels.
fn refresh_cell_pixels() {
    if let Ok(size) = terminal::window_size()
        && size.columns > 0
        && size.rows > 0
    {
        crate::terminal::set_cell_pixels(size.width / size.columns, size.height / size.rows);
    }
}

fn write_images(
    mut stdout: impl Write,
    images: &[crate::terminal::image::ImagePlacement],
    draw: &[usize],
    cell_px: (u16, u16),
) -> io::Result<()> {
    for &i in draw {
        let image = &images[i];
        queue!(stdout, MoveTo(image.at.0, image.at.1))?;
        stdout.write_all(&image.paint.draw(image.size, image.visible, cell_px))?;
    }
    stdout.flush()
}

// Keep terminal output independent of raw-mode setup so the actual emitted
// commands can be verified alongside the in-memory buffer.
fn write_buffer_diff(
    stdout: impl Write,
    curr_buffer: &Buffer,
    prev_buffer: &Buffer,
    force_redraw: bool,
) -> io::Result<()> {
    let mut stdout = io::BufWriter::new(stdout);
    let mut current_fg = None;
    let mut current_bg = None;
    let mut current_bold = false;
    let mut current_italic = false;
    let mut current_underline = false;

    for y in 0..curr_buffer.height {
        for x in 0..curr_buffer.width {
            let curr_cell = curr_buffer.get(x, y).copied().unwrap_or(Cell::default());
            let prev_cell = prev_buffer.get(x, y).copied().unwrap_or(Cell::default());

            if force_redraw || curr_cell != prev_cell {
                queue!(stdout, MoveTo(x, y))?;

                if curr_cell.fg != current_fg {
                    match curr_cell.fg {
                        Some(color) => queue!(stdout, SetForegroundColor(color))?,
                        None => queue!(stdout, SetForegroundColor(Color::Reset))?,
                    }
                    current_fg = curr_cell.fg;
                }

                if curr_cell.bg != current_bg {
                    match curr_cell.bg {
                        Some(color) => queue!(stdout, SetBackgroundColor(color))?,
                        None => queue!(stdout, SetBackgroundColor(Color::Reset))?,
                    }
                    current_bg = curr_cell.bg;
                }

                if curr_cell.bold != current_bold {
                    if curr_cell.bold {
                        queue!(stdout, SetAttribute(Attribute::Bold))?;
                    } else {
                        queue!(stdout, SetAttribute(Attribute::NormalIntensity))?;
                    }
                    current_bold = curr_cell.bold;
                }

                if curr_cell.italic != current_italic {
                    if curr_cell.italic {
                        queue!(stdout, SetAttribute(Attribute::Italic))?;
                    } else {
                        queue!(stdout, SetAttribute(Attribute::NoItalic))?;
                    }
                    current_italic = curr_cell.italic;
                }

                if curr_cell.underline != current_underline {
                    if curr_cell.underline {
                        queue!(stdout, SetAttribute(Attribute::Underlined))?;
                    } else {
                        queue!(stdout, SetAttribute(Attribute::NoUnderline))?;
                    }
                    current_underline = curr_cell.underline;
                }

                queue!(stdout, Print(curr_cell.ch))?;
            }
        }
    }

    queue!(stdout, ResetColor, SetAttribute(Attribute::Reset))?;

    if let Some((cx, cy)) = curr_buffer.cursor {
        if cx < curr_buffer.width && cy < curr_buffer.height {
            if let Some(style) = curr_buffer.cursor_style {
                queue!(stdout, style.to_crossterm())?;
            }
            queue!(stdout, MoveTo(cx, cy), cursor::Show)?;
        } else {
            queue!(stdout, cursor::Hide)?;
        }
    } else {
        queue!(stdout, cursor::Hide)?;
    }

    stdout.flush()?;

    Ok(())
}

#[cfg(test)]
mod backend_tests;
