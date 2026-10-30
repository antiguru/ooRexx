use std::collections::BTreeSet;

/// Whether `haystack` holds `needle`.
pub(super) fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

/// The group file `text` with each test of `group` that `left_out` names
/// renamed as [`skip`] renames it, and the `GROUP.TEST` names it renamed.
pub fn renamed<S: AsRef<str> + Ord>(
    text: &[u8],
    group: &str,
    left_out: &BTreeSet<S>,
) -> (Vec<u8>, BTreeSet<String>) {
    let mut skipped = BTreeSet::new();
    let lines: Vec<Vec<u8>> = text
        .split(|&byte| byte == b'\n')
        .map(|line| {
            let trimmed = line.trim_ascii_start();
            let Some(rest) = trimmed
                .get(..8)
                .filter(|head| head.eq_ignore_ascii_case(b"::method"))
                .map(|_| &trimmed[8..])
            else {
                return line.to_vec();
            };
            let rest = rest.trim_ascii_start();
            let quoted = rest.starts_with(b"'") || rest.starts_with(b"\"");
            let name_start = usize::from(quoted);
            let name: Vec<u8> = rest[name_start..]
                .iter()
                .copied()
                .take_while(|&byte| {
                    if quoted {
                        !matches!(byte, b'\'' | b'"')
                    } else {
                        byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'!' | b'?' | b'.')
                    }
                })
                .collect();
            let member = format!(
                "{group}.{}",
                String::from_utf8_lossy(&name).to_ascii_uppercase()
            );
            if !left_out.iter().any(|name| name.as_ref() == member) {
                return line.to_vec();
            }
            skipped.insert(member);
            let at = line.len() - rest.len() + name_start;
            [&line[..at], b"SKIPPED_".as_slice(), &line[at..]].concat()
        })
        .collect();
    (lines.join(&b'\n'), skipped)
}

