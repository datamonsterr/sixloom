//! The only module that talks to the backend. SXL_API_MODE=mock serves it from `mocks`.

#[derive(Debug, Clone, PartialEq)]
pub struct Note {
    pub id: String,
    pub text: String,
    pub created_at: String,
}

#[derive(Debug)]
pub struct ApiError {
    pub code: String,
    pub message: String,
}

pub trait Api {
    fn list_notes(&self) -> Result<Vec<Note>, ApiError>;
}

pub fn from_env() -> Box<dyn Api> {
    match std::env::var("SXL_API_MODE").as_deref() {
        Ok("mock") => Box::new(crate::mocks::MockClient::new(crate::mocks::Scenario::from_env())),
        _ => unimplemented!("HttpClient arrives in the build loom"),
    }
}
