use tui::backend::Backend;
use tui::Frame;

use crate::app::App;

pub(crate) fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {}
