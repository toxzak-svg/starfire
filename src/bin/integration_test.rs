//! Starfire Integration Test — Exercises all modules end-to-end
//!
//! Run with: `cargo run --bin integration_test`
//!
//! This test:
//! 1. Initializes Quanot with text input
//! 2. Creates a WorldModel and updates it
//! 3. Runs Causal Discovery on temporal data
//! 4. Sets up Goals and tracks progress
//! 5. Uses Few-Shot Learning to form hypotheses
//! 6. Uses Curriculum to identify knowledge gaps
//! 7. Prints results

use std::path::PathBuf;

fn main() {
    println!("═══════════════════════════════════════════════════");
    println!("         STARFIRE INTEGRATION TEST");
    println!("═══════════════════════════════════════════════════\n");

    test_quanot();
    test_world_model();
    test_causal_discovery();
    test_goals();
    test_fewshot_learning();
    test_curriculum();
    test_multimodal();

    println!("\n═══════════════════════════════════════════════════");
    println!("         ALL TESTS PASSED ✓");
    println!("═══════════════════════════════════════════════════");
}

fn test_quanot() {
    println!("[1/7] Testing Quanot (Rust Reservoir Computing)...\n");

    let mut quanot = star::quanot::Quanot::new(128, 500);

    // Process a few inputs to build up reservoir state history
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
    }

    let avg_consciousness = consciousness_sum / inputs.len() as f64;
    let avg_novelty = novelty_sum / inputs.len() as f64;

    println!("  Average consciousness: {:.4}", avg_consciousness);
    println!("  Average novelty:       {:.4}", avg_novelty);

    assert!(avg_consciousness >= 0.0 && avg_consciousness <= 1.0, "Consciousness out of range");
    assert!(avg_novelty >= 0.0 && avg_novelty <= 1.0, "Novelty out of range");

    println!("\n  ✓ Quanot PASSED\n");
}

fn test_world_model() {
    println!("[2/7] Testing WorldModel (Entity/Relation Management)...\n");

    let mut world = star::world_model::WorldModel::new();

    // Add entities
    let fire = star::world_model::Entity::new(
        star::world_model::EntityId::new("fire"),
        "Fire".to_string(),
    )
    .with_property("temperature", star::world_model::PropertyValue::from(1000.0_f64))
    .with_property("state", star::world_model::PropertyValue::from("burning"));

    let heat = star::world_model::Entity::new(
        star::world_model::EntityId::new("heat"),
        "Heat".to_string(),
    );

    let water = star::world_model::Entity::new(
        star::world_model::EntityId::new("water"),
        "Water".to_string(),
    )
    .with_property("state", star::world_model::PropertyValue::from("liquid"));

    world.upsert_entity(fire);
    world.upsert_entity(heat);
    world.upsert_entity(water);

    // Add relations
    world.add_relation(
        star::world_model::EntityId::new("fire"),
        star::world_model::EntityId::new("heat"),
        star::world_model::RelationType::CausallyRelated,
    );

    // Update with Quanot perception
    let perception = star::world_model::perception::QuanotPerception {
        reservoir_state: vec![0.5; 100],
        consciousness_proxy: 0.75,
        novelty: 0.4,
        creativity_scores: star::world_model::perception::CreativityOutput {
            creative_state: 0.6,
            divergence_metric: 0.3,
            diversity_index: 0.5,
            originality_score: 0.4,
            oscillation_phase: 0.0,
        },
    };

    world.update_from_perception(perception);

    // Query
    let facts = world.query("fire");
    println!("  Facts about 'fire':");
    for fact in &facts {
        println!("    {}", fact);
    }

    // Test prediction
    if let Some(outcome) = world.predict_next_state(&star::world_model::EntityId::new("fire")) {
        println!("  Prediction: {} actions predicted", outcome.predictions.len());
    }

    let stats = world.stats();
    println!("  WorldModel stats: {} entities, {} relations",
        stats.entity_count, stats.total_relations);

    assert!(world.entity_count() >= 3, "Should have at least 3 entities");

    println!("\n  ✓ WorldModel PASSED\n");
}

fn test_causal_discovery() {
    println!("[3/7] Testing Causal Discovery (Temporal → Causal)...\n");

    let mut engine = star::causal::CausalEngine::new();

    // Add known causal edges
    let fire_heat = engine.add_edge("fire", "heat", 0.9, Some(1));
    let water_electricity = engine.add_edge("water", "electricity", 0.7, Some(2));
    let sun_photosynthesis = engine.add_edge("sun", "photosynthesis", 0.85, Some(60));

    // Update with evidence
    engine.update_edge(&fire_heat, true);
    engine.update_edge(&fire_heat, true);
    engine.update_edge(&fire_heat, false); // One contradiction

    engine.update_edge(&water_electricity, true);
    engine.update_edge(&water_electricity, true);

    // Get effects of fire
    let fire_effects = engine.get_effects_of("fire");
    println!("  Fire causes: {:?}", fire_effects.iter().map(|e| e.effect.as_str()).collect::<Vec<_>>());

    // Get top hypotheses
    let top = engine.top_hypotheses(3);
    println!("  Top causal hypotheses:");
    for h in &top {
        println!("    {} → {} (conf: {:.2})",
            h.candidate.cause, h.candidate.effect, h.candidate.confidence);
    }

    // Test causal graph
    let mut graph = star::causal::graph::CausalGraph::new();
    for edge in engine.edges().values() {
        graph.add_edge(edge.clone());
    }

    println!("  Causal graph: {} nodes, {} edges",
        graph.node_count(), graph.edge_count());

    let fire_heat_exists = graph.path_exists("fire", "heat");
    println!("  Path fire → heat exists: {}", fire_heat_exists);

    assert!(!engine.is_empty(), "CausalEngine should not be empty");
    assert!(graph.node_count() >= 3, "Should have at least 3 nodes");

    println!("\n  ✓ Causal Discovery PASSED\n");
}

fn test_goals() {
    println!("[4/7] Testing Goals (Hierarchical Goal Memory)...\n");

    let mut goal_engine = star::goals::GoalEngine::new();

    // Create root goal
    let build_ai = goal_engine.create_goal("Build an AGI system", None);
    println!("  Created goal: Build an AGI system (id: {:?})", build_ai);

    // Set priority
    if let Some(goal) = goal_engine.get_mut(&build_ai) {
        goal.with_priority(0.95);
        goal.with_deadline(star::now_timestamp() + 86400 * 30); // 30 days
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

    let stats = goal_engine;
    println!("\n  Total goals: {}", stats.len());

    assert!(stats.len() >= 6, "Should have at least 6 goals");

    println!("\n  ✓ Goals PASSED\n");
}

fn test_fewshot_learning() {
    println!("[5/7] Testing Few-Shot Learning (Rapid Hypothesis)...\n");

    let mut learner = star::learning::FewShotLearner::new();

    // Add examples from a domain
    learner.add_example(star::learning::Example::new(
        "Fire produces heat",
        "heat + flame + burn",
        "physics",
    ));
    learner.add_example(star::learning::Example::new(
        "Fire is dangerous",
        "danger + burn + warning",
        "physics",
    ));
    learner.add_example(star::learning::Example::new(
        "The sun produces light",
        "light + warmth + energy",
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
    learner.add_example(star::learning::Example::new(
        "Water flows downhill",
        "flow + gravity + downhill",
        "geography",
    ));
    learner.add_example(star::learning::Example::new(
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

    assert!(learner.example_count() >= 5, "Should have at least 5 examples");

    println!("\n  ✓ Few-Shot Learning PASSED\n");
}

fn test_curriculum() {
    println!("[6/7] Testing Curriculum (Self-Directed Learning)...\n");

    let mut curriculum = star::curriculum::CurriculumEngine::new();

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

    // Manually add a high-urgency gap
    let high_urgency = star::curriculum::KnowledgeGap::new(
        "Quantum consciousness",
        star::curriculum::GapType::Incomplete,
    ).with_urgency(0.9);
    curriculum.add_gap(high_urgency);

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

fn test_multimodal() {
    println!("[7/7] Testing Multimodal (Text/Image/Audio Binding)...\n");

    let mut engine = star::multimodal::MultimodalEngine::new(256);

    // Add text content
    let text_content = star::multimodal::Modality::Text {
        content: "I generated an image of a sunset over the ocean".to_string(),
        role: Some("user".to_string()),
        timestamp: Some(star::now_timestamp()),
    };

    let dalle_content = star::multimodal::Modality::Dalle {
        prompt: "A beautiful sunset over the ocean with vibrant colors".to_string(),
        image_path: "/exports/dalle/sunset_001.png".to_string(),
        generation_id: "dalle_abc123".to_string(),
        generation_time: Some(star::now_timestamp()),
    };

    // Create bound content
    let bound = star::multimodal::BoundContent {
        id: star::multimodal::ContentId::new("msg_001"),
        modalities: vec![text_content, dalle_content],
        embedding: engine.text_encoder.encode("sunset ocean colors"),
        timestamp: Some(star::now_timestamp()),
        provenance: "chatgpt_export".to_string(),
        conversation_id: Some("conv_123".to_string()),
    };

    println!("  Created bound content: {} modalities", bound.modalities.len());

    // Test search
    let results = engine.search("sunset ocean", 5);
    println!("  Search results for 'sunset ocean': {} found", results.len());

    // Test cross-modal binding
    let binder = star::multimodal::binding::CrossModalBinder::new(0.5);
    let related = binder.find_related(&engine, &bound.id);
    println!("  Related content: {} items", related.len());

    println!("\n  ✓ Multimodal PASSED\n");
}
