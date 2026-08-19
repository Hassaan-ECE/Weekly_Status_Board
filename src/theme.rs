use crate::model::ThemeMode;
use gpui::{rgb, Rgba};

#[derive(Clone, Copy)]
pub struct Theme {
    pub background: Rgba,
    pub foreground: Rgba,
    pub card: Rgba,
    pub border: Rgba,
    pub muted: Rgba,
    pub fill_4: Rgba,
    pub primary: Rgba,
    pub primary_fg: Rgba,
    pub target_header: Rgba,
    pub progress_header: Rgba,
    pub done_header: Rgba,
    pub header_fg: Rgba,
}

pub fn light() -> Theme {
    Theme {
        background: rgb(0xffffff),
        foreground: rgb(0x262626),
        card: rgb(0xffffff),
        border: Rgba {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.08,
        },
        muted: rgb(0x686868),
        fill_4: Rgba {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.04,
        },
        primary: rgb(0x5558E6),
        primary_fg: rgb(0xffffff),
        target_header: rgb(0x5558E6),
        progress_header: rgb(0xB45309),
        done_header: rgb(0x047857),
        header_fg: rgb(0xffffff),
    }
}

pub fn dark() -> Theme {
    Theme {
        background: rgb(0x0C0C0C),
        foreground: rgb(0xF5F5F5),
        card: rgb(0x101010),
        border: Rgba {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 0.06,
        },
        muted: rgb(0x7E7E7E),
        fill_4: Rgba {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 0.04,
        },
        primary: rgb(0x7A7DF0),
        primary_fg: rgb(0xffffff),
        target_header: rgb(0x5558E6),
        progress_header: rgb(0xB45309),
        done_header: rgb(0x047857),
        header_fg: rgb(0xffffff),
    }
}

pub fn for_mode(mode: ThemeMode) -> Theme {
    match mode {
        ThemeMode::Dark => dark(),
        ThemeMode::Light => light(),
    }
}
