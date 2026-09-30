use std::fmt::{self, Write as _};

pub fn write_byte(byte: u8) -> [char; 2] {
    fn char_hex(x: u8) -> char {
        if x < 10 {
            (b'0' + x) as _
        } else {
            (b'a' + (x - 10)) as _
        }
    }

    [char_hex(byte >> 4), char_hex(byte & 0xf)]
}

pub fn read_byte(msb: u8, lsb: u8) -> Option<u8> {
    fn hex_char(hex: u8) -> Option<u8> {
        if hex.is_ascii_digit() {
            Some(hex - b'0')
        } else if (b'a'..=b'f').contains(&hex) {
            Some(hex - b'a' + 10)
        } else if (b'A'..=b'F').contains(&hex) {
            Some(hex - b'A' + 10)
        } else {
            None
        }
    }

    Some((hex_char(msb)? << 4) | hex_char(lsb)?)
}

pub trait PrintAscii {
    fn print_ascii(&self) -> impl fmt::Display;
}

impl PrintAscii for [u8] {
    fn print_ascii(&self) -> impl fmt::Display {
        AsciiPrinter(self)
    }
}

pub struct AsciiPrinter<'a>(&'a [u8]);

impl fmt::Display for AsciiPrinter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            if byte.is_ascii_whitespace() || byte.is_ascii_graphic() {
                f.write_char((*byte).into())?;
            } else {
                let [msb, lsb] = write_byte(*byte);
                f.write_fmt(format_args!("\\x{msb}{lsb}"))?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_write_byte() {
        assert_eq!(['0', '0'], super::write_byte(0));
        assert_eq!(['0', '1'], super::write_byte(1));
        assert_eq!(['1', '0'], super::write_byte(0x10));
        assert_eq!(['f', 'f'], super::write_byte(0xff));
    }

    #[test]
    fn test_read_byte() {
        assert_eq!(Some(0), super::read_byte(b'0', b'0'));
        assert_eq!(Some(1), super::read_byte(b'0', b'1'));
        assert_eq!(Some(0x10), super::read_byte(b'1', b'0'));
        assert_eq!(Some(0xff), super::read_byte(b'f', b'f'));
        assert_eq!(Some(0xff), super::read_byte(b'F', b'F'));
    }
}
