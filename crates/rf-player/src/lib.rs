//! Önizleme için saf Rust oynatıcı: FilmCraft çözücüleriyle açma ve çözme, ses çıkışı, video çizimi.

mod audio_out;
mod clock;
mod convert;
mod player;
// D3D11 video output (Windows); on Linux the preview draws `to_bgra` frames itself.
#[cfg(windows)]
mod present;
mod source;

pub use audio_out::{AudioSink, NullSink, wasapi_sink};
pub use clock::Clock;
pub use convert::to_bgra;
pub use player::{Player, PlayerState};
#[cfg(windows)]
pub use present::{Presenter, render_offscreen};
pub use source::{Frame, Media, Planes, can_open, open};
