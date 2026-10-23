#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod alert;
pub mod appearance;
pub mod application;
pub mod application_host;
pub mod blur;
pub mod callbacks;
pub mod cursor;
pub mod document_controller;
pub mod drag;
pub mod error;
pub mod event;
pub mod ffi;
pub mod host_view;
pub mod host_window;
pub mod image;
pub mod main_thread;
pub mod menu;
mod object;
pub mod panel;
pub mod pasteboard;
pub mod screen;
pub mod scroller;
pub mod sound;
pub mod text_input;
mod util;
pub mod view;
pub mod window;
pub mod window_ops;
pub mod window_state;
pub mod workspace;

pub use appearance::AppearanceName;
pub use apple_cf::cg::{CGPoint, CGRect, CGSize};
pub use application::{AboutPanelOptions, ActivationPolicy, Application};
pub use callbacks::{
    CallbackRegistration, CallbackState, CallbackTableHeader, CallbackTableV1, HandlerSlot,
    NativeCallbackRegistration, RegistrationToken, Utf16Range,
};
pub use error::AppKitError;
pub use event::{Event, EventRef};
pub use image::Image;
pub use main_thread::MainThreadMarker;
pub use screen::Screen;
pub use scroller::{Scroller, ScrollerStyle};
pub use view::View;
pub use window::{AnimationBehavior, TitleVisibility, TitlebarSeparatorStyle, Window};
pub use workspace::Workspace;

pub mod prelude {
    pub use crate::appearance::AppearanceName;
    pub use crate::application::{AboutPanelOptions, ActivationPolicy, Application};
    pub use crate::callbacks::CallbackState;
    pub use crate::error::AppKitError;
    pub use crate::event::{Event, EventRef};
    pub use crate::image::Image;
    pub use crate::main_thread::MainThreadMarker;
    pub use crate::screen::Screen;
    pub use crate::scroller::{Scroller, ScrollerStyle};
    pub use crate::view::View;
    pub use crate::window::{AnimationBehavior, TitleVisibility, TitlebarSeparatorStyle, Window};
    pub use crate::workspace::Workspace;
    pub use crate::Utf16Range;
    pub use apple_cf::cg::{CGPoint, CGRect, CGSize};
}
