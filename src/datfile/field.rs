use std::str;

use crate::datfile::BufferUtil;
use crate::datfile::Field;
use crate::errors::*;

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
        return Err(Error::UnexpectedFieldEnd(buffer.to_string_lossy()));
    }
    if !is_alpha(buffer[0]) {
        return Err(Error::UnexpectedCharInField(buffer.to_string_lossy()));
    }
    let mut i = 1;
    i += count_alphanumeric(&buffer[i..]);
    if i > 8 {
        return Err(Error::UnexpectedCharInField(buffer.to_string_lossy()));
    }
    // j is the index designating the end of the string
    let j = i;
    i += count_spaces(&buffer[i..]);
    if i == n {
        let s = str::from_utf8(&buffer[..j])?;
        return Ok(Field::String(s));
    }
    // '*' can only exist in a first field so field length must be <= 8
    if i < 8 && buffer[i] == b'*' {
        i += 1;
        if i == n {
            let s = str::from_utf8(&buffer[..j])?;
            return Ok(Field::DoubleString(s));
        }
    }
    Err(Error::UnexpectedCharInField(buffer.to_string_lossy()))
}

fn maybe_number(buffer: &[u8]) -> Result<Field> {
    let n = buffer.len();
    let mut i = 0;
    if is_plus_minus(buffer[i]) {
        i += 1
    }
    if i == n {
        return Err(Error::UnexpectedFieldEnd(buffer.to_string_lossy()));
    }
    let mut try_read_exponent = false;
    if is_numeric(buffer[i]) {
        i += count_digits(&buffer[i..]);
        if i == n {
            if i <= 8 {
                let s = str::from_utf8(buffer)?;
                return s.parse().map(Field::Int).map_err(|e| e.into());
            } else {
                return Err(Error::UnexpectedCharInField(buffer.to_string_lossy()));
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
            return Err(Error::UnexpectedCharInField(buffer.to_string_lossy()));
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
                return Err(Error::UnexpectedFieldEnd(buffer.to_string_lossy()));
            }
            if is_plus_minus(buffer[i]) {
                i += 1;
                if i == n {
                    return Err(Error::UnexpectedFieldEnd(buffer.to_string_lossy()));
                }
            }
            let n_digits = count_digits(&buffer[i..]);
            if n_digits == 0 || i + n_digits != n {
                return Err(Error::UnexpectedCharInField(buffer.to_string_lossy()));
            }
            let s = str::from_utf8(buffer)?;
            return s.parse().map(Field::Float).map_err(|e| e.into());
        } else if buffer[i] == b'd' || buffer[i] == b'D' {
            // j is the idnex of 'd' or 'D'. Needed for later replacing the value
            let j = i;
            i += 1;
            if i == n {
                return Err(Error::UnexpectedFieldEnd(buffer.to_string_lossy()));
            }
            if is_plus_minus(buffer[i]) {
                i += 1;
                if i == n {
                    return Err(Error::UnexpectedFieldEnd(buffer.to_string_lossy()));
                }
            }
            let n_digits = count_digits(&buffer[i..]);
            if n_digits == 0 || i + n_digits != n {
                return Err(Error::UnexpectedCharInField(buffer.to_string_lossy()));
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
                return Err(Error::UnexpectedFieldEnd(buffer.to_string_lossy()));
            }
            let n_digits = count_digits(&buffer[i..]);
            if n_digits == 0 || i + n_digits != n {
                return Err(Error::UnexpectedCharInField(buffer.to_string_lossy()));
            }
            let mut temp = [b' '; 80];
            temp[..j].copy_from_slice(&buffer[..j]);
            temp[j] = b'e';
            temp[j + 1..n + 1].copy_from_slice(&buffer[j..]);
            let s = str::from_utf8(&temp[..n + 1])?;
            return s.parse().map(Field::Float).map_err(|e| e.into());
        }
    }
    Err(Error::UnexpectedCharInField(buffer.to_string_lossy()))
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
            let s = str::from_utf8(&buffer[1..])?;
            return Ok(Field::Continuation(s));
        } else {
            return Err(Error::UnexpectedCharInField(buffer.to_string_lossy()));
        }
    } else if buffer[0] == b'*' {
        let buffer = trim_spaces(buffer);
        let n = buffer.len();
        if n <= 8 {
            let s = str::from_utf8(&buffer[1..])?;
            return Ok(Field::DoubleContinuation(s));
        } else {
            return Err(Error::UnexpectedCharInField(buffer.to_string_lossy()));
        }
    }
    let buffer = trim_spaces(buffer);
    if buffer.is_empty() {
        return Ok(Field::Blank);
    }
    match buffer[0] {
        b'a'..=b'z' | b'A'..=b'Z' => maybe_string(buffer),
        _ => Err(Error::UnexpectedCharInField(buffer.to_string_lossy())),
    }
}

pub fn trailing_continuation(buffer: &[u8]) -> Result<&str> {
    let n = buffer.len();
    if n == 0 {
        return Ok("");
    }
    match buffer[0] {
        b'+' | b'*' | b' ' => (),
        _ => return Err(Error::UnexpectedCharInField(buffer.to_string_lossy())),
    }
    let s = str::from_utf8(&buffer[1..])?;
    Ok(s.trim())
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
            let s = str::from_utf8(&buffer[1..])?;
            return Ok(Field::Continuation(s));
        } else {
            return Err(Error::UnexpectedCharInField(buffer.to_string_lossy()));
        }
    } else if buffer[0] == b'*' {
        let buffer = trim_spaces(buffer);
        let n = buffer.len();
        if n <= 8 {
            let s = str::from_utf8(&buffer[1..])?;
            return Ok(Field::DoubleContinuation(s));
        } else {
            return Err(Error::UnexpectedCharInField(buffer.to_string_lossy()));
        }
    }
    let buffer = trim_spaces(buffer);
    if buffer.is_empty() {
        return Ok(Field::Blank);
    }
    match buffer[0] {
        b'a'..=b'z' | b'A'..=b'Z' => maybe_string(buffer),
        b'+' | b'-' | b'0'..=b'9' | b'.' => maybe_number(buffer),
        _ => Err(Error::UnexpectedCharInField(buffer.to_string_lossy())),
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
        _ => Err(Error::UnexpectedCharInField(buffer.to_string_lossy())),
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
        success_maybe_first_field("+A B", Field::Continuation("A B"));
        success_maybe_first_field("+", Field::Continuation(""));
        success_maybe_first_field("+       ", Field::Continuation(""));
        success_maybe_field("HI1", Field::String("HI1"));
        success_maybe_field("ABCDEFGH", Field::String("ABCDEFGH"));
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
        success_maybe_first_field("HI2*", Field::DoubleString("HI2"));
        success_maybe_first_field("HI3 *", Field::DoubleString("HI3"));
        success_maybe_first_field("* HI4", Field::DoubleContinuation(" HI4"));
        success_maybe_field("", Field::Blank);
        success_maybe_field("  ", Field::Blank);
    }
}
