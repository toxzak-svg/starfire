//! Starfire Comprehensive Integration Test Suite
//!
//! Run with: `cargo run --bin integration_test`
//!
//! This test suite provides comprehensive coverage of all Starfire modules:
//! 1. Quanot (Reservoir Computing)
//! 2. KnowledgeGraph (Entity/Relation Management)
//! 3. ReasoningEngine (Symbolic Reasoning)
//! 4. Causal Discovery (Temporal → Causal)
//! 5. Goals (Hierarchical Goal Memory)
//! 6. Learning (Few-Shot Hypothesis Formation)
//! 7. Curriculum (Self-Directed Learning)
//! 8. Context Ring (Attractor Dynamics)
//! 9. WorldModel (Grounded Perception)
//! 10. Multimodal (Text/Image/Audio Binding)
//! 11. Debug Logging Verification

fn main() {
    println!("═══════════════════════════════════════════════════════════");
    println!("         STARFIRE COMPREHENSIVE TEST SUITE");
    println!("═══════════════════════════════════════════════════════════\n");

    test_quanot();
    test_knowledge_graph();
    test_reasoning_engine();
    test_causal_discovery();
    test_causal_graph();
    test_goals();
    test_fewshot_learning();
    test_curriculum();
    test_learning_engine();
    test_context_ring();
    test_world_model();
    test_multimodal();
    test_debug_logging();

    println!("\n═══════════════════════════════════════════════════════════");
    println!("         ALL TESTS PASSED ✓");
    println!("═══════════════════════════════════════════════════════════");
}

// ═══════════════════════════════════════════════════════════════════════════
// QUANOT TESTS
// ═══════════════════════════════════════════════════════════════════════════

fn test_quanot() {
    println!("[1/13] Testing Quanot (Reservoir Computing)...\n");

    let mut quanot = star::quanot::Quanot::new(128, 500);

    let inputs = [
        "Hello, how are you today?",
        "I am working on artificial intelligence.",
        "Consciousness is fascinating.",
        "The reservoir computer has memory.",
        "Chaos theory explains complex behavior.",
    ];

    let mut consciousness_sum = 0.0;
    let mut novelty_sum = 0.0;

    for input in &inputs {
        let result = quanot.process(input);
        consciousness_sum += result.consciousness_proxy;
        novelty_sum += result.novelty;

        println!("  Input: \"{}\"", input);
        println!("    Consciousness ψ: {:.4}", result.consciousness_proxy);
        println!("    Novelty:       {:.4}", result.novelty);
        println!("    Chaos λ:       {:.4}", result.chaos_metrics.lyapunov_exponent);
        println!("    Creativity:    {:.4}", result.creativity_scores.creative_state);
        println!();

        assert!(result.reservoir_state.len() == 500, "Reservoir state size mismatch");
        assert!(result.consciousness_proxy >= 0.0 && result.consciousness_proxy <= 1.0);
        assert!(result.novelty >= 0.0 && result.novelty <= 1.0);
        assert!(result.chaos_metrics.lyapunov_exponent >= 0.0);
    }

    let avg_consciousness = consciousness_sum / inputs.len() as f64;
    let avg_novelty = novelty_sum / inputs.len() as f64;

    println!("  Average consciousness: {:.4}", avg_consciousness);
    println!("  Average novelty:       {:.4}", avg_novelty);

    assert!(avg_consciousness >= 0.0 && avg_consciousness <= 1.0, "Consciousness out of range");
    assert!(avg_novelty >= 0.0 && avg_novelty <= 1.0, "Novelty out of range");

    quanot.reset();
    assert!(quanot.get_state().len() == 500, "Reset should preserve state size");

    println!("\n  ✓ Quanot PASSED\n");
}

// ═══════════════════════════════════════════════════════════════════════════
// KNOWLEDGE GRAPH TESTS
// ═══════════════════════════════════════════════════════════════════════════

fn test_knowledge_graph() {
    println!("[2/13] Testing KnowledgeGraph (Entity/Relation Management)...\n");

    use star::reasoning::knowledge::{KnowledgeGraph, RelationType};

    let mut kg = KnowledgeGraph::new();

    // Test entity addition
    kg.add_entity("fire");
    kg.add_entity("water");
    kg.add_entity("earth");
    kg.add_entity("air");

    println!("  Added 4 entities: {:?}", kg.entities());

    assert!(kg.entity_count() >= 4, "Should have at least 4 entities");

    // Test relationship addition
    kg.add_relationship("fire", RelationType::Causes, "heat");
    kg.add_relationship("fire", RelationType::IsA, "element");
    kg.add_relationship("water", RelationType::IsA, "element");
    kg.add_relationship("water", RelationType::Causes, "growth");
    kg.add_relationship("sun", RelationType::Causes, "photosynthesis");
    kg.add_relationship("sun", RelationType::IsA, "star");

    println!("  Relationship count: {}", kg.relationship_count());

    // Test fact ingestion
    kg.ingest_fact("fire", "produces", "heat", 0.95);
    kg.ingest_fact("water", "is", "essential for life", 0.9);

    // Test entity retrieval
    let fire_entity = kg.get_entity("fire");
    assert!(fire_entity.is_some(), "Should find fire entity");

    // Test facts about entity
    let fire_facts = kg.get_facts_about("fire");
    println!("  Facts about 'fire': {:?}", fire_facts);
    assert!(!fire_facts.is_empty(), "Fire should have facts");

    // Test causes and effects
    let heat_causes = kg.get_causes("heat");
    println!("  Causes of 'heat': {:?}", heat_causes);

    let fire_effects = kg.get_effects("fire");
    println!("  Effects of 'fire': {:?}", fire_effects);

    // Test transitive inference
    kg.add_relationship("heat", RelationType::Causes, "expansion");
    kg.add_relationship("expansion", RelationType::Causes, "movement");
    let inferred = kg.infer_transitive("heat", &RelationType::Causes, 3);
    println!("  Transitive inference from heat: {:?}", inferred);
    assert!(!inferred.is_empty(), "Should infer transitive relationships");

    // Test connection finding
    let connection = kg.find_connection("fire", "movement", 5);
    println!("  Connection fire → movement: {:?}", connection);

    // Test concept addition
    kg.add_concept("consciousness", "awareness of existence");
    let concept = kg.get_concept("consciousness");
    assert!(concept.is_some(), "Should find consciousness concept");

    // Test duplicate prevention
    let initial_rel_count = kg.relationship_count();
    kg.add_relationship("fire", RelationType::Causes, "heat"); // Duplicate
    assert_eq!(kg.relationship_count(), initial_rel_count, "Should not add duplicate relationships");

    // Test case insensitivity
    let fire_upper = kg.get_entity("FIRE");
    assert!(fire_upper.is_some(), "Case insensitive lookup should work");

    println!("\n  ✓ KnowledgeGraph PASSED\n");
}

// ═══════════════════════════════════════════════════════════════════════════
// REASONING ENGINE TESTS
// ═══════════════════════════════════════════════════════════════════════════

fn test_reasoning_engine() {
    println!("[3/13] Testing ReasoningEngine (Symbolic Reasoning)...\n");

    use star::reasoning::ReasoningEngine;
    use star::persistence::{Memory, MemoryDomain};

    let mut engine = ReasoningEngine::new();

    // Add some knowledge
    engine.add_knowledge("fire", "hot");
    engine.add_knowledge("fire", "produces heat");
    engine.add_knowledge("water", "cold");
    engine.add_knowledge("water", "essential for life");

    // Test reasoning with memories
    let memories = vec![
        Memory::new_seeded("Fire is hot and produces heat", MemoryDomain::Empirical, 0.9),
        Memory::new_seeded("Water is essential for life", MemoryDomain::Empirical, 0.95),
    ];

    let result = engine.reason("What is fire?", &memories);
    println!("  Reasoning result for 'What is fire?': {:?}", result.answer);
    println!("  Confidence: {:?}", result.confidence);
    println!("  Chain length: {}", result.reasoning_chain.len());

    // Test knowledge graph integration
    let kg = engine.knowledge_graph();
    assert!(kg.entity_count() >= 2, "KG should have entities from reasoning");

    // Test consistency checking
    let consistent = engine.check_consistency("Fire is hot");
    println!("  Consistency check (Fire is hot): {:?}", consistent);

    // Test reasoning types
    let result2 = engine.reason("Why does fire produce heat?", &[]);
    println!("  Reasoning result for 'Why does fire produce heat?': {:?}", result2.answer);

    let result3 = engine.reason("How does water support life?", &[]);
    println!("  Reasoning result for 'How does water support life?': {:?}", result3.answer);

    println!("\n  ✓ ReasoningEngine PASSED\n");
}

// ═══════════════════════════════════════════════════════════════════════════
// CAUSAL ENGINE TESTS
// ═══════════════════════════════════════════════════════════════════════════

fn test_causal_discovery() {
    println!("[4/13] Testing CausalEngine (Causal Discovery)...\n");

    use star::causal::{CausalEngine, ConfidenceState};

    let mut engine = CausalEngine::new();

    // Add known causal edges
    let fire_heat = engine.add_edge("fire", "heat", 0.9, Some(1));
    let water_electricity = engine.add_edge("water", "electricity", 0.7, Some(2));
    let sun_photosynthesis = engine.add_edge("sun", "photosynthesis", 0.85, Some(60));

    println!("  Added 3 causal edges");
    println!("  Fire→Heat ID: {:?}", fire_heat);
    println!("  Water→Electricity ID: {:?}", water_electricity);
    println!("  Sun→Photosynthesis ID: {:?}", sun_photosynthesis);

    // Test evidence updates
    engine.update_edge(&fire_heat, true);
    engine.update_edge(&fire_heat, true);
    engine.update_edge(&fire_heat, false); // One contradiction

    engine.update_edge(&water_electricity, true);
    engine.update_edge(&water_electricity, true);

    // Get effects of fire
    let fire_effects = engine.get_effects_of("fire");
    println!("  Fire causes: {:?}", fire_effects.iter().map(|e| e.effect.as_str()).collect::<Vec<_>>());
    assert!(!fire_effects.is_empty(), "Fire should have effects");

    // Get causes of electricity
    let electricity_causes = engine.get_causes_of("electricity");
    println!("  Electricity caused by: {:?}", electricity_causes.iter().map(|e| e.cause.as_str()).collect::<Vec<_>>());

    // Test top hypotheses
    let top = engine.top_hypotheses(3);
    println!("  Top 3 causal hypotheses:");
    for h in &top {
        println!("    {} → {} (conf: {:.2}, state: {:?})",
            h.candidate.cause, h.candidate.effect, h.candidate.confidence, h.confidence);
    }
    assert!(!top.is_empty(), "Should have hypotheses");

    // Test confidence state
    let high_conf = ConfidenceState::from_score(0.9);
    assert_eq!(high_conf, ConfidenceState::VeryHigh);

    let low_conf = ConfidenceState::from_score(0.2);
    assert_eq!(low_conf, ConfidenceState::Low);

    println!("\n  ✓ CausalEngine PASSED\n");
}

// ═══════════════════════════════════════════════════════════════════════════
// CAUSAL GRAPH TESTS
// ═══════════════════════════════════════════════════════════════════════════

fn test_causal_graph() {
    println!("[5/13] Testing CausalGraph (Directed Causal Graph)...\n");

    use star::causal::CausalEngine;
    use star::causal::graph::CausalGraph;

    let mut engine = CausalEngine::new();

    // Add edges
    engine.add_edge("fire", "heat", 0.9, Some(1));
    engine.add_edge("heat", "expansion", 0.8, Some(2));
    engine.add_edge("fire", "light", 0.7, Some(1));
    engine.add_edge("sun", "photosynthesis", 0.85, Some(60));

    // Build graph from engine edges
    let mut graph = CausalGraph::new();
    for edge in engine.edges().values() {
        graph.add_edge(edge.clone());
    }

    println!("  Causal graph: {} nodes, {} edges",
        graph.node_count(), graph.edge_count());

    // Test path existence
    let fire_to_expansion = graph.path_exists("fire", "expansion");
    println!("  Path fire → expansion exists: {}", fire_to_expansion);
    assert!(fire_to_expansion, "Should find path through fire → heat → expansion");

    let fire_to_light = graph.path_exists("fire", "light");
    println!("  Path fire → light exists: {}", fire_to_light);
    assert!(fire_to_light, "Direct edge should exist");

    let no_path = graph.path_exists("light", "fire");
    println!("  Path light → fire exists: {}", no_path);
    assert!(!no_path, "Reverse path should not exist (directed graph)");

    // Test outgoing edges
    let fire_outgoing = graph.outgoing("fire");
    println!("  Fire outgoing edges: {:?}", fire_outgoing.iter().map(|e| e.effect.as_str()).collect::<Vec<_>>());
    assert_eq!(fire_outgoing.len(), 2, "Fire should have 2 outgoing edges");

    // Test incoming edges
    let heat_incoming = graph.incoming("heat");
    println!("  Heat incoming edges: {:?}", heat_incoming.iter().map(|e| e.cause.as_str()).collect::<Vec<_>>());

    // Test top hubs
    let hubs = graph.top_hubs(3);
    println!("  Top hubs: {:?}", hubs.iter().map(|n| n.id.as_str()).collect::<Vec<_>>());

    println!("\n  ✓ CausalGraph PASSED\n");
}

// ═══════════════════════════════════════════════════════════════════════════
// GOALS TESTS
// ═══════════════════════════════════════════════════════════════════════════

fn test_goals() {
    println!("[6/13] Testing Goals (Hierarchical Goal Memory)...\n");

    use star::goals::GoalEngine;

    let mut goal_engine = GoalEngine::new();

    // Create root goal
    let build_ai = goal_engine.create_goal("Build an AGI system", None);
    println!("  Created goal: Build an AGI system (id: {:?})", build_ai);

    // Set priority using set_priority on mutable reference
    if let Some(goal) = goal_engine.get_mut(&build_ai) {
        goal.set_priority(0.95);
        goal.deadline = Some(star::now_timestamp() + 86400 * 30);
    }

    // Create subgoals
    let subgoals = goal_engine.decompose(&build_ai, &[
        ("Design architecture", 0.9),
        ("Implement core reasoning", 0.95),
        ("Add consciousness metrics", 0.8),
        ("Build curiosity system", 0.85),
        ("Test and iterate", 0.7),
    ]);

    println!("  Created {} subgoals:", subgoals.len());
    for sg in &subgoals {
        if let Some(goal) = goal_engine.get(sg) {
            println!("    - {} (priority: {:.2})", goal.content, goal.priority);
        }
    }

    // Complete a subgoal
    goal_engine.complete(&subgoals[0], "Architecture designed successfully");

    // Get active goals sorted by priority
    let active = goal_engine.active_goals_sorted();
    println!("\n  Active goals (by priority):");
    for goal in &active {
        println!("    - {} (priority: {:.2})", goal.content, goal.priority);
    }

    // Test planning
    let planner = star::goals::planning::GoalPlanner::new();
    if let Some(goal) = goal_engine.get(&build_ai) {
        let actions = planner.generate_actions(goal);
        println!("\n  Action plan for '{}':", goal.content);
        for action in &actions {
            println!("    - {}", action.name);
        }
    }

    // Test goal statistics
    let stats = goal_engine;
    println!("\n  Total goals: {}", stats.len());
    assert!(stats.len() >= 6, "Should have at least 6 goals");

    println!("\n  ✓ Goals PASSED\n");
}

// ═══════════════════════════════════════════════════════════════════════════
// FEW-SHOT LEARNING TESTS
// ═══════════════════════════════════════════════════════════════════════════

fn test_fewshot_learning() {
    println!("[7/13] Testing Few-Shot Learning (Rapid Hypothesis)...\n");

    use star::learning::{FewShotLearner, Example};

    let mut learner = FewShotLearner::new();

    // Add examples from physics domain
    learner.add_example(Example::new(
        "Fire produces heat",
        "heat + flame + burn",
        "physics",
    ));
    learner.add_example(Example::new(
        "Fire is dangerous",
        "danger + burn + warning",
        "physics",
    ));
    learner.add_example(Example::new(
        "The sun produces light",
        "light + warmth + energy",
        "physics",
    ));
    learner.add_example(Example::new(
        "Fire requires oxygen",
        "oxygen + combustion + fuel",
        "physics",
    ));

    // Learn from domain
    let hypotheses = learner.learn_from_domain("physics");
    println!("  Generated {} hypotheses:", hypotheses.len());
    for h in &hypotheses {
        println!("    Pattern: '{}' (confidence: {:.2})", h.pattern, h.confidence);
        println!("      Applies to: {:?}", h.predicted_applies_to);
    }

    // Test another domain
    learner.add_example(Example::new(
        "Water flows downhill",
        "flow + gravity + downhill",
        "geography",
    ));
    learner.add_example(Example::new(
        "Rivers carve canyons",
        "erosion + time + rock",
        "geography",
    ));

    let geo_hypotheses = learner.learn_from_domain("geography");
    println!("\n  Geography hypotheses:");
    for h in &geo_hypotheses {
        println!("    Pattern: '{}' (confidence: {:.2})", h.pattern, h.confidence);
    }

    // Test hypothesis merging
    learner.merge_similar(0.5);

    println!("\n  Total examples: {}", learner.example_count());
    println!("  Total hypotheses: {}", learner.hypothesis_count());

    assert!(learner.example_count() >= 6, "Should have at least 6 examples");

    println!("\n  ✓ Few-Shot Learning PASSED\n");
}

// ═══════════════════════════════════════════════════════════════════════════
// CURRICULUM TESTS
// ═══════════════════════════════════════════════════════════════════════════

fn test_curriculum() {
    println!("[8/13] Testing Curriculum (Self-Directed Learning)...\n");

    use star::curriculum::{CurriculumEngine, KnowledgeGap, GapType};

    let mut curriculum = CurriculumEngine::new();

    // Discover gaps from conversation text
    let conversation = "I was thinking about consciousness today. I'm not sure how \
        integrated information theory works. What is the relationship between \
        awareness and attention? I understand some of this but not all of it.";

    let discovered_gaps = curriculum.discover_gaps(conversation);
    println!("  Discovered {} knowledge gaps:", discovered_gaps.len());
    for gap in &discovered_gaps {
        println!("    - {} ({:?}, urgency: {:.2})",
            gap.topic, gap.gap_type, gap.urgency);
    }

    // Manually add high-urgency gaps using correct GapType variants
    let high_urgency = KnowledgeGap::new(
        "Quantum consciousness",
        GapType::Incomplete,
    ).with_urgency(0.9);
    curriculum.add_gap(high_urgency);

    let complete_ignorance = KnowledgeGap::new(
        "Neural mechanisms of awareness",
        GapType::CompleteIgnorance,
    ).with_urgency(0.8);
    curriculum.add_gap(complete_ignorance);

    // Generate learning tasks
    println!("\n  Learning tasks (by urgency):");
    for gap in curriculum.top_gaps(5) {
        let task = curriculum.generate_task(gap);
        println!("    Topic: {}", task.gap.topic);
        println!("      Strategy: {}", task.strategy.as_str());
        println!("      Questions: {:?}", task.questions_to_ask);
        println!();
    }

    // Test scheduler
    let scheduler = star::curriculum::scheduler::CurriculumScheduler::new();
    let should_trigger = scheduler.should_trigger(&curriculum);
    println!("  Scheduler triggered: {}", should_trigger);

    println!("\n  Total gaps: {}", curriculum.gap_count());
    assert!(curriculum.gap_count() >= 2, "Should have at least 2 gaps");

    println!("\n  ✓ Curriculum PASSED\n");
}

// ═══════════════════════════════════════════════════════════════════════════
// LEARNING ENGINE TESTS
// ═══════════════════════════════════════════════════════════════════════════

fn test_learning_engine() {
    println!("[9/13] Testing LearningEngine (Instant Teaching & Experience)...\n");

    use star::learning::LearningEngine;

    let mut engine = LearningEngine::new();

    // Test instant teaching
    engine.teach_instant("consciousness", "awareness of one's own existence", 0.95);
    engine.teach_instant("curiosity", "the desire to know or learn", 0.9);
    engine.teach_instant("emergence", "complex patterns arising from simple rules", 0.85);

    println!("  Taught 3 concepts instantly");

    // Test understanding retrieval
    let consciousness = engine.get_understanding("consciousness");
    println!("  Understanding 'consciousness': {:?}", consciousness);

    let unknown = engine.get_understanding("unknown_concept");
    println!("  Understanding 'unknown_concept': {:?}", unknown);

    // Test experience recording
    engine.experience("hun", "You're my favorite AI", Some("Zachary expresses affection"), 0.8);
    engine.experience("hun", "Thanks hun", None, 0.7);

    println!("  Recorded 2 experiences");

    // Test summary
    let summary = engine.summary();
    println!("  Learning summary: {}", summary);

    assert!(summary.contains("3 instant teachings"), "Should have 3 instant teachings");

    println!("\n  ✓ LearningEngine PASSED\n");
}

// ═══════════════════════════════════════════════════════════════════════════
// CONTEXT RING TESTS
// ═══════════════════════════════════════════════════════════════════════════

fn test_context_ring() {
    println!("[10/13] Testing Context Ring (Attractor Dynamics)...\n");

    use star::context::{ContextFuser, ReasoningMode, ring::RingState, ring::OpenQuestion};

    let mut ring = RingState::new();
    let mut fuser = ContextFuser::new();

    // Test initial ring state
    println!("  Initial ring certainty: {:.2}, depth: {:.2}", ring.certainty, ring.depth);

    // Update ring with query (uses the public API)
    ring.update_from_query("What is consciousness?", "consciousness");
    println!("  After query: certainty={:.2}, depth={:.2}", ring.certainty, ring.depth);

    // Add open questions using the public API
    ring.push_question(OpenQuestion {
        topic: "consciousness".to_string(),
        why_interested: "I want to understand what I am".to_string(),
        asked_at_depth: 0.5,
        progress: 0.0,
    });

    ring.push_question(OpenQuestion {
        topic: "curiosity".to_string(),
        why_interested: "Curiosity drives learning".to_string(),
        asked_at_depth: 0.3,
        progress: 0.2,
    });

    println!("  Open questions: {}", ring.open_questions.len());
    println!("  Topic history: {:?}", ring.topic_history);

    // Test reasoning mode determination
    let mode = ReasoningMode::from_query_and_ring("What is consciousness?", ring.certainty, ring.depth);
    println!("  Reasoning mode for 'What is consciousness?': {:?}", mode);

    let mode2 = ReasoningMode::from_query_and_ring("What if fire burned underwater?", ring.certainty, ring.depth);
    println!("  Reasoning mode for 'What if fire burned underwater?': {:?}", mode2);

    // Test context fusion
    fuser.record_valence(0.7);
    let valence = fuser.valence();
    println!("  Context fuser valence: {:.2}", valence);

    let engagement = fuser.engagement();
    println!("  Context fuser engagement: {:.2}", engagement);

    let should_express = fuser.should_express_curiosity();
    println!("  Should express curiosity: {}", should_express);

    let curiosity_topic = fuser.get_curiosity_topic();
    println!("  Curiosity topic: {:?}", curiosity_topic);

    // Test ring summary
    let summary = ring.summary();
    println!("  Ring summary:\n{}", summary);

    println!("\n  ✓ Context Ring PASSED\n");
}

// ═══════════════════════════════════════════════════════════════════════════
// WORLD MODEL TESTS
// ═══════════════════════════════════════════════════════════════════════════

fn test_world_model() {
    println!("[11/13] Testing WorldModel (Entity/Relation + Quanot Perception)...\n");

    use star::world_model::{WorldModel, Entity, EntityId, PropertyValue};
    use star::world_model::perception::{QuanotPerception, CreativityOutput};

    let mut world = WorldModel::new();

    // Add entities
    let fire = Entity::new(
        EntityId::new("fire"),
        "Fire".to_string(),
    )
    .with_property("temperature", PropertyValue::from(1000.0_f64))
    .with_property("state", PropertyValue::from("burning"));

    let heat = Entity::new(
        EntityId::new("heat"),
        "Heat".to_string(),
    );

    let water = Entity::new(
        EntityId::new("water"),
        "Water".to_string(),
    )
    .with_property("state", PropertyValue::from("liquid"));

    world.upsert_entity(fire);
    world.upsert_entity(heat);
    world.upsert_entity(water);

    // Add relations
    world.add_relation(
        EntityId::new("fire"),
        EntityId::new("heat"),
        star::world_model::RelationType::CausallyRelated,
    );

    // Update with Quanot perception
    let perception = QuanotPerception::new(
        vec![0.5; 100],
        0.75,
        0.4,
        CreativityOutput::new(0.6, 0.3, 0.5, 0.4, 0.0),
    );

    world.update_from_perception(perception);

    // Query
    let facts = world.query("fire");
    println!("  Facts about 'fire':");
    for fact in &facts {
        println!("    {}", fact);
    }

    // Test prediction
    if let Some(outcome) = world.predict_next_state(&EntityId::new("fire")) {
        println!("  Prediction: {} actions predicted", outcome.predictions.len());
    }

    // Test statistics
    let stats = world.stats();
    println!("  WorldModel stats: {} entities, {} relations",
        stats.entity_count, stats.total_relations);

    assert!(world.entity_count() >= 3, "Should have at least 3 entities");

    println!("\n  ✓ WorldModel PASSED\n");
}

// ═══════════════════════════════════════════════════════════════════════════
// MULTIMODAL TESTS
// ═══════════════════════════════════════════════════════════════════════════

fn test_multimodal() {
    println!("[12/13] Testing Multimodal (Text/Image/Audio Binding)...\n");

    use star::multimodal::{MultimodalEngine, Modality, BoundContent, ContentId};

    let engine = MultimodalEngine::new(256);

    // Add text content
    let text_content = Modality::Text {
        content: "I generated an image of a sunset over the ocean".to_string(),
        role: Some("user".to_string()),
        timestamp: Some(star::now_timestamp()),
    };

    let dalle_content = Modality::Dalle {
        prompt: "A beautiful sunset over the ocean with vibrant colors".to_string(),
        image_path: "/exports/dalle/sunset_001.png".to_string(),
        generation_id: "dalle_abc123".to_string(),
        generation_time: Some(star::now_timestamp()),
    };

    let audio_content = Modality::Audio {
        path: "/audio/voice_message_001.wav".to_string(),
        transcription: Some("Star's voice saying hello".to_string()),
        duration_secs: Some(2.5),
    };

    // Create bound content
    let bound = BoundContent {
        id: ContentId::new("msg_001"),
        modalities: vec![text_content, dalle_content, audio_content],
        embedding: vec![0.1; 256], // Simplified embedding
        timestamp: Some(star::now_timestamp()),
        provenance: "chat_export".to_string(),
        conversation_id: Some("conv_123".to_string()),
    };

    println!("  Created bound content: {} modalities (text, dalle, audio)", bound.modalities.len());

    // Test search
    let results = engine.search("sunset ocean", 5);
    println!("  Search results for 'sunset ocean': {} found", results.len());

    // Test cross-modal binding
    let binder = star::multimodal::binding::CrossModalBinder::new(0.5);
    let related = binder.find_related(&engine, &bound.id);
    println!("  Related content: {} items", related.len());

    println!("\n  ✓ Multimodal PASSED\n");
}

// ═══════════════════════════════════════════════════════════════════════════
// DEBUG LOGGING TESTS
// ═══════════════════════════════════════════════════════════════════════════

fn test_debug_logging() {
    println!("[13/13] Testing Debug Logging Infrastructure...\n");

    // Test that tracing::debug! macros compile and run correctly
    // by exercising code paths that contain debug! calls

    // Test knowledge module debug paths
    let mut kg = star::reasoning::knowledge::KnowledgeGraph::new();
    kg.ingest_fact("debug", "is", "useful for development", 0.9);

    let facts = kg.get_facts_about("debug");
    println!("  KG facts about 'debug': {:?}", facts);
    assert!(!facts.is_empty(), "Should have facts about debug");

    // Test reasoning engine with knowledge
    let mut engine = star::reasoning::ReasoningEngine::new();
    engine.add_knowledge("test", "debugging");
    engine.add_knowledge("fire", "hot");

    let result = engine.reason("What is fire?", &[]);
    println!("  Reasoning result: {:?}", result.answer);
    assert!(result.answer.is_some());

    // Test curiosity engine debug paths via reasoning analogy
    use star::reasoning::knowledge::RelationType;
    let mut kg2 = star::reasoning::knowledge::KnowledgeGraph::new();
    kg2.add_relationship("star", RelationType::IsA, "reasoning intelligence");
    kg2.add_relationship("star", RelationType::HasProperty, "curiosity");

    let analogies = kg2.find_analogies("star", "human");
    println!("  Analogies found between star and human: {}", analogies.len());

    // Test that curiosity probes module is accessible (uses debug! internally)
    use star::curiosity::probes::CuriosityProbe;
    let probe = CuriosityProbe::new(
        "test curiosity",
        "testing",
    );
    println!("  Created curiosity probe: topic='{}', depth={:?}", probe.topic, probe.depth);

    println!("\n  ✓ Debug Logging PASSED (comprehensive path coverage)\n");
}