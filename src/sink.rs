use core::{hash::Hash, ops};

use imara_diff::{
    intern::{InternedInput, Token},
    Sink,
};

use crate::range::{DiffRange, Range};

pub(crate) struct DiffyDiffRangeBuilder<'a> {
    before: &'a [Token],
    after: &'a [Token],

    prev_before_end: usize,
    prev_after_end: usize,

    dst: Vec<DiffRange<'a, 'a, [Token]>>,
}

impl<'a> DiffyDiffRangeBuilder<'a> {
    pub fn from_tokens(before: &'a [Token], after: &'a [Token]) -> Self {
        Self {
            before,
            after,

            prev_before_end: 0,
            prev_after_end: 0,
            dst: vec![],
        }
    }

    pub fn new<T: Eq + Hash>(input: &'a InternedInput<T>) -> Self {
        Self {
            before: &input.before,
            after: &input.after,

            prev_before_end: 0,
            prev_after_end: 0,
            dst: vec![],
        }
    }
}

impl<'a> Sink for DiffyDiffRangeBuilder<'a> {
    type Out = Vec<DiffRange<'a, 'a, [Token]>>;

    fn process_change(&mut self, before: ops::Range<u32>, after: ops::Range<u32>) {
        let before = before.start as usize..before.end as usize;
        let after = after.start as usize..after.end as usize;

        let unchanged_before_range = self.prev_before_end..before.start;
        let hunk_before_range = before.start..before.end;
        let unchanged_after_range = self.prev_after_end..after.start;
        let hunk_after_range = after.start..after.end;

        if !unchanged_before_range.is_empty() || !unchanged_after_range.is_empty() {
            self.dst.push(DiffRange::Equal(
                Range::new(self.before, unchanged_before_range),
                Range::new(self.after, unchanged_after_range),
            ));
        }
        if !hunk_before_range.is_empty() {
            self.dst.push(DiffRange::Delete(Range::new(
                self.before,
                hunk_before_range,
            )));
        }
        if !hunk_after_range.is_empty() {
            self.dst
                .push(DiffRange::Insert(Range::new(self.after, hunk_after_range)));
        };

        (self.prev_before_end, self.prev_after_end) = (before.end, after.end);
    }

    fn finish(mut self) -> Self::Out {
        let before_till_end = self.prev_before_end..self.before.len();
        let after_till_end = self.prev_after_end..self.after.len();

        if !before_till_end.is_empty() || !after_till_end.is_empty() {
            self.dst.push(DiffRange::Equal(
                Range::new(self.before, before_till_end),
                Range::new(self.after, after_till_end),
            ));
        }

        self.dst
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use imara_diff::{intern::InternedInput, sources::lines_with_terminator};

    #[test]
    fn equal_insert_equal() {
        let before = "A\nB\nD\n";
        let after = "A\nB\nC\nD\n";

        let input = InternedInput::new(lines_with_terminator(before), lines_with_terminator(after));
        let diff = imara_diff::diff_with_tokens(
            imara_diff::Algorithm::Histogram,
            &input.before,
            &input.after,
            input.interner.num_tokens(),
            DiffyDiffRangeBuilder::new(&input),
        );

        assert_eq!(
            &diff,
            &[
                DiffRange::Equal(
                    Range::new(input.before.as_slice(), 0..2),
                    Range::new(&input.after, 0..2),
                ),
                DiffRange::Insert(Range::new(&input.after, 2..3)),
                DiffRange::Equal(
                    Range::new(&input.before, 2..3),
                    Range::new(&input.after, 3..4)
                )
            ]
        );
    }

    #[test]
    fn equal_insert_equal_delete_equal() {
        let before = "A\nC\nD\nE\n";
        let after = "A\nB\nC\nE\n";

        let input = InternedInput::new(lines_with_terminator(before), lines_with_terminator(after));
        let diff = imara_diff::diff_with_tokens(
            imara_diff::Algorithm::Histogram,
            &input.before,
            &input.after,
            input.interner.num_tokens(),
            DiffyDiffRangeBuilder::new(&input),
        );

        assert_eq!(
            &diff,
            &[
                DiffRange::Equal(
                    Range::new(input.before.as_slice(), 0..1),
                    Range::new(&input.after, 0..1),
                ),
                DiffRange::Insert(Range::new(&input.after, 1..2)),
                DiffRange::Equal(
                    Range::new(&input.before, 1..2),
                    Range::new(&input.after, 2..3)
                ),
                DiffRange::Delete(Range::new(&input.before, 2..3)),
                DiffRange::Equal(
                    Range::new(&input.before, 3..4),
                    Range::new(&input.after, 3..4)
                )
            ]
        );
    }

    #[test]
    fn equal_delete_insert_equal() {
        let before = "A\nD\nE\n";
        let after = "A\nB\nE\n";

        let input = InternedInput::new(lines_with_terminator(before), lines_with_terminator(after));
        let diff = imara_diff::diff_with_tokens(
            imara_diff::Algorithm::Histogram,
            &input.before,
            &input.after,
            input.interner.num_tokens(),
            DiffyDiffRangeBuilder::new(&input),
        );

        assert_eq!(
            &diff,
            &[
                DiffRange::Equal(
                    Range::new(input.before.as_slice(), 0..1),
                    Range::new(&input.after, 0..1),
                ),
                DiffRange::Delete(Range::new(&input.before, 1..2)),
                DiffRange::Insert(Range::new(&input.after, 1..2)),
                DiffRange::Equal(
                    Range::new(&input.before, 2..3),
                    Range::new(&input.after, 2..3)
                )
            ]
        );
    }

    #[test]
    fn insert_equal() {
        let before = "B\n";
        let after = "A\nB\n";

        let input = InternedInput::new(lines_with_terminator(before), lines_with_terminator(after));
        let diff = imara_diff::diff_with_tokens(
            imara_diff::Algorithm::Histogram,
            &input.before,
            &input.after,
            input.interner.num_tokens(),
            DiffyDiffRangeBuilder::new(&input),
        );

        assert_eq!(
            &diff,
            &[
                DiffRange::Insert(Range::new(input.after.as_slice(), 0..1)),
                DiffRange::Equal(
                    Range::new(&input.before, 0..1),
                    Range::new(&input.after, 1..2),
                ),
            ]
        );
    }

    #[test]
    fn insert() {
        let before = "";
        let after = "A\n";

        let input = InternedInput::new(lines_with_terminator(before), lines_with_terminator(after));
        let diff = imara_diff::diff_with_tokens(
            imara_diff::Algorithm::Histogram,
            &input.before,
            &input.after,
            input.interner.num_tokens(),
            DiffyDiffRangeBuilder::new(&input),
        );

        assert_eq!(
            &diff,
            &[DiffRange::Insert(Range::new(input.after.as_slice(), 0..1))]
        );
    }

    #[test]
    fn delete() {
        let before = "A\n";
        let after = "";

        let input = InternedInput::new(lines_with_terminator(before), lines_with_terminator(after));
        let diff = imara_diff::diff_with_tokens(
            imara_diff::Algorithm::Histogram,
            &input.before,
            &input.after,
            input.interner.num_tokens(),
            DiffyDiffRangeBuilder::new(&input),
        );

        assert_eq!(
            &diff,
            &[DiffRange::Delete(Range::new(input.before.as_slice(), 0..1))]
        );
    }

    #[test]
    fn empty() {
        let before = "";
        let after = "";

        let input = InternedInput::new(lines_with_terminator(before), lines_with_terminator(after));
        let diff = imara_diff::diff_with_tokens(
            imara_diff::Algorithm::Histogram,
            &input.before,
            &input.after,
            input.interner.num_tokens(),
            DiffyDiffRangeBuilder::new(&input),
        );

        assert_eq!(&diff, &[]);
    }
}
