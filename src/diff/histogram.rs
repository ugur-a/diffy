use crate::range::{DiffRange, Range as crateRange};
use core::ops::Range;
use imara_diff::{intern::InternedInput, Algorithm};

pub fn diff<'a, 'b>(old: &'a str, new: &'b str) -> Vec<DiffRange<'a, 'b, str>> {
    let mut diff_ranges = vec![];

    let mut prev_before = 0..0;
    let mut prev_after = 0..0;

    let input = InternedInput::new(old, new);
    let sink = |before: Range<u32>, after: Range<u32>| {
        let before = before.start as usize..before.end as usize;
        let after = after.start as usize..after.end as usize;

        let unchanged_before_range = prev_before.end..before.start;
        let hunk_before_range = before.start..before.end;
        let unchanged_after_range = prev_after.end..after.start;
        let hunk_after_range = after.start..after.end;

        let unchanged_before = &input.before[unchanged_before_range.clone()];
        let hunk_before = &input.before[hunk_before_range.clone()];
        let unchanged_after = &input.after[unchanged_after_range.clone()];
        let hunk_after = &input.after[hunk_after_range.clone()];

        if !unchanged_before.is_empty() || !&unchanged_after.is_empty() {
            debug_assert!(!unchanged_before.is_empty() && !unchanged_after.is_empty());
            diff_ranges.push(DiffRange::Equal(
                crateRange::new(old, unchanged_before_range),
                crateRange::new(new, unchanged_after_range),
            ));
        } else {
            if !hunk_before.is_empty() {
                diff_ranges.push(DiffRange::Delete(crateRange::new(old, hunk_before_range)));
            }
            if !hunk_after.is_empty() {
                diff_ranges.push(DiffRange::Insert(crateRange::new(new, hunk_after_range)));
            }
        }
        (prev_before, prev_after) = (before, after);
    };

    imara_diff::diff(Algorithm::Histogram, &input, sink);

    diff_ranges
}

mod diffy_diff_range;
mod intern;

pub(crate) use diffy_diff_range::DiffyDiffRangeBuilder;
pub(crate) use intern::InternedMergeInput;
