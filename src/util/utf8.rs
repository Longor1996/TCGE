use std;
use std::fmt;
use std::io::Read;
use std::io::Error;

pub struct UTF8Read<R> {
    input: R,
    now: char,
    last: char,
    nowpos: FilePosition,
    lastpos: FilePosition,
    unread: bool
}

#[derive(Debug, Copy, Clone)]
pub struct FilePosition {
    row: usize,
    col: usize,
    idx: usize,
}

#[derive(Debug)]
pub enum UTF8ReadError {
    Io(Error),
    Invalid(FilePosition),
    EOF(FilePosition),
    DblUnread,
}

impl<R: Read> UTF8Read<R> {
    pub fn new(input: R) -> UTF8Read<R> {
        UTF8Read {
            input,
            last: 0 as char,
            lastpos: self::FilePosition {
                row: 0,
                col: 0,
                idx: 0
            },
            now: 0 as char,
            nowpos: self::FilePosition {
                row: 0,
                col: 0,
                idx: 0
            },
            unread: false
        }
    }

    fn onebyte(&mut self) -> Result<u8, UTF8ReadError> {
        let mut b0 = [0];
        if self.input.read(&mut b0)? == 1 {
            return Result::Ok(b0[0])
        }
        Result::Err(UTF8ReadError::EOF(self.nowpos))
    }

    pub fn read(&mut self) -> Result<char, UTF8ReadError> {
        if self.unread {
            self.unread = false;
            return Ok(self.now);
        }

        // make 'now' the new 'last'
        self.last = self.now;
        self.lastpos = self.nowpos;

        let mut buf: [u8; 4] = [0, 0, 0, 0];

        buf[0] = self.onebyte()?;

        if buf[0] < 0b10000000 {
            return Ok(buf[0] as char)
        }

        if (buf[0] & 0xC0) == 0x80 {
            return Err(UTF8ReadError::Invalid(self.nowpos))
        }

        if (buf[0] & 0xFE) == 0xFE {
            return Err(UTF8ReadError::Invalid(self.nowpos))
        }

        let mut result = buf[0] as u32;

        let mut sequence_length = 1;
        loop {
            if (buf[0] & (0x80 >> sequence_length)) == 0 {
                break
            }
            sequence_length += 1;
        }

        let mask = match sequence_length {
            1 => 0b1111111,
            2 => 0b11111,
            3 => 0b1111,
            4 => 0b111,
            _ => return Err(UTF8ReadError::Invalid(self.nowpos))
        };

        result &= mask;

        let mut index = 1;
        loop {
            if index < sequence_length {
                break
            }
            buf[index] = self.onebyte()?;
            result = (result << 6) | ((buf[index] as u32) & 0b111111);
            index += 1;
        }

        match std::char::from_u32(result) {
            Some(x) => {
                self.now = x;

                // update position
                self.nowpos.idx += 1;
                if x == '\n' {
                    self.nowpos.row += 1;
                    self.nowpos.col = 1;
                } else {
                    self.nowpos.col += 1;
                }

                Ok(x)
            },
            None => Err(UTF8ReadError::Invalid(self.nowpos))
        }
    }

    pub fn unread(&mut self) -> Result<(),UTF8ReadError> {
        if self.unread {
            return Err(UTF8ReadError::DblUnread);
        }

        self.unread = true;
        Ok(())
    }
}

impl<R> fmt::Debug for UTF8Read<R> where R: fmt::Debug {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("BufReader")
            .field("reader", &self.input)
            .field("now", &self.now)
            .field("last", &self.last)
            .field("now_pos", &self.nowpos)
            .field("last_pos", &self.lastpos)
            .finish()
    }
}

impl From<Error> for UTF8ReadError {
    fn from(error: Error) -> UTF8ReadError {
        UTF8ReadError::Io(error)
    }
}