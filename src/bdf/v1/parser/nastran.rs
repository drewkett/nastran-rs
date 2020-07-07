struct ExpandTabs<I> {
    iter: I,
    col: usize,
    seen_tab: bool,
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

trait NastranLineIter: Iterator<Item = u8> + Sized {
    fn expand_tabs(self) -> ExpandTabs<Self> {
        ExpandTabs {
            iter: self,
            col: 0,
            seen_tab: false,
        }
    }

    fn take8(&mut self) -> [u8; 8] {
        let mut field = [b' '; 8];
        let mut iter = self.take(8).skip_while(|c| *c == b' ').enumerate();
        while let Some((i, c)) = iter.next() {
            field[i] = c
        }
        field
    }

    fn take16(&mut self) -> [u8; 16] {
        let mut field = [b' '; 16];
        let mut iter = self.take(16).skip_while(|c| *c == b' ').enumerate();
        while let Some((i, c)) = iter.next() {
            field[i] = c
        }
        field
    }
}

impl<I> NastranLineIter for I where I: Iterator<Item = u8> + Sized {}

fn parse_line<I>(line: I)
where
    I: Iterator<Item = u8>,
{
    let mut iter = line.expand_tabs().take_while(|c| *c != b'$').take(80);
    let first: [u8; 8] = iter.take8();
}

fn parse_lines<R>(reader: R) {
    reader.lines().map(parse_line)
}
