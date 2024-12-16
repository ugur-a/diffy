use core::ops::Range;

use imara_diff::{intern::Token, Sink};

use crate::range::{DiffRange, Range as crateRange};

pub(crate) struct DiffyDiffRangeBuilder<'a> {
    before: &'a [Token],
    after: &'a [Token],

    prev_before: Range<usize>,
    prev_after: Range<usize>,

    dst: Vec<DiffRange<'a, 'a, [Token]>>,
}

impl<'a> DiffyDiffRangeBuilder<'a> {
    pub fn new(before: &'a [Token], after: &'a [Token]) -> Self {
        Self {
            before,
            after,

            prev_before: 0..0,
            prev_after: 0..0,
            dst: vec![],
        }
    }
}

impl<'a> Sink for DiffyDiffRangeBuilder<'a> {
    type Out = Vec<DiffRange<'a, 'a, [Token]>>;

    fn process_change(&mut self, before: Range<u32>, after: Range<u32>) {
        let before = before.start as usize..before.end as usize;
        let after = after.start as usize..after.end as usize;
        dbg!(&before, &after);

        let unchanged_before_range = self.prev_before.end..before.start;
        let hunk_before_range = before.start..before.end;
        let unchanged_after_range = self.prev_after.end..after.start;
        let hunk_after_range = after.start..after.end;
        dbg!(
            &unchanged_before_range,
            &hunk_before_range,
            &unchanged_after_range,
            &hunk_after_range
        );

        let unchanged_before = &self.before[unchanged_before_range.clone()];
        let hunk_before = &self.before[hunk_before_range.clone()];
        let unchanged_after = &self.after[unchanged_after_range.clone()];
        let hunk_after = &self.after[hunk_after_range.clone()];
        eprintln!("{:?}", unchanged_before);
        eprintln!("{:?}", hunk_before);
        eprintln!("{:?}", unchanged_after);
        eprintln!("{:?}", hunk_after);

        if !unchanged_before.is_empty() || !unchanged_after.is_empty() {
            debug_assert_eq!(unchanged_before, unchanged_after);
            self.dst.push(DiffRange::Equal(
                crateRange::new(unchanged_before, self.prev_before.end..before.start),
                crateRange::new(unchanged_after, unchanged_after_range),
            ));
        }
        match (hunk_before.is_empty(), hunk_after.is_empty()) {
            (false, true) => {
                self.dst.push(DiffRange::Delete(crateRange::new(
                    self.before,
                    hunk_before_range,
                )));
            }
            (true, false) => {
                dbg!(&hunk_after);
                self.dst.push(DiffRange::Insert(crateRange::new(
                    &self.after[hunk_after_range.start..],
                    hunk_after_range,
                )));
            }
            _ => {}
        };

        (self.prev_before, self.prev_after) = (before, after);
    }

    fn finish(self) -> Self::Out {
        self.dst
    }
}
