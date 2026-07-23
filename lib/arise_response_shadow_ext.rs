//! Opt-in response-boundary hook for the ARISE-A1 semantic-program shadow.
//!
//! The extension consumes an already-constructed response, observes its final
//! body against a validated semantic program, and returns the exact same response.

use crate::arise_edge::arise_typed_plan_shadow::observe_semantic_program;
use crate::language_realization::LexicalBindingTable;
use crate::runtime::response_intent::Response;
use crate::semantic_response::SemanticResponseProgram;

pub trait ResponseSemanticShadowExt: Sized {
    /// Observe the final response body without mutating text or metadata.
    #[must_use]
    fn observe_semantic_shadow(
        self,
        program: &SemanticResponseProgram,
        lexical_table: &LexicalBindingTable,
    ) -> Self;
}

impl ResponseSemanticShadowExt for Response {
    fn observe_semantic_shadow(
        self,
        program: &SemanticResponseProgram,
        lexical_table: &LexicalBindingTable,
    ) -> Self {
        let _ = observe_semantic_program(program, lexical_table, &self.body);
        self
    }
}
