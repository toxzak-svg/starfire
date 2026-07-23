fn microstate_for(
    intent: ShadowIntent,
    sensitivity: crate::semantic_response::SensitivityLevel,
) -> Result<ConversationalMicrostate, L1CShadowError> {
    use crate::semantic_response::SensitivityLevel;
    let personal = sensitivity >= SensitivityLevel::Personal;
    let state = match intent {
        ShadowIntent::Emotional | ShadowIntent::Reflection | ShadowIntent::Aspiration => {
            ConversationalMicrostate::new(
                5_800,
                if personal { 8_800 } else { 7_800 },
                5_800,
                5_800,
                4_500,
                7_000,
            )?
        }
        ShadowIntent::StoryPrompt | ShadowIntent::CuriosityCheck => {
            ConversationalMicrostate::new(6_400, 6_800, 7_600, 5_200, 8_000, 8_400)?
        }
        ShadowIntent::ResearchStatus
        | ShadowIntent::Capability
        | ShadowIntent::Teaching
        | ShadowIntent::Recall => {
            ConversationalMicrostate::new(8_400, 5_200, 6_800, 7_800, 2_200, 7_400)?
        }
        ShadowIntent::SelfCheck | ShadowIntent::Consciousness | ShadowIntent::Identity => {
            ConversationalMicrostate::new(7_600, 5_800, 6_200, 7_000, 3_000, 7_800)?
        }
    };
    Ok(state)
}

fn entropy_seed(bundle: &ShadowInputBundle, response: ResponseFingerprint) -> u64 {
    let mut material = Vec::new();
    material.extend_from_slice(bundle.event_id.as_bytes());
    material.extend_from_slice(&bundle.program.digest.0.to_le_bytes());
    material.extend_from_slice(&bundle.lexical_table.digest.0.to_le_bytes());
    material.extend_from_slice(&response.before_digest.to_le_bytes());
    domain_hash(SEED_DOMAIN, &material).max(1)
}

fn trace_store() -> &'static Mutex<BTreeMap<String, RecentLanguageTrace>> {
    TRACE_STORE.get_or_init(|| Mutex::new(BTreeMap::new()))
}

fn trace_snapshot(key: &str) -> Result<RecentLanguageTrace, L1CShadowError> {
    let guard = trace_store()
        .lock()
        .map_err(|_| L1CShadowError::Trace("ephemeral trace lock poisoned".to_owned()))?;
    Ok(guard.get(key).cloned().unwrap_or_default())
}

fn commit_trace(key: &str, trace: RecentLanguageTrace) -> Result<(), L1CShadowError> {
    trace.verify_integrity()?;
    let mut guard = trace_store()
        .lock()
        .map_err(|_| L1CShadowError::Trace("ephemeral trace lock poisoned".to_owned()))?;
    if !guard.contains_key(key) && guard.len() >= MAX_TRACE_KEYS {
        let oldest = guard.keys().next().cloned();
        if let Some(oldest) = oldest {
            guard.remove(&oldest);
        }
    }
    guard.insert(key.to_owned(), trace);
    Ok(())
}

fn append_record(record: &L1CShadowLedgerRecord) -> Result<(), L1CShadowError> {
    let path = ledger_path();
    append_record_to_path(record, &path)
}

pub fn append_record_to_path(
    record: &L1CShadowLedgerRecord,
    path: &Path,
) -> Result<(), L1CShadowError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| L1CShadowError::Ledger(error.to_string()))?;
    }
    let serialized = serde_json::to_string(record)
        .map_err(|error| L1CShadowError::Serialization(error.to_string()))?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| L1CShadowError::Ledger(error.to_string()))?;
    file.write_all(serialized.as_bytes())
        .and_then(|_| file.write_all(b"\n"))
        .map_err(|error| L1CShadowError::Ledger(error.to_string()))
}

fn ledger_path() -> PathBuf {
    std::env::var_os("STARFIRE_STLM_L1C_LEDGER_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::env::var_os("STARFIRE_DATA")
                .or_else(|| std::env::var_os("STARFIRE_HOME"))
                .map(PathBuf::from)
                .or_else(|| dirs::data_local_dir().map(|path| path.join("star")))
                .unwrap_or_else(|| PathBuf::from("."))
                .join("logs")
                .join(DEFAULT_LEDGER_FILENAME)
        })
}

fn response_matches_bytes(response: ResponseFingerprint, bytes: &[u8]) -> bool {
    response.before_len == u32::try_from(bytes.len()).unwrap_or(u32::MAX)
        && response.before_digest == domain_hash(OMEGA_F2_RESPONSE_DOMAIN, bytes)
}

fn comparison_digest(payload: &L1CComparisonPayload) -> Result<u64, L1CShadowError> {
    let bytes = serde_json::to_vec(payload)
        .map_err(|error| L1CShadowError::Serialization(error.to_string()))?;
    Ok(domain_hash(COMPARISON_DOMAIN, &bytes).max(1))
}

fn authority_matrix_digest() -> String {
    let bytes = serde_json::to_vec(&authority_boundary()).unwrap_or_default();
    format!("{:016x}", domain_hash(AUTHORITY_DOMAIN, &bytes))
}

fn text_fingerprint(text: &str) -> u64 {
    domain_hash(TEXT_DOMAIN, text.as_bytes()).max(1)
}

fn domain_hash(domain: &[u8], bytes: &[u8]) -> u64 {
    let mut digest = 0xcbf2_9ce4_8422_2325_u64;
    for byte in domain.iter().chain(bytes.iter()) {
        digest ^= u64::from(*byte);
        digest = digest.wrapping_mul(0x1000_0000_01b3);
    }
    digest
}

fn bounded_reason(reason: &str) -> String {
    reason.chars().take(160).collect()
}

fn bounded_len(len: usize) -> u16 {
    u16::try_from(len).unwrap_or(u16::MAX)
}
