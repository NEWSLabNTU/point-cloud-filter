use crate::{Config, Filter};
use kiss3d::window::Window;

impl Config {
    pub fn render_kiss3d(&self, window: &mut Window) {
        if let Some(range_filter) = &self.range_filter {
            range_filter.render_kiss3d(window);
        }
    }
}

impl Filter {
    pub fn render_kiss3d(&self, window: &mut Window) {
        self.config().render_kiss3d(window);
    }
}
