use super::super::error::{Error, Result};
use super::{Comment, EOL};

use std::io;

use smallvec::SmallVec;

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
        let mut iter = (&mut self.iter)
            .take(8)
            .skip_while(|c| *c == b' ')
            .enumerate();
        while let Some((i, c)) = iter.next() {
            field[i] = c
        }
        field
    }

    pub(crate) fn take16(&mut self) -> [u8; 16] {
        let mut field = [b' '; 16];
        let mut iter = (&mut self.iter)
            .take(16)
            .skip_while(|c| *c == b' ')
            .enumerate();
        while let Some((i, c)) = iter.next() {
            field[i] = c
        }
        field
    }

    pub(crate) fn comment_and_eol(&mut self) -> Result<(Comment, Option<EOL>)> {
        match self.iter.comment.take() {
            Some(comment) => Ok((comment, self.iter.eol)),
            None => Err(Error::UnparsedChars),
        }
    }

    pub(crate) fn end_of_data(&mut self) -> bool {
        self.iter.comment.is_some() || self.iter.peek().is_none()
    }

    pub(crate) fn original(&self) -> Vec<u8> {
        self.original.clone()
    }
}

pub(crate) struct NastranLineIter {
    iter: std::iter::Peekable<std::iter::Enumerate<ExpandTabs<std::vec::IntoIter<u8>>>>,
    comment: Option<Comment>,
    eol: Option<EOL>,
}

impl NastranLineIter {
    pub(crate) fn new(iter: std::vec::IntoIter<u8>) -> Self {
        Self {
            iter: ExpandTabs::new(iter).enumerate().peekable(),
            comment: None,
            eol: None,
        }
    }

    pub(crate) fn peek(&mut self) -> Option<u8> {
        self.iter.peek().map(|c| c.1)
    }

    pub(crate) fn comment_and_eol(&mut self) -> (Option<Comment>, Option<EOL>) {
        (self.comment.take(), self.eol)
    }

    //fn to_comment(&mut self) -> Result<SmallVec<[u8; 8]>> {
    //    self.comment.take().ok_or(Error::UnparsedChars)
    //}
}

impl Iterator for NastranLineIter {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        #[derive(Debug)]
        enum Res {
            Char(u8),
            CharAndEOL(u8),
            DollarSign(u8),
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
            None => return None,
        };
        // There's probably a better way to handle this
        match result {
            Char(c) => Some(c),
            CharAndEOL(c) => {
                let mut comment = SmallVec::new();
                while let Some((_, c)) = self.iter.next() {
                    match c {
                        b'\r' => {
                            self.eol = Some(self::EOL::CRLF);
                            break;
                        }
                        b'\n' => {
                            self.eol = Some(self::EOL::LF);
                            break;
                        }
                        _ => comment.push(c),
                    }
                }
                self.comment = Some(Comment(comment));
                Some(c)
            }
            DollarSign(c) => {
                let mut comment = SmallVec::new();
                comment.push(c);
                while let Some((_, c)) = self.iter.next() {
                    match c {
                        b'\r' => {
                            self.eol = Some(self::EOL::CRLF);
                            break;
                        }
                        b'\n' => {
                            self.eol = Some(self::EOL::LF);
                            break;
                        }
                        _ => comment.push(c),
                    }
                }
                self.comment = Some(Comment(comment));
                None
            }
            CRLF => {
                let comment = SmallVec::new();
                self.comment = Some(Comment(comment));
                self.eol = Some(self::EOL::CRLF);
                None
            }
            LF => {
                let comment = SmallVec::new();
                self.comment = Some(Comment(comment));
                self.eol = Some(self::EOL::LF);
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
                return Some(b' ');
            }
            Some(c) => {
                self.col += 1;
                return Some(c);
            }
            None => return None,
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
