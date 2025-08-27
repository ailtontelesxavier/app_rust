mod model;
mod repository;
mod router;
mod schema;
mod service;
mod view;
mod filters;

pub use model::StatusChamado;
pub use router::router;
pub use service::ChamadoService;
pub use filters::status_filter;
