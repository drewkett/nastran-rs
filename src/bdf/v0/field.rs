use std::{cmp, fmt};

use bstr::ByteSlice;

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

fn float_to_8<T>(f: T) -> String
where
    T: Into<f64> + Copy + fmt::Display + fmt::LowerExp + dtoa::Floating + cmp::PartialOrd,
{
    // FIXME: can be improved
    let mut buf = vec![b' '; 8];
    let f = f.into();
    if let Ok(_n) = dtoa::write(&mut buf[..], f) {
        unsafe { String::from_utf8_unchecked(buf) }
    } else {
        let s = if f <= -1e+100 {
            format!("{:<8.1e}", f)
        } else if f < -1e+10 {
            format!("{:<8.2e}", f)
        } else if f < -1e+0 {
            format!("{:<8.3e}", f)
        } else if f < -1e-9 {
            format!("{:<8.2e}", f)
        } else if f < -1e-99 {
            format!("{:<8.1e}", f)
        } else if f < 0.0 {
            format!("{:<8.0e}", f)
        } else if f <= 1e-99 {
            format!("{:<8.1e}", f)
        } else if f <= 1e-9 {
            format!("{:<8.2e}", f)
        } else if f < 1e+0 {
            format!("{:<8.3e}", f)
        } else if f < 1e+10 {
            format!("{:<8.4e}", f)
        } else if f < 1e+100 {
            format!("{:<8.3e}", f)
        } else {
            format!("{:<8.2e}", f)
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

    #[test]
    fn test_float_to_8() {
        assert_eq!(float_to_8(1.234), "1.234   ".to_string());
        assert_eq!(float_to_8(1.234567), "1.234567".to_string());
        assert_eq!(float_to_8(-1.234), "-1.234  ".to_string());
        assert_eq!(float_to_8(-1.23456), "-1.23456".to_string());

        assert_eq!(float_to_8(1.23456e-100), "1.2e-100".to_string());
        assert_eq!(float_to_8(1.23456e-10), "1.23e-10".to_string());
        assert_eq!(float_to_8(1.23456e-9), "1.235e-9".to_string());
        assert_eq!(float_to_8(0.012345678), "1.235e-2".to_string());
        assert_eq!(float_to_8(0.12345678), "1.235e-1".to_string());
        assert_eq!(float_to_8(1.2345678), "1.2346e0".to_string());
        assert_eq!(float_to_8(12.345678), "1.2346e1".to_string());
        assert_eq!(float_to_8(123.45678), "1.2346e2".to_string());
        assert_eq!(float_to_8(1.23456e9), "1.2346e9".to_string());
        assert_eq!(float_to_8(1.23456e10), "1.235e10".to_string());
        assert_eq!(float_to_8(1.23456e100), "1.23e100".to_string());

        assert_eq!(float_to_8(-1.23456e-100), "-1e-100 ".to_string());
        assert_eq!(float_to_8(-1.23456e-10), "-1.2e-10".to_string());
        assert_eq!(float_to_8(-1.23456e-9), "-1.23e-9".to_string());
        assert_eq!(float_to_8(-0.012345678), "-1.23e-2".to_string());
        assert_eq!(float_to_8(-0.12345678), "-1.23e-1".to_string());
        assert_eq!(float_to_8(-1.2345678), "-1.235e0".to_string());
        assert_eq!(float_to_8(-12.345678), "-1.235e1".to_string());
        assert_eq!(float_to_8(-123.45678), "-1.235e2".to_string());
        assert_eq!(float_to_8(-1.23456e9), "-1.235e9".to_string());
        assert_eq!(float_to_8(-1.23456e10), "-1.23e10".to_string());
        assert_eq!(float_to_8(-1.23456e100), "-1.2e100".to_string());
    }
}
