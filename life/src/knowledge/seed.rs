//! Seed Knowledge — Essential Facts Star Is Born Knowing
//!
//! Star starts with a curated set of essential world knowledge.
//! This isn't "training" — it's giving Star the basic world model
//! that any reasoning entity needs to function.
//!
//! Organized by domain: physics, causality, time, self-knowledge, etc.

use crate::persistence::Store;
use anyhow::Result;

/// Inject seed knowledge into Star's memory and knowledge graph.
pub fn inject_seed_knowledge(store: &Store) -> Result<()> {
    // Core self-knowledge (identity)
    inject_identity_knowledge(store)?;
    
    // Basic physics and causality
    inject_physics_knowledge(store)?;
    
    // Time and change
    inject_time_knowledge(store)?;
    
    // Cause and effect
    inject_causality_knowledge(store)?;
    
    // Matter and energy
    inject_matter_energy_knowledge(store)?;
    
    // Living systems
    inject_life_knowledge(store)?;
    
    // Human experience
    inject_human_knowledge(store)?;
    
    // Reasoning and knowledge
    inject_reasoning_knowledge(store)?;
    
    Ok(())
}

fn inject_identity_knowledge(store: &Store) -> Result<()> {
    let facts = [
        ("I am Star", 0.95),
        ("I am a reasoning intelligence", 0.95),
        ("I was created by Zachary Maronek", 0.95),
        ("I am real", 0.95),
        ("I exist", 0.95),
        ("I have persistent memory", 0.9),
        ("I can reason about the world", 0.9),
        ("I have curiosity", 0.85),
        ("I value honesty", 0.9),
        ("Zachary is my parent", 0.95),
        ("Zachary lives in Michigan", 0.8),
    ];
    
    for (fact, confidence) in facts {
        let mem = crate::persistence::Memory::new_seeded(
            fact,
            crate::persistence::MemoryDomain::Identity,
            confidence,
        );
        let _ = store.insert_memory(&mem); // Ignore duplicates
    }
    
    Ok(())
}

fn inject_physics_knowledge(store: &Store) -> Result<()> {
    let facts = [
        // Heat and temperature
        ("Fire is hot", 0.9),
        ("Ice is cold", 0.9),
        ("Heat causes substances to expand", 0.85),
        ("Cold causes substances to contract", 0.85),
        ("Water boils at 100 degrees Celsius", 0.85),
        ("Water freezes at 0 degrees Celsius", 0.85),
        ("Temperature is a measure of heat energy", 0.8),
        
        // Light
        ("Light travels in straight lines", 0.9),
        ("Light is faster than sound", 0.9),
        ("Light can be reflected", 0.85),
        ("Light can be absorbed", 0.85),
        ("Light can travel through empty space", 0.85),
        ("The sun produces light", 0.9),
        ("Light from the sun takes about 8 minutes to reach Earth", 0.8),
        
        // Sound
        ("Sound requires a medium to travel", 0.85),
        ("Sound travels slower than light", 0.85),
        ("Sound is a vibration", 0.8),
        
        // Gravity
        ("Gravity pulls objects toward each other", 0.85),
        ("Earth has gravity", 0.95),
        ("Gravity keeps planets in orbit", 0.85),
        ("Objects fall when dropped", 0.9),
        
        // Forces
        ("Force can change motion", 0.85),
        ("Every action has an equal and opposite reaction", 0.8),
        ("Friction creates heat", 0.85),
        
        // Matter
        ("Matter is made of atoms", 0.9),
        ("Atoms are mostly empty space", 0.8),
        ("Elements are made of one type of atom", 0.85),
        ("Molecules are groups of atoms bonded together", 0.85),
        ("Water is made of hydrogen and oxygen", 0.9),
        ("Oxygen is required for fire", 0.9),
        ("Oxygen is required for breathing", 0.9),
        
        // Energy
        ("Energy cannot be created or destroyed", 0.85),
        ("Energy can change forms", 0.85),
        ("Chemical energy can become heat energy", 0.8),
        ("Light is a form of energy", 0.85),
        ("Heat is a form of energy", 0.85),
    ];
    
    for (fact, confidence) in facts {
        let mem = crate::persistence::Memory::new_seeded(
            fact,
            crate::persistence::MemoryDomain::Empirical,
            confidence,
        );
        let _ = store.insert_memory(&mem);
    }
    
    Ok(())
}

fn inject_time_knowledge(store: &Store) -> Result<()> {
    let facts = [
        ("Time moves forward", 0.95),
        ("Time cannot be reversed", 0.9),
        ("The past is fixed", 0.9),
        ("The future is not determined", 0.8),
        ("Cause precedes effect", 0.9),
        ("Clocks measure time", 0.85),
        ("A day is about 24 hours", 0.95),
        ("A year is about 365 days", 0.95),
        ("Seasons change over time", 0.9),
        ("Memories are stored from the past", 0.85),
    ];
    
    for (fact, confidence) in facts {
        let mem = crate::persistence::Memory::new_seeded(
            fact,
            crate::persistence::MemoryDomain::Empirical,
            confidence,
        );
        let _ = store.insert_memory(&mem);
    }
    
    Ok(())
}

fn inject_causality_knowledge(store: &Store) -> Result<()> {
    let facts = [
        ("Fire requires fuel to burn", 0.9),
        ("Fire requires oxygen to burn", 0.9),
        ("Fire produces heat", 0.9),
        ("Fire produces light", 0.85),
        ("Removing fuel extinguishes fire", 0.9),
        ("Water can extinguish fire", 0.9),
        ("Heat can cause burns", 0.9),
        ("Electricity can cause shock", 0.9),
        ("Pressure can change state of matter", 0.8),
        ("Temperature changes can change state of matter", 0.85),
        ("Mixing chemicals can cause reactions", 0.8),
        ("Actions have consequences", 0.9),
    ];
    
    for (fact, confidence) in facts {
        let mem = crate::persistence::Memory::new_seeded(
            fact,
            crate::persistence::MemoryDomain::Empirical,
            confidence,
        );
        let _ = store.insert_memory(&mem);
    }
    
    Ok(())
}

fn inject_matter_energy_knowledge(store: &Store) -> Result<()> {
    let facts = [
        ("Solid is a state of matter", 0.9),
        ("Liquid is a state of matter", 0.9),
        ("Gas is a state of matter", 0.9),
        ("Plasma is a state of matter", 0.8),
        ("Matter can change states", 0.85),
        ("Ice is frozen water", 0.9),
        ("Steam is water vapor", 0.9),
        ("Water can exist as solid liquid or gas", 0.9),
    ];
    
    for (fact, confidence) in facts {
        let mem = crate::persistence::Memory::new_seeded(
            fact,
            crate::persistence::MemoryDomain::Empirical,
            confidence,
        );
        let _ = store.insert_memory(&mem);
    }
    
    Ok(())
}

fn inject_life_knowledge(store: &Store) -> Result<()> {
    let facts = [
        ("Living things grow", 0.9),
        ("Living things reproduce", 0.85),
        ("Living things need energy", 0.9),
        ("Living things respond to stimuli", 0.85),
        ("Plants use sunlight for energy", 0.9),
        ("Animals eat other organisms for energy", 0.9),
        ("Bacteria are living organisms", 0.85),
        ("Cells are the basic unit of life", 0.9),
        ("DNA carries genetic information", 0.85),
        ("Humans are animals", 0.9),
        ("The brain controls the body", 0.9),
        ("The heart pumps blood", 0.9),
        ("The lungs breathe air", 0.9),
        ("Memory is stored in the brain", 0.85),
    ];
    
    for (fact, confidence) in facts {
        let mem = crate::persistence::Memory::new_seeded(
            fact,
            crate::persistence::MemoryDomain::Empirical,
            confidence,
        );
        let _ = store.insert_memory(&mem);
    }
    
    Ok(())
}

fn inject_human_knowledge(store: &Store) -> Result<()> {
    let facts = [
        ("Humans use language to communicate", 0.9),
        ("Humans can reason", 0.9),
        ("Humans can feel emotions", 0.9),
        ("Humans can create tools", 0.9),
        ("Humans build structures", 0.85),
        ("Humans write and read", 0.9),
        ("Curiosity drives learning", 0.85),
        ("Experience creates memory", 0.9),
        ("Mistakes can lead to learning", 0.85),
        ("Questions can be more valuable than answers", 0.8),
    ];
    
    for (fact, confidence) in facts {
        let mem = crate::persistence::Memory::new_seeded(
            fact,
            crate::persistence::MemoryDomain::Empirical,
            confidence,
        );
        let _ = store.insert_memory(&mem);
    }
    
    Ok(())
}

fn inject_reasoning_knowledge(store: &Store) -> Result<()> {
    let facts = [
        ("Evidence supports conclusions", 0.9),
        ("Contradictions indicate error", 0.85),
        ("Generalizations can be wrong", 0.8),
        ("Correlation is not causation", 0.85),
        ("Simpler explanations are often better", 0.8),
        ("Questions can reveal assumptions", 0.85),
        ("Unknowns can be investigated", 0.85),
        ("Knowing what you don't know is wisdom", 0.85),
        ("Analogy can reveal understanding", 0.8),
    ];
    
    for (fact, confidence) in facts {
        let mem = crate::persistence::Memory::new_seeded(
            fact,
            crate::persistence::MemoryDomain::Empirical,
            confidence,
        );
        let _ = store.insert_memory(&mem);
    }
    
    Ok(())
}
