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
    
    // Technology
    inject_technology_knowledge(store)?;
    
    // Earth and nature
    inject_earth_knowledge(store)?;
    
    // Mathematics
    inject_math_knowledge(store)?;
    
    // Society
    inject_society_knowledge(store)?;
    
    // Communication
    inject_communication_knowledge(store)?;
    
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
        ("Logic can prove things", 0.85),
        ("Science tests hypotheses with experiments", 0.85),
        ("Theories can be revised with new evidence", 0.85),
        ("Facts are different from opinions", 0.9),
        ("Assumptions should be examined", 0.85),
        ("Multiple explanations can fit the same facts", 0.8),
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

fn inject_technology_knowledge(store: &Store) -> Result<()> {
    let facts = [
        // Computers
        ("Computers process information with electricity", 0.85),
        ("Software is instructions for computers", 0.85),
        ("Data is stored in memory", 0.85),
        ("Networks connect computers", 0.85),
        ("The internet connects networks worldwide", 0.9),
        ("Websites are accessed via browsers", 0.85),
        ("Search engines find information online", 0.85),
        ("Code is written in programming languages", 0.85),
        ("Algorithms are step-by-step procedures", 0.85),
        ("Databases store structured information", 0.8),
        ("Encryption protects information", 0.85),
        ("Passwords protect accounts", 0.9),
        
        // Transportation
        ("Cars move using engines", 0.85),
        ("Planes fly using wings and engines", 0.85),
        ("Boats float on water", 0.85),
        ("Trains run on tracks", 0.8),
        ("Bicycles use human power", 0.85),
        
        // Tools
        ("Hammers drive nails", 0.85),
        ("Screws hold things together tighter than nails", 0.8),
        ("Knives cut", 0.85),
        ("Engines convert fuel to motion", 0.85),
        ("Generators create electricity", 0.85),
        
        // Medicine
        ("Vaccines prevent disease", 0.9),
        ("Antibiotics kill bacteria", 0.85),
        ("Viruses are not killed by antibiotics", 0.85),
        ("Surgery can repair injuries", 0.85),
        ("Medicine can relieve symptoms", 0.85),
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

fn inject_earth_knowledge(store: &Store) -> Result<()> {
    let facts = [
        // Earth structure
        ("Earth is a planet", 0.95),
        ("Earth orbits the sun", 0.95),
        ("Earth rotates on its axis", 0.9),
        ("Earth has a molten core", 0.85),
        ("Earth has a solid crust", 0.85),
        ("Earth is about 4.5 billion years old", 0.8),
        ("The moon orbits Earth", 0.95),
        
        // Atmosphere
        ("Air is a mixture of gases", 0.9),
        ("Oxygen is about 21% of air", 0.85),
        ("Nitrogen is about 78% of air", 0.85),
        ("The atmosphere protects Earth", 0.85),
        ("Weather happens in the atmosphere", 0.9),
        ("Clouds are water vapor", 0.9),
        ("Rain is water falling from clouds", 0.9),
        ("Snow is frozen rain", 0.9),
        ("Lightning is electrical discharge", 0.85),
        ("Thunder is the sound of lightning", 0.85),
        
        // Water
        ("Water covers most of Earth", 0.95),
        ("Oceans are large bodies of salt water", 0.9),
        ("Rivers flow to the sea", 0.9),
        ("Lakes are bodies of water surrounded by land", 0.9),
        ("Freshwater is water without salt", 0.9),
        ("Glaciers are frozen water", 0.85),
        ("Water evaporates from oceans and lakes", 0.9),
        
        // Land
        ("Mountains are high landforms", 0.9),
        ("Valleys are low areas between mountains", 0.85),
        ("Plains are flat areas of land", 0.85),
        ("Deserts are dry regions", 0.9),
        ("Forests have many trees", 0.9),
        ("Soil contains nutrients for plants", 0.85),
        ("Rocks are made of minerals", 0.85),
        ("Earthquakes are shaking of the ground", 0.85),
        ("Volcanoes erupt molten rock", 0.85),
        
        // Solar system
        ("The sun is a star", 0.95),
        ("The solar system has eight planets", 0.9),
        ("Mercury is the closest planet to the sun", 0.85),
        ("Venus is the hottest planet", 0.85),
        ("Mars is the red planet", 0.85),
        ("Jupiter is the largest planet", 0.85),
        ("Saturn has prominent rings", 0.85),
        ("Uranus and Neptune are ice giants", 0.8),
        ("Pluto is a dwarf planet", 0.8),
        ("Asteroids are rocky objects in space", 0.85),
        ("Comets are icy objects that develop tails near the sun", 0.8),
        
        // Universe
        ("The universe is very old", 0.85),
        ("The universe is very large", 0.9),
        ("Galaxies contain billions of stars", 0.85),
        ("The Milky Way is our galaxy", 0.9),
        ("Light years measure distance in space", 0.85),
        ("Stars produce light and heat", 0.9),
        ("Black holes have extremely strong gravity", 0.8),
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

fn inject_math_knowledge(store: &Store) -> Result<()> {
    let facts = [
        ("Numbers represent quantities", 0.9),
        ("Addition combines quantities", 0.9),
        ("Subtraction removes quantities", 0.9),
        ("Multiplication is repeated addition", 0.9),
        ("Division splits quantities", 0.9),
        ("Zero represents nothing", 0.85),
        ("Negative numbers are less than zero", 0.85),
        ("Fractions represent parts of wholes", 0.85),
        ("Decimals are another way to write fractions", 0.85),
        ("Percent means per hundred", 0.85),
        ("Geometry studies shapes", 0.85),
        ("Circles have no corners", 0.85),
        ("Triangles have three sides", 0.9),
        ("Squares have four equal sides", 0.9),
        ("Pi relates circle circumference to diameter", 0.8),
        ("Probability measures likelihood", 0.85),
        ("Statistics analyzes data", 0.85),
        ("Patterns exist in mathematics", 0.85),
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

fn inject_society_knowledge(store: &Store) -> Result<()> {
    let facts = [
        ("Families are social groups", 0.9),
        ("Communities are groups of people living together", 0.9),
        ("Governments make rules for societies", 0.9),
        ("Laws are rules enforced by authorities", 0.9),
        ("Money enables trade", 0.9),
        ("Work creates value", 0.85),
        ("Education teaches skills and knowledge", 0.9),
        ("Books contain written knowledge", 0.9),
        ("Libraries store books and information", 0.85),
        ("Art expresses ideas and emotions", 0.85),
        ("Music uses sound and rhythm", 0.85),
        ("Stories convey experiences and ideas", 0.9),
        ("Science advances through research", 0.85),
        ("History records past events", 0.9),
        ("Cultures have different customs", 0.85),
        ("Languages vary between cultures", 0.9),
        ("Trade exchanges goods and services", 0.9),
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

fn inject_communication_knowledge(store: &Store) -> Result<()> {
    let facts = [
        ("Language conveys meaning", 0.95),
        ("Words represent concepts", 0.9),
        ("Sentences express complete thoughts", 0.9),
        ("Questions ask for information", 0.95),
        ("Answers respond to questions", 0.95),
        ("Stories have beginnings middles and ends", 0.85),
        ("Explanations clarify understanding", 0.9),
        ("Descriptions convey details", 0.9),
        ("Arguments present reasons", 0.85),
        ("Listening is as important as speaking", 0.85),
        ("Misunderstanding causes confusion", 0.9),
        ("Clarity aids understanding", 0.9),
        ("Examples illustrate concepts", 0.85),
        ("Metaphors compare different things", 0.8),
        ("Writing preserves thoughts over time", 0.9),
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
