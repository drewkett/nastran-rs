use std::{cmp, fmt, str};

use bstr::ByteSlice;

use super::{Error, Result};

//TODO Need to make sure right number of fields are being output for card
#[derive(PartialEq, Clone, Copy)]
pub enum Field {
    Blank,
    Int(i32),
    Float(f32),
    Double(f64),
    Continuation([u8; 8]),
    DoubleContinuation([u8; 8]),
    String([u8; 8]),
    DoubleString([u8; 8]),
}

impl fmt::Debug for Field {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Field::Blank => write!(f, "Blank"),
            Field::Int(i) => write!(f, "Int({})", i),
            Field::Float(d) => write!(f, "Float({})", d),
            Field::Double(d) => write!(f, "Double({})", d),
            Field::Continuation(c) => write!(f, "Continuation('{}')", c.as_bstr()),
            Field::String(s) => write!(f, "String('{}')", s.as_bstr()),
            Field::DoubleContinuation(s) => write!(f, "DoubleContinuation('{}')", s.as_bstr()),
            Field::DoubleString(s) => write!(f, "DoubleString('{}')", s.as_bstr()),
        }
    }
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let width = match f.width() {
            Some(8) | None => 8,
            Some(16) => 16,
            Some(_) => return Err(fmt::Error),
        };
        if width == 8 {
            match *self {
                Field::Blank => write!(f, "        "),
                Field::Int(i) => write!(f, "{:8}", i),
                Field::Float(d) => write!(f, "{:>8}", float_to_8(d)),
                Field::Double(d) => write!(f, "{:>8}", float_to_8(d)),
                Field::Continuation(c) => write!(f, "+{:7}", c.as_bstr()),
                Field::String(s) => write!(f, "{:8}", s.as_bstr()),
                Field::DoubleContinuation(c) => write!(f, "*{:7}", c.as_bstr()),
                Field::DoubleString(s) => write!(f, "{:7}*", s.as_bstr()),
            }
        } else if width == 16 {
            match *self {
                Field::Blank => write!(f, "                "),
                Field::Int(i) => write!(f, "{:16}", i),
                Field::Float(d) => write!(f, "{:>16}", float_to_16(d)),
                Field::Double(d) => write!(f, "{:>16}", float_to_16(d)),
                Field::Continuation(_) => unreachable!(),
                Field::String(s) => write!(f, "{:16}", s.as_bstr()),
                Field::DoubleContinuation(_) => unreachable!(),
                Field::DoubleString(s) => write!(f, "{:15}*", s.as_bstr()),
            }
        } else {
            unreachable!()
        }
    }
}

#[inline]
fn count_spaces(buffer: &[u8]) -> usize {
    buffer.iter().take_while(|&&c| c == b' ').count()
}

#[inline]
fn trim_spaces(buffer: &[u8]) -> &[u8] {
    let n = buffer.len();
    let i = buffer.iter().take_while(|&&c| c == b' ').count();
    if i == n {
        return b"";
    }
    let j = buffer
        .iter()
        .skip(i)
        .rev()
        .take_while(|&&c| c == b' ')
        .count();
    &buffer[i..n - j]
}

#[inline]
fn is_plus_minus(c: u8) -> bool {
    c == b'+' || c == b'-'
}

#[inline]
fn is_alpha(c: u8) -> bool {
    (c >= b'a' && c <= b'z') || (c >= b'A' && c <= b'Z')
}

#[inline]
fn is_numeric(c: u8) -> bool {
    c >= b'0' && c <= b'9'
}

#[inline]
fn count_digits(buffer: &[u8]) -> usize {
    buffer.iter().take_while(|&&c| is_numeric(c)).count()
}

#[inline]
fn count_alphanumeric(buffer: &[u8]) -> usize {
    buffer
        .iter()
        .take_while(|&&c| is_numeric(c) || is_alpha(c))
        .count()
}

//fn print_slice(s: &str, buffer: &[u8]) {
//let b = unsafe { str::from_utf8_unchecked(buffer) };
//println!("{} {}", s, b)
//}

fn maybe_string(buffer: &[u8]) -> Result<Field> {
    let n = buffer.len();
    if n == 0 {
        return Err(Error::UnexpectedFieldEnd(buffer));
    }
    if !is_alpha(buffer[0]) {
        return Err(Error::UnexpectedCharInField(buffer));
    }
    let mut i = 1;
    i += count_alphanumeric(&buffer[i..]);
    if i > 8 {
        return Err(Error::UnexpectedCharInField(buffer));
    }
    // j is the index designating the end of the string
    let j = i;
    i += count_spaces(&buffer[i..]);
    if i == n {
        let mut dst = [b' '; 8];
        dst[..j].copy_from_slice(&buffer[..j]);
        return Ok(Field::String(dst));
    }
    // '*' can only exist in a first field so field length must be <= 8
    if i < 8 && buffer[i] == b'*' {
        i += 1;
        if i == n {
            let mut dst = [b' '; 8];
            dst[..j].copy_from_slice(&buffer[..j]);
            return Ok(Field::DoubleString(dst));
        }
    }
    Err(Error::UnexpectedCharInField(buffer))
}

fn maybe_number(buffer: &[u8]) -> Result<Field> {
    let n = buffer.len();
    let mut i = 0;
    if is_plus_minus(buffer[i]) {
        i += 1
    }
    if i == n {
        return Err(Error::UnexpectedFieldEnd(buffer));
    }
    let mut try_read_exponent = false;
    if is_numeric(buffer[i]) {
        i += count_digits(&buffer[i..]);
        if i == n {
            if i <= 8 {
                let s = str::from_utf8(buffer)?;
                return s.parse().map(Field::Int).map_err(|e| e.into());
            } else {
                return Err(Error::UnexpectedCharInField(buffer));
            }
        } else if buffer[i] == b'.' {
            i += 1;
            i += count_digits(&buffer[i..]);
            if i == n {
                let s = str::from_utf8(buffer)?;
                return s.parse().map(Field::Float).map_err(|e| e.into());
            }
        }
        try_read_exponent = true;
    } else if buffer[i] == b'.' {
        i += 1;
        let n_digits = count_digits(&buffer[i..]);
        if n_digits == 0 {
            return Err(Error::UnexpectedCharInField(buffer));
        }
        i += n_digits;
        try_read_exponent = true;
        if i == n {
            let s = str::from_utf8(buffer)?;
            return s.parse().map(Field::Float).map_err(|e| e.into());
        }
    }
    if try_read_exponent {
        if buffer[i] == b'e' || buffer[i] == b'E' {
            i += 1;
            if i == n {
                return Err(Error::UnexpectedFieldEnd(buffer));
            }
            if is_plus_minus(buffer[i]) {
                i += 1;
                if i == n {
                    return Err(Error::UnexpectedFieldEnd(buffer));
                }
            }
            let n_digits = count_digits(&buffer[i..]);
            if n_digits == 0 || i + n_digits != n {
                return Err(Error::UnexpectedCharInField(buffer));
            }
            let s = str::from_utf8(buffer)?;
            return s.parse().map(Field::Float).map_err(|e| e.into());
        } else if buffer[i] == b'd' || buffer[i] == b'D' {
            // j is the idnex of 'd' or 'D'. Needed for later replacing the value
            let j = i;
            i += 1;
            if i == n {
                return Err(Error::UnexpectedFieldEnd(buffer));
            }
            if is_plus_minus(buffer[i]) {
                i += 1;
                if i == n {
                    return Err(Error::UnexpectedFieldEnd(buffer));
                }
            }
            let n_digits = count_digits(&buffer[i..]);
            if n_digits == 0 || i + n_digits != n {
                return Err(Error::UnexpectedCharInField(buffer));
            }
            let mut temp = [b' '; 80];
            temp[..n].copy_from_slice(buffer);
            temp[j] = b'e';
            let s = str::from_utf8(&temp[..n])?;
            return s.parse().map(Field::Double).map_err(|e| e.into());
        } else if buffer[i] == b'+' || buffer[i] == b'-' {
            //j is the index that separates the value from the exponent
            let j = i;
            i += 1;
            if i == n {
                return Err(Error::UnexpectedFieldEnd(buffer));
            }
            let n_digits = count_digits(&buffer[i..]);
            if n_digits == 0 || i + n_digits != n {
                return Err(Error::UnexpectedCharInField(buffer));
            }
            let mut temp = [b' '; 80];
            temp[..j].copy_from_slice(&buffer[..j]);
            temp[j] = b'e';
            temp[j + 1..n + 1].copy_from_slice(&buffer[j..]);
            let s = str::from_utf8(&temp[..n + 1])?;
            return s.parse().map(Field::Float).map_err(|e| e.into());
        }
    }
    Err(Error::UnexpectedCharInField(buffer))
}

pub fn maybe_first_field(buffer: &[u8]) -> Result<Field> {
    let n = buffer.len();
    if n == 0 {
        return Ok(Field::Blank);
    }
    if buffer[0] == b'+' {
        let buffer = trim_spaces(buffer);
        let n = buffer.len();
        if n <= 8 {
            let mut dst = [b' '; 8];
            dst[..n - 1].copy_from_slice(&buffer[1..]);
            return Ok(Field::Continuation(dst));
        } else {
            return Err(Error::UnexpectedCharInField(buffer));
        }
    } else if buffer[0] == b'*' {
        let buffer = trim_spaces(buffer);
        let n = buffer.len();
        if n <= 8 {
            let mut dst = [b' '; 8];
            dst[..n - 1].copy_from_slice(&buffer[1..]);
            return Ok(Field::DoubleContinuation(dst));
        } else {
            return Err(Error::UnexpectedCharInField(buffer));
        }
    }
    let buffer = trim_spaces(buffer);
    if buffer.is_empty() {
        return Ok(Field::Blank);
    }
    match buffer[0] {
        b'a'..=b'z' | b'A'..=b'Z' => maybe_string(buffer),
        _ => Err(Error::UnexpectedCharInField(buffer)),
    }
}

pub fn trailing_continuation(buffer: &[u8]) -> Result<[u8; 8]> {
    let n = buffer.len();
    if n > 8 {
        return Err(Error::UnexpectedFieldEnd(buffer));
    }
    if n == 0 {
        return Ok(*b"        ");
    }
    match buffer[0] {
        b'+' | b'*' | b' ' => (),
        _ => return Err(Error::UnexpectedCharInField(buffer)),
    }
    let mut dst = [b' '; 8];
    dst[..n - 1].copy_from_slice(&buffer[1..]);
    Ok(dst)
}

pub fn maybe_any_field(buffer: &[u8]) -> Result<Field> {
    let n = buffer.len();
    if n == 0 {
        return Ok(Field::Blank);
    }
    if buffer[0] == b'+' {
        let buffer = trim_spaces(buffer);
        let n = buffer.len();
        if n > 1 && (is_numeric(buffer[1]) || buffer[1] == b'.') {
            return maybe_number(buffer);
        } else if n <= 8 {
            let mut dst = [b' '; 8];
            dst[..n - 1].copy_from_slice(&buffer[1..n]);
            return Ok(Field::Continuation(dst));
        } else {
            return Err(Error::UnexpectedCharInField(buffer));
        }
    } else if buffer[0] == b'*' {
        let buffer = trim_spaces(buffer);
        let n = buffer.len();
        if n <= 8 {
            let mut dst = [b' '; 8];
            dst[..n - 1].copy_from_slice(&buffer[1..n]);
            return Ok(Field::DoubleContinuation(dst));
        } else {
            return Err(Error::UnexpectedCharInField(buffer));
        }
    }
    let buffer = trim_spaces(buffer);
    if buffer.is_empty() {
        return Ok(Field::Blank);
    }
    match buffer[0] {
        b'a'..=b'z' | b'A'..=b'Z' => maybe_string(buffer),
        b'+' | b'-' | b'0'..=b'9' | b'.' => maybe_number(buffer),
        _ => Err(Error::UnexpectedCharInField(buffer)),
    }
}

pub fn maybe_field(buffer: &[u8]) -> Result<Field> {
    let n = buffer.len();
    if n == 0 {
        return Ok(Field::Blank);
    }
    let buffer = trim_spaces(buffer);
    if buffer.is_empty() {
        return Ok(Field::Blank);
    }
    match buffer[0] {
        b'a'..=b'z' | b'A'..=b'Z' => maybe_string(buffer),
        b'+' | b'-' | b'0'..=b'9' | b'.' => maybe_number(buffer),
        _ => Err(Error::UnexpectedCharInField(buffer)),
    }
}

fn float_to_8<T>(f: T) -> String
where
    T: Into<f64> + Copy + fmt::Display + fmt::LowerExp + dtoa::Floating + cmp::PartialOrd,
{
    // FIXME: can be improved
    let mut buf = [b' '; 9];
    if let Ok(n) = dtoa::write(&mut buf[..], f) {
        unsafe { String::from_utf8_unchecked(buf[..n].to_vec()) }
    } else {
        let s = if f.into() <= -1e+10 {
            format!("{:8.1e}", f)
        } else if f.into() < -1e-10 {
            format!("{:8.2e}", f)
        } else if f.into() < 0.0 {
            format!("{:8.1e}", f)
        } else if f.into() <= 1e-10 {
            format!("{:8.2e}", f)
        } else if f.into() < 1e+10 {
            format!("{:8.3e}", f)
        } else {
            format!("{:8.2e}", f)
        };
        if s.len() > 8 {
            panic!("help '{}'", s)
        }
        s
    }
}

fn float_to_16<T>(f: T) -> String
where
    T: Copy + fmt::Display + fmt::LowerExp + dtoa::Floating,
{
    // FIXME: can be improved
    let mut buf = [b' '; 16];
    if let Ok(n) = dtoa::write(&mut buf[..], f) {
        unsafe { String::from_utf8_unchecked(buf[..n].to_vec()) }
    } else {
        let s = format!("{:16.8e}", f);
        if s.len() > 16 {
            panic!("Couldn't write {} in less than 16 chars '{}'", f, s)
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn success_maybe_field(test: &str, result: Field) {
        match maybe_field(test.as_bytes()) {
            Ok(r) => assert_eq!(r, result),
            Err(e) => panic!("Expected Ok for '{}' got '{}'", test, e),
        }
    }

    fn success_maybe_first_field(test: &str, result: Field) {
        match maybe_first_field(test.as_bytes()) {
            Ok(r) => assert_eq!(r, result),
            Err(e) => panic!("Expected Ok for '{}' got '{}'", test, e),
        }
    }

    #[test]
    fn test_maybe_field() {
        success_maybe_first_field("+A B", Field::Continuation(*b"A B     "));
        success_maybe_first_field("+", Field::Continuation(*b"        "));
        success_maybe_first_field("+       ", Field::Continuation(*b"        "));
        success_maybe_field("HI1", Field::String(*b"HI1     "));
        success_maybe_field("ABCDEFGH", Field::String(*b"ABCDEFGH"));
        success_maybe_field(" 2.23 ", Field::Float(2.23));
        success_maybe_field("+2.24 ", Field::Float(2.24));
        success_maybe_field(" 2.25e7 ", Field::Float(2.25e7));
        success_maybe_field(" 2.26e+7 ", Field::Float(2.26e7));
        success_maybe_field(" 2.27e-7 ", Field::Float(2.27e-7));
        success_maybe_field(" .28 ", Field::Float(0.28));
        success_maybe_field(" .29e+7 ", Field::Float(0.29e7));
        success_maybe_field(" 30e+7 ", Field::Float(3e8));
        success_maybe_field(" 3.1+7 ", Field::Float(3.1e7));
        success_maybe_field(" 3.+7 ", Field::Float(3.0e7));
        success_maybe_field(" .2+7 ", Field::Float(0.2e7));
        success_maybe_field(" .2-7 ", Field::Float(0.2e-7));
        success_maybe_first_field("HI2*", Field::DoubleString(*b"HI2     "));
        success_maybe_first_field("HI3 *", Field::DoubleString(*b"HI3     "));
        success_maybe_first_field("* HI4", Field::DoubleContinuation(*b" HI4    "));
        success_maybe_field("", Field::Blank);
        success_maybe_field("  ", Field::Blank);
    }
}
