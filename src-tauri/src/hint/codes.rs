use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap, VecDeque};

/// Precomputed once at startup. Allocation during a hint session remains an
/// indexed lookup, so progressive scans never rename an already visible hint.
pub(super) fn generate(charsets: &[Vec<char>], extra: &[char]) -> Vec<String> {
    if charsets.is_empty() || charsets.iter().any(Vec::is_empty) {
        return Vec::new();
    }
    let rows: Vec<&[char]> = charsets.iter().map(Vec::as_slice).collect();
    let mut codes = Vec::new();
    append_codes(&rows, &mut codes);
    if !extra.is_empty() {
        let mut extended = vec![extra];
        extended.extend_from_slice(&rows);
        // Exhaust the shorter codes before introducing the extra prefix.
        append_codes(&extended, &mut codes);
    }
    codes
}

type Layer = BTreeMap<usize, VecDeque<String>>;

fn append_codes(rows: &[&[char]], codes: &mut Vec<String>) {
    // A code's rank is the greatest character index used in any row. Finishing
    // rank k exhausts every combination of the first k+1 characters of each row.
    // The map stores only nonempty prefix buckets, including for unequal rows.
    let mut layers = vec![Layer::new(); rows.iter().map(|row| row.len()).max().unwrap()];
    for (first_index, &first) in rows[0].iter().enumerate() {
        collect_suffixes(
            &rows[1..],
            &mut first.to_string(),
            first_index,
            first_index,
            &mut layers,
        );
    }

    let mut previous = None;
    for layer in layers {
        let mut buckets: Vec<_> = layer.into_values().collect();
        let prefixes: Vec<_> = buckets
            .iter()
            .map(|bucket| bucket.front().unwrap().chars().next().unwrap())
            .collect();
        let mut remaining: BinaryHeap<_> = buckets
            .iter()
            .enumerate()
            .map(|(index, bucket)| (bucket.len(), Reverse(index)))
            .collect();
        // Use the largest bucket first so a new prefix cannot pile up at the
        // end. Break ties in configured order; avoid repeating the last prefix
        // whenever this layer still has an alternative.
        while let Some(mut next) = remaining.pop() {
            let (_, Reverse(index)) = next;
            if Some(prefixes[index]) == previous && !remaining.is_empty() {
                let alternative = remaining.pop().unwrap();
                remaining.push(next);
                next = alternative;
            }
            let (count, Reverse(index)) = next;
            codes.push(buckets[index].pop_front().unwrap());
            previous = Some(prefixes[index]);
            if count > 1 {
                remaining.push((count - 1, Reverse(index)));
            }
        }
    }
}

fn collect_suffixes(
    rows: &[&[char]],
    text: &mut String,
    rank: usize,
    first_index: usize,
    layers: &mut [Layer],
) {
    let Some((row, rest)) = rows.split_first() else {
        layers[rank]
            .entry(first_index)
            .or_default()
            .push_back(text.clone());
        return;
    };
    for (index, &character) in row.iter().enumerate() {
        let length = text.len();
        text.push(character);
        collect_suffixes(rest, text, rank.max(index), first_index, layers);
        text.truncate(length);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn rows(values: &[&str]) -> Vec<Vec<char>> {
        values.iter().map(|value| value.chars().collect()).collect()
    }

    #[test]
    fn familiar_characters_expand_in_stages_with_interleaved_prefixes() {
        assert_eq!(
            generate(&rows(&["ASD", "JKL"]), &[]),
            ["AJ", "SJ", "AK", "SK", "DJ", "AL", "DK", "SL", "DL",]
        );
    }

    #[test]
    fn shorter_codes_finish_before_extra_prefixes_expand_together() {
        let codes = generate(&rows(&["AS", "JK"]), &['Q', 'W', ';']);
        assert_eq!(&codes[..4], ["AJ", "SJ", "AK", "SK"]);
        assert_eq!(
            &codes[4..12],
            ["QAJ", "WAJ", "QAK", "WAK", "QSJ", "WSJ", "QSK", "WSK",]
        );
        assert!(codes[12..].iter().all(|code| code.starts_with(';')));
        assert_eq!(codes.len(), 16);
        assert_eq!(codes.iter().collect::<HashSet<_>>().len(), codes.len());
    }

    // Independent contract checks over all combinations of small row lengths:
    // complete Cartesian coverage, increasing familiarity rank, and no repeated
    // prefix when that same rank still offers another prefix.
    fn check_contract(charsets: &[Vec<char>]) {
        let codes = generate(charsets, &[]);
        assert_eq!(
            codes,
            generate(charsets, &[]),
            "ordering must be repeatable"
        );
        assert_eq!(
            codes.len(),
            charsets.iter().map(Vec::len).product::<usize>()
        );
        assert_eq!(codes.iter().collect::<HashSet<_>>().len(), codes.len());
        let mut ranked = BTreeMap::<usize, Vec<&String>>::new();
        let mut previous_rank = 0;
        for code in &codes {
            assert_eq!(code.chars().count(), charsets.len());
            let rank = code
                .chars()
                .zip(charsets)
                .map(|(character, row)| {
                    row.iter()
                        .position(|&c| c == character)
                        .expect("character belongs to its position")
                })
                .max()
                .unwrap();
            assert!(
                rank >= previous_rank,
                "familiar combinations must finish first"
            );
            previous_rank = rank;
            ranked.entry(rank).or_default().push(code);
        }
        let mut previous = None;
        let mut used = 0;
        for (rank, layer) in ranked {
            used += layer.len();
            assert_eq!(
                used,
                charsets
                    .iter()
                    .map(|row| row.len().min(rank + 1))
                    .product::<usize>()
            );
            for (index, code) in layer.iter().enumerate() {
                let first = code.chars().next().unwrap();
                if Some(first) == previous {
                    assert!(
                        layer[index..].iter().all(|other| other.starts_with(first)),
                        "repeated a prefix while another was available"
                    );
                }
                previous = Some(first);
            }
        }
    }

    #[test]
    fn arbitrary_row_counts_and_unequal_lengths_preserve_the_contract() {
        for a in 1..=4 {
            check_contract(&rows(&[&"ASDF"[..a]]));
            for b in 1..=4 {
                check_contract(&rows(&[&"ASDF"[..a], &"JKL;"[..b]]));
                for c in 1..=4 {
                    check_contract(&rows(&[&"QWER"[..a], &"ASDF"[..b], &"JKL;"[..c]]));
                }
            }
        }
        check_contract(&rows(&["QW", "AS", "JK", "12"]));
    }

    #[test]
    fn shipped_configuration_retains_all_two_and_three_character_codes() {
        let config: crate::config::Config =
            toml::from_str(include_str!("../../config_macos.toml")).unwrap();
        let charsets = &config.hint.charsets;
        let extra = &config.hint.charset_extra;
        let codes = generate(charsets, extra);
        let short_count = charsets.iter().map(Vec::len).product::<usize>();
        assert_eq!(short_count, 784);
        assert_eq!(codes.len(), short_count * (extra.len() + 1));
        assert_eq!(codes.iter().collect::<HashSet<_>>().len(), codes.len());
        assert!(codes[..short_count].iter().all(|code| code.len() == 2));
        assert!(codes[short_count..].iter().all(|code| {
            let mut chars = code.chars();
            extra.contains(&chars.next().unwrap())
                && chars.zip(charsets).all(|(c, row)| row.contains(&c))
                && code.len() == 3
        }));
        assert!(codes[..short_count]
            .iter()
            .all(|code| !extra.contains(&code.chars().next().unwrap())));
        let mut extended = vec![extra.clone()];
        extended.extend_from_slice(charsets);
        check_contract(&extended);
    }

    #[test]
    fn empty_configuration_never_allocates_empty_or_partial_codes() {
        assert!(generate(&[], &['Q']).is_empty());
        assert!(generate(&rows(&["AS", ""]), &['Q']).is_empty());
        assert!(generate(&rows(&["", "JK"]), &[]).is_empty());
    }
}
