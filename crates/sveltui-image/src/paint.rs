//! Draws an image with a terminal graphics protocol, beside the cell grid.

use image::{RgbaImage, imageops};
use std::cell::RefCell;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use sveltui::layout::Rect;
use sveltui::terminal::ImagePaint;

use crate::image_render::canvas;
use crate::image_types::ImageFit;
use crate::protocol::ImageProtocol;

thread_local! {
    /// The few most recent canvases, so scrolling an image doesn't scale it
    /// again on every frame.
    static CANVASES: RefCell<Vec<(u64, Rc<RgbaImage>)>> = const { RefCell::new(Vec::new()) };
}

const CANVAS_SLOTS: usize = 4;

pub(crate) struct GraphicsPaint {
    pub protocol: ImageProtocol,
    pub source: Rc<RgbaImage>,
    pub fit: ImageFit,
    pub background: [u8; 3],
    pub cells: (u16, u16),
    pub cell_px: (u16, u16),
}

impl GraphicsPaint {
    fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        (Rc::as_ptr(&self.source) as usize).hash(&mut hasher);
        (self.protocol as u8).hash(&mut hasher);
        (self.fit as u8).hash(&mut hasher);
        self.background.hash(&mut hasher);
        self.cells.hash(&mut hasher);
        self.cell_px.hash(&mut hasher);
        hasher.finish()
    }

    #[cfg(feature = "kitty")]
    fn id(&self) -> u32 {
        // Kitty ids are non-zero 32-bit numbers.
        (self.hash() % 0x00FF_FFFF) as u32 + 1
    }

    fn canvas(&self) -> Rc<RgbaImage> {
        let key = self.hash();
        let hit = CANVASES.with(|c| {
            c.borrow()
                .iter()
                .find(|(k, _)| *k == key)
                .map(|(_, img)| img.clone())
        });
        if let Some(img) = hit {
            return img;
        }
        let (cw, ch) = (u32::from(self.cell_px.0), u32::from(self.cell_px.1));
        let img = Rc::new(canvas(
            &self.source,
            u32::from(self.cells.0) * cw,
            u32::from(self.cells.1) * ch,
            self.fit,
            self.background,
        ));
        CANVASES.with(|c| {
            let mut c = c.borrow_mut();
            if c.len() == CANVAS_SLOTS {
                c.remove(0);
            }
            c.push((key, img.clone()));
        });
        img
    }
}

impl ImagePaint for GraphicsPaint {
    fn key(&self) -> u64 {
        self.hash()
    }

    #[allow(unused_variables)]
    fn draw(&self, size: (u16, u16), visible: Rect, cell_px: (u16, u16)) -> Vec<u8> {
        let canvas = self.canvas();
        let (cw, ch) = (u32::from(cell_px.0), u32::from(cell_px.1));
        // Only the part that shows is sent: a protocol can't clip an image
        // to a scroll window, but it can draw less of it.
        let crop = imageops::crop_imm(
            &*canvas,
            u32::from(visible.x) * cw,
            u32::from(visible.y) * ch,
            u32::from(visible.width) * cw,
            u32::from(visible.height) * ch,
        )
        .to_image();
        match self.protocol {
            #[cfg(feature = "kitty")]
            ImageProtocol::Kitty => {
                crate::encode::kitty::encode(&crop, visible.width, visible.height, self.id())
            }
            #[cfg(feature = "iterm2")]
            ImageProtocol::Iterm2 => {
                crate::encode::iterm2::encode(&crop, visible.width, visible.height)
            }
            #[cfg(feature = "sixel")]
            ImageProtocol::Sixel => crate::encode::sixel::encode(&crop),
            _ => Vec::new(),
        }
    }

    fn erase(&self) -> Option<Vec<u8>> {
        match self.protocol {
            #[cfg(feature = "kitty")]
            ImageProtocol::Kitty => Some(crate::encode::kitty::erase(self.id())),
            _ => None,
        }
    }
}
