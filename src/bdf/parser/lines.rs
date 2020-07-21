use super::super::error::{Error, Result};
use super::{Comment, EOL};

use std::io;

pub(crate) struct NastranLine {
    original: Vec<u8>,
    iter: NastranLineIter,
}

impl NastranLine {
    pub(crate) fn new(line: Vec<u8>) -> Self {
        // Add comma check here?
        NastranLine {
            original: line.clone(),
            iter: NastranLineIter::new(line.into_iter()),
        }
    }

    pub(crate) fn take8(&mut self) -> [u8; 8] {
        let mut field = [b' '; 8];
        let iter = (&mut self.iter)
            .take(8)
            .skip_while(|c| *c == b' ')
            .enumerate();
        for (i, c) in iter {
            field[i] = c
        }
        field
    }

    pub(crate) fn take16(&mut self) -> [u8; 16] {
        let mut field = [b' '; 16];
        let iter = (&mut self.iter)
            .take(16)
            .skip_while(|c| *c == b' ')
            .enumerate();
        for (i, c) in iter {
            field[i] = c
        }
        field
    }

    pub(crate) fn comment_and_eol(&mut self) -> Result<(Comment, Option<EOL>)> {
        self.iter.comment_and_eol().ok_or_else(|| {
            let chars = (&mut self.iter).collect();
            Error::UnparsedChars(chars)
        })
    }

    pub(crate) fn end_of_data(&mut self) -> bool {
        self.iter.state != NastranLineIterState::Parsing
    }

    pub(crate) fn original(&self) -> Vec<u8> {
        self.original.clone()
    }
}

#[derive(PartialEq, Clone)]
pub(crate) enum NastranLineIterState {
    Parsing,
    Comment(Comment, Option<EOL>),
    End,
}

pub(crate) struct NastranLineIter {
    iter: std::iter::Peekable<std::iter::Enumerate<ExpandTabs<std::vec::IntoIter<u8>>>>,
    state: NastranLineIterState,
}

impl NastranLineIter {
    pub(crate) fn new(iter: std::vec::IntoIter<u8>) -> Self {
        Self {
            iter: ExpandTabs::new(iter).enumerate().peekable(),
            state: NastranLineIterState::Parsing,
        }
    }

    pub(crate) fn peek(&mut self) -> Option<u8> {
        self.iter.peek().map(|c| c.1)
    }

    pub(crate) fn comment_and_eol(&mut self) -> Option<(Comment, Option<EOL>)> {
        // TODO this is a mess
        let (state, res) = match &self.state {
            NastranLineIterState::Comment(comment, eol) => {
                (NastranLineIterState::End, Some((comment.clone(), *eol)))
            }
            s => (s.clone(), None),
        };
        self.state = state;
        res
    }
}

impl Iterator for NastranLineIter {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        match self.state {
            NastranLineIterState::Comment(_, _) | NastranLineIterState::End => return None,
            _ => {}
        }
        #[derive(Debug)]
        enum Res {
            Char(u8),
            CharAndEOL(u8),
            DollarSign(u8),
            EOL,
            CRLF,
            LF,
        }
        use Res::*;
        // Be careful here. The ordering matters so that the EOL is processed
        let result = match self.iter.next() {
            Some((_, b'$')) => DollarSign(b'$'),
            Some((_, b'\n')) => LF,
            Some((_, b'\r')) => CRLF,
            Some((79, c @ b'a'..=b'z')) => CharAndEOL(c - 32),
            Some((79, c)) => CharAndEOL(c),
            Some((_, c @ b'a'..=b'z')) => Char(c - 32),
            Some((_, c)) => Char(c),
            None => EOL,
        };
        // There's probably a better way to handle this
        match result {
            Char(c) => Some(c),
            CharAndEOL(c) => {
                let mut comment = Comment::new();
                let mut eol = None;
                while let Some((_, c)) = self.iter.next() {
                    match c {
                        b'\r' => {
                            eol = Some(self::EOL::LF);
                            break;
                        }
                        b'\n' => {
                            eol = Some(self::EOL::LF);
                            break;
                        }
                        _ => comment.push(c),
                    }
                }
                self.state = NastranLineIterState::Comment(comment, eol);
                Some(c)
            }
            DollarSign(c) => {
                let mut comment = Comment::new();
                let mut eol = None;
                comment.push(c);
                while let Some((_, c)) = self.iter.next() {
                    match c {
                        b'\r' => {
                            eol = Some(self::EOL::CRLF);
                            break;
                        }
                        b'\n' => {
                            eol = Some(self::EOL::LF);
                            break;
                        }
                        _ => comment.push(c),
                    }
                }
                self.state = NastranLineIterState::Comment(comment, eol);
                None
            }
            EOL => {
                self.state = NastranLineIterState::Comment(Comment::new(), None);
                None
            }
            CRLF => {
                self.state = NastranLineIterState::Comment(Comment::new(), Some(self::EOL::CRLF));
                None
            }
            LF => {
                self.state = NastranLineIterState::Comment(Comment::new(), Some(self::EOL::LF));
                None
            }
        }
    }
}

struct ExpandTabs<I> {
    iter: I,
    col: usize,
    seen_tab: bool,
}

impl<I> ExpandTabs<I>
where
    I: Sized,
{
    fn new(iter: I) -> Self {
        ExpandTabs {
            iter,
            col: 0,
            seen_tab: false,
        }
    }
}

impl<I> Iterator for ExpandTabs<I>
where
    I: Iterator<Item = u8>,
{
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.seen_tab && self.col % 8 != 0 {
            self.col += 1;
            return Some(b' ');
        }
        match self.iter.next() {
            Some(b'\t') => {
                self.seen_tab = true;
                self.col += 1;
                Some(b' ')
            }
            Some(c) => {
                self.col += 1;
                Some(c)
            }
            None => None,
        }
    }
}

pub(crate) struct SplitLines<I> {
    iter: I,
}

impl<I> Iterator for SplitLines<I>
where
    I: Iterator<Item = io::Result<u8>> + Sized,
{
    type Item = Result<Vec<u8>>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut result = Vec::with_capacity(80);
        loop {
            match self.iter.next() {
                Some(Ok(c)) => {
                    result.push(c);
                    if c == b'\n' {
                        break;
                    }
                }
                Some(Err(e)) => return Some(Err(e.into())),
                None => break,
            }
        }
        if result.is_empty() {
            None
        } else {
            Some(Ok(result))
        }
    }
}

pub(crate) trait NastranFileIter: Iterator<Item = io::Result<u8>> + Sized {
    fn split_lines(self) -> SplitLines<Self> {
        SplitLines { iter: self }
    }
}

impl<I> NastranFileIter for I where I: Iterator<Item = io::Result<u8>> + Sized {}
