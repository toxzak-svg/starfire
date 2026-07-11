use std::error::Error;

use star::charge::OntologyObservation;

mod canonical_fixture {
    include!("h5_task_profiled_nonmemory_diagnostic.rs");

    pub(super) fn task_profiled_fixed_windows(
    ) -> Result<Vec<Vec<OntologyObservation>>, Box<dyn Error>> {
        let state = ProbeState::new()?;
        let recorder = CycleObservationRecorder::new(RelativeImprovementJudge);
        let fixed_config = FixedResidualProjectionConfig::default();
        let mut next_id = 1u64;
        let mut windows = Vec::with_capacity(FAMILIES.len());

        for family in FAMILIES {
            let mut window = Vec::with_capacity(OBSERVATIONS_PER_WINDOW);
            for class in EventClass::all() {
                for repeat in 0..REPEATS_PER_CLASS {
                    let task = surface_task(family, class, repeat);
                    let mut charge = emit_real_charge(&task)?;
                    charge.kind = ChargeKind::Custom("unresolved".into());
                    charge.id = next_id;
                    next_id += 1;
                    let fixed = fixed_residual_feature_charge(&charge, fixed_config);
                    let attempts = judged_component_attempts(
                        &fixed,
                        &task,
                        &state,
                        VerifierProfile::TaskProfiled,
                        task.task_class,
                    )?;
                    window.push(recorder.record(fixed, &attempts)?);
                }
            }
            windows.push(window);
        }

        Ok(windows)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let windows = canonical_fixture::task_profiled_fixed_windows()?;
    run_diagnostic(windows)
}
