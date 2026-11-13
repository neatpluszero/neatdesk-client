//! Developer-facing trace of a session's connection attempt: the stages a
//! connect goes through - rendezvous, punch, relay, failure - with their timing,
//! kept per peer so the log and a details view can show why a connect took the
//! path it did, or where it stalled. Pure diagnostics: nothing here feeds back
//! into the connection.

use std::{collections::HashMap, sync::Mutex, time::Instant};

/// Cap per peer so a stuck, retrying attempt can't grow the buffer without bound.
const MAX_ENTRIES: usize = 64;

struct Trace {
    start: Instant,
    entries: Vec<Entry>,
}

struct Entry {
    ms: u64,
    stage: String,
    detail: String,
}

lazy_static::lazy_static! {
    static ref TRACES: Mutex<HashMap<String, Trace>> = Default::default();
}

/// Start (or restart) the trace for `peer`, dropping any previous attempt.
pub fn begin(peer: &str) {
    if let Ok(mut traces) = TRACES.lock() {
        traces.insert(
            peer.to_owned(),
            Trace {
                start: Instant::now(),
                entries: Vec::new(),
            },
        );
    }
}

/// Record one stage of the attempt. Also emitted to the log, so the trace is
/// usable from the log file with no UI. A `peer` never `begin`-ed starts a trace
/// here.
pub fn push(peer: &str, stage: &str, detail: &str) {
    log::info!("[conn {peer}] {stage}: {detail}");
    if let Ok(mut traces) = TRACES.lock() {
        let trace = traces.entry(peer.to_owned()).or_insert_with(|| Trace {
            start: Instant::now(),
            entries: Vec::new(),
        });
        let ms = trace.start.elapsed().as_millis() as u64;
        trace.entries.push(Entry {
            ms,
            stage: stage.to_owned(),
            detail: detail.to_owned(),
        });
        let len = trace.entries.len();
        if len > MAX_ENTRIES {
            trace.entries.drain(0..len - MAX_ENTRIES);
        }
    }
}

/// The trace for `peer` as a JSON array of `{ms, stage, detail}`, oldest first;
/// an empty array when there is none.
pub fn get_json(peer: &str) -> String {
    if let Ok(traces) = TRACES.lock() {
        if let Some(trace) = traces.get(peer) {
            let arr: Vec<serde_json::Value> = trace
                .entries
                .iter()
                .map(|e| {
                    serde_json::json!({
                        "ms": e.ms,
                        "stage": e.stage,
                        "detail": e.detail,
                    })
                })
                .collect();
            return serde_json::Value::Array(arr).to_string();
        }
    }
    "[]".to_owned()
}
