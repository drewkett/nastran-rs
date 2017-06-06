
use std::str;
use errors::*;
use super::Field;

#[inline]
fn count_spaces(buffer: &[u8]) -> usize {
    return buffer.iter().take_while(|&&c| c == b' ').count();
}

#[inline]
fn trim_spaces(buffer: &[u8]) -> &[u8] {
    let n = buffer.len();
    let i = buffer.iter().take_while(|&&c| c == b' ').count();
    if i == n {
        return b"";
    }
    let j = buffer.iter()
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
    buffer.iter().take_while(|&&c| is_numeric(c) || is_alpha(c)).count()
}

fn print_slice(s: &str, buffer: &[u8]) {
    let b = unsafe { str::from_utf8_unchecked(buffer) };
    println!("{} {}", s, b)
}

fn maybe_string(buffer: &[u8]) -> Result<Field> {
    let n = buffer.len();
    if n == 0 {
        return Err(ErrorKind::UnexpectedFieldEnd.into());
    }
    if !is_alpha(buffer[0]) {
        return Err(ErrorKind::UnexpectedCharInField.into());
    }
    let mut i = 1;
    i += count_alphanumeric(&buffer[i..]);
    if i > 8 {
        return Err(ErrorKind::UnexpectedCharInField.into());
    }
    // j is the index designating the end of the string
    let j = i;
    i += count_spaces(&buffer[i..]);
    if i == n {
        return Ok(Field::String(&buffer[..j]));
    }
    // '*' can only exist in a first field so field length must be <= 8
    if i < 8 && buffer[i] == b'*' {
        i += 1;
        if i == n {
            return Ok(Field::DoubleString(&buffer[..j]));
        }
    }
    return Err(ErrorKind::UnexpectedCharInField.into());
}

fn maybe_number(buffer: &[u8]) -> Result<Field> {
    let n = buffer.len();
    let mut i = 0;
    if is_plus_minus(buffer[i]) {
        i += 1
    }
    if i == n {
        return Err(ErrorKind::UnexpectedFieldEnd.into());
    }
    let mut try_read_exponent = false;
    if is_numeric(buffer[i]) {
        i += count_digits(&buffer[i..]);
        if i == n {
            if i <= 8 {
                let s = unsafe { str::from_utf8_unchecked(buffer) };
                return s.parse().map(|v| Field::Int(v)).map_err(|e| e.into());
            } else {
                return Err(ErrorKind::UnexpectedCharInField.into());
            }
        } else if buffer[i] == b'.' {
            i += 1;
            i += count_digits(&buffer[i..]);
            if i == n {
                let s = unsafe { str::from_utf8_unchecked(buffer) };
                return s.parse().map(|v| Field::Float(v)).map_err(|e| e.into());
            }
        }
        try_read_exponent = true;
    } else if buffer[i] == b'.' {
        i += 1;
        let n_digits = count_digits(&buffer[i..]);
        if n_digits == 0 {
            return Err(ErrorKind::UnexpectedCharInField.into());
        }
        i += n_digits;
        try_read_exponent = true;
        if i == n {
            let s = unsafe { str::from_utf8_unchecked(buffer) };
            return s.parse().map(|v| Field::Float(v)).map_err(|e| e.into());
        }
    }
    if try_read_exponent {
        if buffer[i] == b'e' || buffer[i] == b'E' {
            i += 1;
            if i == n {
                return Err(ErrorKind::UnexpectedFieldEnd.into());
            }
            if is_plus_minus(buffer[i]) {
                i += 1;
                if i == n {
                    return Err(ErrorKind::UnexpectedFieldEnd.into());
                }
            }
            let n_digits = count_digits(&buffer[i..]);
            if n_digits == 0 || i + n_digits != n {
                return Err(ErrorKind::UnexpectedCharInField.into());
            }
            let s = unsafe { str::from_utf8_unchecked(buffer) };
            return s.parse().map(|v| Field::Float(v)).map_err(|e| e.into());
        } else if buffer[i] == b'+' || buffer[i] == b'-' {
            //j is the index that separates the value from the exponent
            let j = i;
            i += 1;
            if i == n {
                return Err(ErrorKind::UnexpectedFieldEnd.into());
            }
            let n_digits = count_digits(&buffer[i..]);
            if n_digits == 0 || i + n_digits != n {
                return Err(ErrorKind::UnexpectedCharInField.into());
            }
            let mut temp = [b' '; 80];
            temp[..j].copy_from_slice(&buffer[..j]);
            temp[j] = b'e';
            temp[j + 1..n + 1].copy_from_slice(&buffer[j..]);
            let s = unsafe { str::from_utf8_unchecked(&temp[..n + 1]) };
            return s.parse().map(|v| Field::Float(v)).map_err(|e| e.into());
        }
    }
    return Err(ErrorKind::UnexpectedCharInField.into());
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
            return Ok(Field::Continuation(&buffer[1..]));
        } else {
            return Err(ErrorKind::UnexpectedCharInField.into());
        }
    } else if buffer[0] == b'*' {
        let buffer = trim_spaces(buffer);
        let n = buffer.len();
        if n <= 8 {
            return Ok(Field::DoubleContinuation(&buffer[1..]));
        } else {
            return Err(ErrorKind::UnexpectedCharInField.into());
        }
    }
    let buffer = trim_spaces(buffer);
    if buffer.len() == 0 {
        return Ok(Field::Blank);
    }
    match buffer[0] {
        b'a'...b'z' | b'A'...b'Z' => return maybe_string(buffer),
        _ => return Err(ErrorKind::UnexpectedCharInField.into()),
    }
}

pub fn maybe_last_field(buffer: &[u8]) -> Result<Field> {
    let n = buffer.len();
    if n == 0 {
        return Ok(Field::Blank);
    }
    match buffer[0] {
        b'+' | b'*' | b' ' => (),
        _ => return Err(ErrorKind::UnexpectedCharInField.into()),
    }
    let buffer = trim_spaces(&buffer[1..]);
    Ok(Field::Continuation(buffer))
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
            return Ok(Field::Continuation(&buffer[1..]));
        } else {
            return Err(ErrorKind::UnexpectedCharInField.into());
        }
    } else if buffer[0] == b'*' {
        let buffer = trim_spaces(buffer);
        let n = buffer.len();
        if n <= 8 {
            return Ok(Field::DoubleContinuation(&buffer[1..]));
        } else {
            return Err(ErrorKind::UnexpectedCharInField.into());
        }
    }
    let buffer = trim_spaces(buffer);
    if buffer.len() == 0 {
        return Ok(Field::Blank);
    }
    match buffer[0] {
        b'a'...b'z' | b'A'...b'Z' => return maybe_string(buffer),
        b'+' | b'-' | b'0'...b'9' | b'.' => return maybe_number(buffer),
        _ => return Err(ErrorKind::UnexpectedCharInField.into()),
    }
}

pub fn maybe_field(buffer: &[u8]) -> Result<Field> {
    let n = buffer.len();
    if n == 0 {
        return Ok(Field::Blank);
    }
    let buffer = trim_spaces(buffer);
    if buffer.len() == 0 {
        return Ok(Field::Blank);
    }
    match buffer[0] {
        b'a'...b'z' | b'A'...b'Z' => return maybe_string(buffer),
        b'+' | b'-' | b'0'...b'9' | b'.' => return maybe_number(buffer),
        _ => return Err(ErrorKind::UnexpectedCharInField.into()),
    }
}

#[cfg(test)]
mod tests {
    extern crate test;
    use test::Bencher;

    use super::*;

    #[bench]
    fn bench_maybe_field_nastran_float(b: &mut Bencher) {
        b.iter(|| maybe_field(b"11.22+7"));
    }

    #[bench]
    fn bench_maybe_field_float(b: &mut Bencher) {
        b.iter(|| maybe_field(b"11.22e+7"));
    }

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
        success_maybe_first_field("+A B", Field::Continuation(b"A B"));
        success_maybe_first_field("+", Field::Continuation(b""));
        success_maybe_first_field("+       ", Field::Continuation(b""));
        success_maybe_field("HI1", Field::String(b"HI1"));
        success_maybe_field("ABCDEFGH", Field::String(b"ABCDEFGH"));
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
        success_maybe_first_field("HI2*", Field::DoubleString(b"HI2"));
        success_maybe_first_field("HI3 *", Field::DoubleString(b"HI3"));
        success_maybe_first_field("* HI4", Field::DoubleContinuation(b" HI4"));
        success_maybe_field("", Field::Blank);
        success_maybe_field("  ", Field::Blank);
    }
}
