use std::sync::Arc;

use crate::emblem::render::ShapeCache;
use crate::paths::Paths;

pub struct AppState {
    pub paths: Paths,
    pub shapes: Arc<ShapeCache>,
}
