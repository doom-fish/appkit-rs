#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod appearance;
pub mod application;
pub mod error;
pub mod ffi;
pub mod image;
pub mod main_thread;
mod object;
pub mod screen;
pub mod scroller;
mod util;
pub mod view;
pub mod window;
pub mod workspace;

pub use apple_cf::cg::{CGPoint, CGRect, CGSize};
pub use appearance::AppearanceName;
pub use application::{AboutPanelOptions, Application};
pub use error::AppKitError;
pub use image::Image;
pub use main_thread::MainThreadMarker;
pub use screen::Screen;
pub use scroller::{Scroller, ScrollerStyle};
pub use view::View;
pub use window::{AnimationBehavior, TitleVisibility, TitlebarSeparatorStyle, Window};
pub use workspace::Workspace;

pub mod prelude {
    pub use crate::appearance::AppearanceName;
    pub use crate::application::{AboutPanelOptions, Application};
    pub use crate::error::AppKitError;
    pub use crate::image::Image;
    pub use crate::main_thread::MainThreadMarker;
    pub use crate::screen::Screen;
    pub use crate::scroller::{Scroller, ScrollerStyle};
    pub use crate::view::View;
    pub use crate::window::{AnimationBehavior, TitleVisibility, TitlebarSeparatorStyle, Window};
    pub use crate::workspace::Workspace;
    pub use apple_cf::cg::{CGPoint, CGRect, CGSize};
}
