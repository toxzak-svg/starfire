//! Neural Module Implementations
//!
//! Each module from Starfire's architecture is implemented as a neuron.

pub mod quanot_neuron;
pub mod knowledge_neuron;
pub mod reasoning_neuron;
pub mod causal_neuron;
pub mod goals_neuron;
pub mod fewshot_neuron;
pub mod curriculum_neuron;
pub mod worldmodel_neuron;

pub use quanot_neuron::QuanotNeuron;
pub use knowledge_neuron::KnowledgeNeuron;
pub use reasoning_neuron::ReasoningNeuron;
pub use causal_neuron::CausalNeuron;
pub use goals_neuron::GoalsNeuron;
pub use fewshot_neuron::FewShotNeuron;
pub use curriculum_neuron::CurriculumNeuron;
pub use worldmodel_neuron::WorldModelNeuron;
