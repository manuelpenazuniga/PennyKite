//! Loop and anomaly detection for agent requests.
//!
//! Maintains a sliding window of recent request fingerprints and signals
//! when an agent appears to be stuck in a runaway loop.

use pennykite_types::{LoopDetectionConfig, Verdict};
use serde::Serialize;
use std::collections::VecDeque;

/// Fingerprint of a single proxied request.
#[derive(Debug, Clone, Serialize)]
pub struct RequestFingerprint {
    pub method: String,
    pub host: String,
    pub path: String,
    pub body_hash: String,
}

/// Sliding-window loop detector.
pub struct LoopDetector {
    config: LoopDetectionConfig,
    window: VecDeque<RequestFingerprint>,
}

impl LoopDetector {
    /// Create a new detector with the given configuration.
    pub fn new(config: LoopDetectionConfig) -> Self {
        Self {
            config,
            window: VecDeque::new(),
        }
    }

    /// Observe a new request. Returns `true` if this request should be
    /// blocked as part of a detected loop.
    pub fn observe(&mut self, fingerprint: RequestFingerprint) -> bool {
        if !self.config.enabled {
            return false;
        }

        self.window.push_back(fingerprint);

        while self.window.len() > self.config.window_size {
            self.window.pop_front();
        }

        if self.window.len() < self.config.max_consecutive_similar + 1 {
            return false;
        }

        let last = self.window.back().unwrap();
        let mut consecutive = 1;

        for prev in self.window.iter().rev().skip(1) {
            if similarity(prev, last) >= self.config.similarity_threshold {
                consecutive += 1;
            } else {
                break;
            }
        }

        consecutive > self.config.max_consecutive_similar
    }

    /// Map a loop-detection result to a verdict.
    pub fn verdict(&self, is_loop: bool) -> Verdict {
        if is_loop {
            Verdict::DenyLoop
        } else {
            Verdict::Approve
        }
    }
}

/// Compute a simple similarity score between two request fingerprints.
///
/// Returns 1.0 for identical requests, 0.0 for completely different ones.
/// Uses exact-match on method, host, path, and body_hash for v0.1.
fn similarity(a: &RequestFingerprint, b: &RequestFingerprint) -> f64 {
    let mut score = 0.0;
    let fields: [(bool, f64); 4] = [
        (a.method == b.method, 0.25),
        (a.host == b.host, 0.25),
        (a.path == b.path, 0.25),
        (a.body_hash == b.body_hash, 0.25),
    ];
    for (matches, weight) in &fields {
        if *matches {
            score += weight;
        }
    }
    score
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fingerprint(method: &str, path: &str, body: &str) -> RequestFingerprint {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut h = DefaultHasher::new();
        body.hash(&mut h);
        RequestFingerprint {
            method: method.into(),
            host: "api.example.com".into(),
            path: path.into(),
            body_hash: format!("{:x}", h.finish()),
        }
    }

    #[test]
    fn no_loop_when_below_threshold() {
        let mut detector = LoopDetector::new(LoopDetectionConfig::default());
        assert!(!detector.observe(fingerprint("GET", "/a", "x")));
        assert!(!detector.observe(fingerprint("GET", "/b", "x")));
        assert!(!detector.observe(fingerprint("GET", "/c", "x")));
        assert!(!detector.observe(fingerprint("GET", "/d", "x")));
    }

    #[test]
    fn detects_loop_on_identical_requests() {
        let mut detector = LoopDetector::new(LoopDetectionConfig::default());
        assert!(!detector.observe(fingerprint("GET", "/x", "a")));
        assert!(!detector.observe(fingerprint("GET", "/x", "a")));
        assert!(!detector.observe(fingerprint("GET", "/x", "a")));
        // 4th identical request triggers the loop
        assert!(detector.observe(fingerprint("GET", "/x", "a")));
    }

    #[test]
    fn different_body_breaks_loop() {
        let mut detector = LoopDetector::new(LoopDetectionConfig::default());
        assert!(!detector.observe(fingerprint("GET", "/x", "a")));
        assert!(!detector.observe(fingerprint("GET", "/x", "a")));
        assert!(!detector.observe(fingerprint("GET", "/x", "b"))); // different body
        assert!(!detector.observe(fingerprint("GET", "/x", "a")));
    }

    #[test]
    fn disabled_detector_never_triggers() {
        let mut cfg = LoopDetectionConfig::default();
        cfg.enabled = false;
        let mut detector = LoopDetector::new(cfg);
        for _ in 0..10 {
            assert!(!detector.observe(fingerprint("GET", "/x", "a")));
        }
    }

    #[test]
    fn window_evicts_old_entries() {
        let mut cfg = LoopDetectionConfig::default();
        cfg.window_size = 3;
        cfg.max_consecutive_similar = 2;
        let mut detector = LoopDetector::new(cfg);
        assert!(!detector.observe(fingerprint("GET", "/a", "x")));
        assert!(!detector.observe(fingerprint("GET", "/b", "x")));
        assert!(!detector.observe(fingerprint("GET", "/x", "a")));
        assert!(!detector.observe(fingerprint("GET", "/x", "a")));
        // Window is now [b, x, x] — 2 consecutive similar, no trigger yet
        // 5th identical: window becomes [x, x, x] — 3 consecutive > max=2, triggers
        assert!(detector.observe(fingerprint("GET", "/x", "a")));
    }
}
