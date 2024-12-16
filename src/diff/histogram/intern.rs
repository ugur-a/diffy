use core::hash::Hash;

use imara_diff::intern::{Interner, Token, TokenSource};

#[derive(Default)]
pub struct InternedMergeInput<T: Eq + Hash> {
    pub base: Vec<Token>,
    pub left: Vec<Token>,
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
    /// a memory leak. If this function is called often over a long_running process
    /// consider clearing the interner with [`clear`](imara_diff::intern::Interner::clear).
    pub fn update_base(&mut self, input: impl Iterator<Item = T>) {
        self.base.clear();
        self.base
            .extend(input.map(|token| self.interner.intern(token)));
    }

    /// replaces `self.left` wtih the iterned Tokens yielded by `input`
    /// Note that this does not erase any tokens from the interner and might therefore be considered
    /// a memory leak. If this function is called often over a long_running process
    /// consider clearing the interner with [`clear`](imara_diff::intern::Interner::clear) or
    /// [`erase_tokens_after`](crate::intern::Interner::erase_tokens_after).
    pub fn update_left(&mut self, input: impl Iterator<Item = T>) {
        self.left.clear();
        self.left
            .extend(input.map(|token| self.interner.intern(token)));
    }

    /// replaces `self.right` wtih the iterned Tokens yielded by `input`
    /// Note that this does not erase any tokens from the interner and might therefore be considered
    /// a memory leak. If this function is called often over a long_running process
    /// consider clearing the interner with [`clear`](imara_diff::intern::Interner::clear) or
    /// [`erase_tokens_after`](crate::intern::Interner::erase_tokens_after).
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
