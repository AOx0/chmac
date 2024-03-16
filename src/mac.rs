use rand::prelude::*;
use rand::Fill;
use std::ffi::c_char;

#[derive(Debug, Default, Clone, Copy)]
pub struct Mac(pub [c_char; 14]);

#[derive(Debug, thiserror::Error)]
pub enum Invalid {
    #[error("Address {0:#?} does not have the correct format \"xx:xx:xx:xx:xx:xx\"")]
    MismatchAddrSize(String),
    #[error("Invalid octet {0}: value {1:#?} from address {2:#?} is not valid hexadecimal 0-255")]
    WrongHexValue(usize, String, String),
    #[error("Invalid octet {0}: multicast bit is set in {1:0>8b} from address {3:#?}")]
    MultiCastBitSet(usize, i8, [i8; 14], String),
}

impl std::fmt::Display for Mac {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, hx) in self.0.iter().enumerate().take(6) {
            write!(f, "{hx:0>2x}")?;
            if i + 1 < 6 {
                write!(f, ":")?;
            }
        }

        Ok(())
    }
}

impl Mac {
    pub fn bytes(&self) -> [c_char; 14] {
        self.0
    }

    pub fn rand() -> Self {
        let mut res = Self::default();

        let mut rng = thread_rng();

        res.0[..6]
            .try_fill(&mut rng)
            .expect("We are passing valid values");
        if res.0[0] % 2 != 0 {
            res.0[0] = res.0[0].checked_sub(1).unwrap_or_else(|| {
                res.0[0]
                    .checked_add(1)
                    .expect("If the sub operation failed is impossible to fail addition")
            });
        }

        res
    }
}

impl<'a> TryFrom<&'a str> for Mac {
    type Error = Invalid;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        if value.chars().filter(|c| c == &':').count() == 5 {
            let mut res = Mac::default();
            value.split(':').enumerate().try_for_each(|(i, c)| {
                let Ok(v) = u8::from_str_radix(c, 16) else {
                    return Err(Invalid::WrongHexValue(i, c.to_string(), value.to_string()));
                };
                res.0[i] = unsafe { std::mem::transmute(v) };
                Ok(())
            })?;

            let lower_bits = res.0[0] & 0b0000_1111;
            if lower_bits % 2 != 0 {
                Err(Invalid::MultiCastBitSet(
                    0,
                    res.0[0],
                    res.0,
                    value.to_string(),
                ))
            } else {
                Ok(res)
            }
        } else {
            Err(Invalid::MismatchAddrSize(value.to_string()))
        }
    }
}
