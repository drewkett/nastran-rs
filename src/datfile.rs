mod card;
mod deck;
mod field;

pub use self::field::{maybe_any_field, maybe_field};
pub use card::Card;
pub use deck::{parse_buffer, parse_line, Deck};
pub use field::Field;

pub trait BufferUtil {
    fn to_string_lossy(self) -> String;
}

impl<'a> BufferUtil for &'a [u8] {
    fn to_string_lossy(self) -> String {
        String::from_utf8_lossy(self).into_owned()
    }
}
