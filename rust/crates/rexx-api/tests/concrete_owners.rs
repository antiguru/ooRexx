//! Concrete refusal owners at FFI boundaries, without calling aborting extern functions.

const SOURCE: &str = include_str!("../src/ffi.rs");
const SITES: [&str; 2] = [
    "RexxInstanceInterface.AttachThread",
    "RexxInstanceInterface.AddCommandEnvironment from another thread",
];

fn has_embedding_owner(source: &str, site: &str) -> bool {
    let anchor = format!("\"{site}\",");
    let mut occurrences = source.match_indices(&anchor);
    let Some((at, _)) = occurrences.next() else { return false };
    occurrences.next().is_none()
        && source[at + anchor.len()..].trim_start().starts_with("Owner::Phase10(Deferral::Embedding)")
}

#[test]
fn concrete_cross_thread_refusals_belong_to_embedding() {
    for site in SITES {
        assert!(has_embedding_owner(SOURCE, site), "{site}");
    }
}

#[test]
fn each_wrong_concrete_owner_is_rejected_independently() {
    for site in SITES {
        let anchor = format!("\"{site}\",\n            Owner::Phase10(Deferral::Embedding)");
        assert_eq!(SOURCE.matches(&anchor).count(), 1);
        let changed = SOURCE.replacen(&anchor, &anchor.replace("Deferral::Embedding", "Deferral::Extension"), 1);
        assert!(!has_embedding_owner(&changed, site), "{site}");
        let other = SITES.into_iter().find(|other| *other != site).unwrap();
        assert!(has_embedding_owner(&changed, other), "{other}");
    }
}
