//! Common utilities

use std::hash::Hash;

use imara_diff::intern::{Interner, Token, TokenSource};

// TODO: remove the trait bounds on new release of imara-diff
//
/// Similar to `InternedInput`, but takes 3 files instead of 2
#[derive(Default)]
pub struct InternedMergeInput<T: Eq + Hash> {
    /// The base revision, aka. "ancestor"
    pub base: Vec<Token>,
    /// The left revision, aka. "ours"
    pub left: Vec<Token>,
    /// The right revision, aka. "theirs"
    pub right: Vec<Token>,
    pub interner: Interner<T>,
}

impl<T: Eq + Hash> InternedMergeInput<T> {
    pub fn new<I: TokenSource<Token = T>>(base: I, left: I, right: I) -> Self {
        let token_estimate_base = base.estimate_tokens() as usize;
        let token_estimate_left = left.estimate_tokens() as usize;
        let token_estimate_right = right.estimate_tokens() as usize;
        let mut res = Self {
            base: Vec::with_capacity(token_estimate_base),
            left: Vec::with_capacity(token_estimate_left),
            right: Vec::with_capacity(token_estimate_right),
            interner: Interner::new(
                token_estimate_base + token_estimate_left + token_estimate_right,
            ),
        };
        res.update_base(base.tokenize());
        res.update_left(left.tokenize());
        res.update_right(right.tokenize());
        res
    }

    /// replaces `self.base` wtih the iterned Tokens yielded by `input`
    /// Note that this does not erase any tokens from the interner and might therefore be considered
    /// a memory leak. If this function is called often over a long-running process
    /// consider clearing the interner with [`clear`](crate::intern::InternedMergeInput::clear).
    pub fn update_base(&mut self, input: impl Iterator<Item = T>) {
        self.base.clear();
        self.base
            .extend(input.map(|token| self.interner.intern(token)));
    }

    /// replaces `self.left` wtih the iterned Tokens yielded by `input`
    /// Note that this does not erase any tokens from the interner and might therefore be considered
    /// a memory leak. If this function is called often over a long-running process
    /// consider clearing the interner with [`clear`](crate::intern::InternedMergeInput::clear) or
    /// [`erase_tokens_after`](https://docs.rs/imara-diff/latest/imara_diff/intern/struct.Interner.html#method.erase_tokens_after).
    pub fn update_left(&mut self, input: impl Iterator<Item = T>) {
        self.left.clear();
        self.left
            .extend(input.map(|token| self.interner.intern(token)));
    }

    /// replaces `self.right` wtih the iterned Tokens yielded by `input`
    /// Note that this does not erase any tokens from the interner and might therefore be considered
    /// a memory leak. If this function is called often over a long-running process
    /// consider clearing the interner with [`clear`](crate::intern::InternedMergeInput::clear) or
    /// [`erase_tokens_after`](https://docs.rs/imara-diff/latest/imara_diff/intern/struct.Interner.html#method.erase_tokens_after).
    pub fn update_right(&mut self, input: impl Iterator<Item = T>) {
        self.right.clear();
        self.right
            .extend(input.map(|token| self.interner.intern(token)));
    }

    pub fn clear(&mut self) {
        self.base.clear();
        self.left.clear();
        self.right.clear();
        self.interner.clear();
    }
}

/// Iterator over the lines of a string, including the `\n` character.
pub struct LineIter<'a, T: ?Sized>(&'a T);

impl<'a, T: ?Sized> LineIter<'a, T> {
    pub fn new(text: &'a T) -> Self {
        Self(text)
    }
}

impl<'a, T: Text + ?Sized> Iterator for LineIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0.is_empty() {
            return None;
        }

        let end = if let Some(idx) = self.0.find("\n") {
            idx + 1
        } else {
            self.0.len()
        };

        let (line, remaining) = self.0.split_at(end);
        self.0 = remaining;
        Some(line)
    }
}

/// A helper trait for processing text like `str` and `[u8]`
/// Useful for abstracting over those types for parsing as well as breaking input into lines
pub trait Text: Eq + Hash {
    fn is_empty(&self) -> bool;
    fn len(&self) -> usize;
    fn starts_with(&self, prefix: &str) -> bool;
    #[allow(unused)]
    fn ends_with(&self, suffix: &str) -> bool;
    fn strip_prefix(&self, prefix: &str) -> Option<&Self>;
    fn strip_suffix(&self, suffix: &str) -> Option<&Self>;
    fn split_at_exclusive(&self, needle: &str) -> Option<(&Self, &Self)>;
    fn find(&self, needle: &str) -> Option<usize>;
    fn split_at(&self, mid: usize) -> (&Self, &Self);
    fn as_str(&self) -> Option<&str>;
    fn as_bytes(&self) -> &[u8];
    #[allow(unused)]
    fn lines(&self) -> LineIter<Self>;

    fn parse<T: std::str::FromStr>(&self) -> Option<T> {
        self.as_str().and_then(|s| s.parse().ok())
    }
}

impl Text for str {
    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn starts_with(&self, prefix: &str) -> bool {
        self.starts_with(prefix)
    }

    fn ends_with(&self, suffix: &str) -> bool {
        self.ends_with(suffix)
    }

    fn strip_prefix(&self, prefix: &str) -> Option<&Self> {
        self.strip_prefix(prefix)
    }

    fn strip_suffix(&self, suffix: &str) -> Option<&Self> {
        self.strip_suffix(suffix)
    }

    fn split_at_exclusive(&self, needle: &str) -> Option<(&Self, &Self)> {
        self.find(needle)
            .map(|idx| (&self[..idx], &self[idx + needle.len()..]))
    }

    fn find(&self, needle: &str) -> Option<usize> {
        self.find(needle)
    }

    fn split_at(&self, mid: usize) -> (&Self, &Self) {
        self.split_at(mid)
    }

    fn as_str(&self) -> Option<&str> {
        Some(self)
    }

    fn as_bytes(&self) -> &[u8] {
        self.as_bytes()
    }

    fn lines(&self) -> LineIter<Self> {
        LineIter::new(self)
    }
}

impl Text for [u8] {
    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn starts_with(&self, prefix: &str) -> bool {
        self.starts_with(prefix.as_bytes())
    }

    fn ends_with(&self, suffix: &str) -> bool {
        self.ends_with(suffix.as_bytes())
    }

    fn strip_prefix(&self, prefix: &str) -> Option<&Self> {
        self.strip_prefix(prefix.as_bytes())
    }

    fn strip_suffix(&self, suffix: &str) -> Option<&Self> {
        self.strip_suffix(suffix.as_bytes())
    }

    fn split_at_exclusive(&self, needle: &str) -> Option<(&Self, &Self)> {
        find_bytes(self, needle.as_bytes()).map(|idx| (&self[..idx], &self[idx + needle.len()..]))
    }

    fn find(&self, needle: &str) -> Option<usize> {
        find_bytes(self, needle.as_bytes())
    }

    fn split_at(&self, mid: usize) -> (&Self, &Self) {
        self.split_at(mid)
    }

    fn as_str(&self) -> Option<&str> {
        std::str::from_utf8(self).ok()
    }

    fn as_bytes(&self) -> &[u8] {
        self
    }

    fn lines(&self) -> LineIter<Self> {
        LineIter::new(self)
    }
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    match needle.len() {
        0 => Some(0),
        1 => find_byte(haystack, needle[0]),
        len if len > haystack.len() => None,
        needle_len => {
            let mut offset = 0;
            let mut haystack = haystack;

            while let Some(position) = find_byte(haystack, needle[0]) {
                offset += position;

                if let Some(haystack) = haystack.get(position..position + needle_len) {
                    if haystack == needle {
                        return Some(offset);
                    }
                } else {
                    return None;
                }

                haystack = &haystack[position + 1..];
                offset += 1;
            }

            None
        }
    }
}

// XXX Maybe use `memchr`?
fn find_byte(haystack: &[u8], byte: u8) -> Option<usize> {
    haystack.iter().position(|&b| b == byte)
}
