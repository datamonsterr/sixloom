//! Mock client and named scenarios for the preview (SXL_SCENARIO=ready|empty).
use crate::client::{Api, ApiError, Note};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Scenario {
    Ready,
    Empty,
}

impl Scenario {
    pub fn from_env() -> Scenario {
        match std::env::var("SXL_SCENARIO").as_deref() {
            Ok("empty") => Scenario::Empty,
            _ => Scenario::Ready,
        }
    }
}

pub struct MockClient {
    scenario: Scenario,
}

impl MockClient {
    pub fn new(scenario: Scenario) -> Self {
        MockClient { scenario }
    }
}

impl Api for MockClient {
    fn list_notes(&self) -> Result<Vec<Note>, ApiError> {
        if self.scenario == Scenario::Empty {
            return Ok(vec![]);
        }
        let mut notes = vec![
            Note { id: "0b8e9d3c-4f21-4a7e-8c55-2d6e7f8a9b01".into(), text: "Buy coffee".into(), created_at: "2026-10-25T18:30:00Z".into() },
            Note { id: "6f1c2e1a-2b7d-4a55-9d0e-1f2a3b4c5d6e".into(), text: "Call landlord".into(), created_at: "2026-10-26T09:00:00Z".into() },
        ];
        notes.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(notes)
    }
}
