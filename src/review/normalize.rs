//! Paragraph normalization. Lines of a paragraph join into one; runs of
//! whitespace collapse; blank lines, headings and list items start a new
//! paragraph. Both sides are normalized before comparison and the
//! normalized text is what gets shown.

fn starts_paragraph(line: &str) -> bool {
    let t = line.trim_start();
    t.starts_with("- ")
        || t.starts_with("* ")
        || t.starts_with("+ ")
        || t.starts_with('#')
        || t.starts_with("> ")
        || t.starts_with("| ")
        || t.starts_with("```")
        || t.split_once(". ")
            .map(|(n, _)| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()))
            .unwrap_or(false)
}

fn collapse(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn paragraphs(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut current: Vec<&str> = Vec::new();
    let flush = |current: &mut Vec<&str>, out: &mut Vec<String>| {
        if !current.is_empty() {
            out.push(collapse(&current.join(" ")));
            current.clear();
        }
    };
    for line in text.lines() {
        if line.trim().is_empty() {
            flush(&mut current, &mut out);
        } else if starts_paragraph(line) {
            flush(&mut current, &mut out);
            current.push(line);
        } else {
            current.push(line);
        }
    }
    flush(&mut current, &mut out);
    out
}

/// The normalized text of a body, one paragraph per line.
pub fn normalized(text: &str) -> String {
    paragraphs(text).join("\n")
}

/// FNV-1a, so an approval hash means the same thing in every build.
pub fn text_hash(text: &str) -> u64 {
    text.bytes().fold(0xcbf2_9ce4_8422_2325u64, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}
