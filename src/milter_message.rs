use std::convert::{TryFrom, TryInto};

use crate::accept_reject_action::AcceptRejectAction;
use crate::milter_error::MilterError;

use regex::Regex;

#[derive(Debug)]
pub(crate) enum MilterMessage {
    AbortFilterChecks,
    BodyChunk {
        value: String,
    },
    ConnectionInformation {
        hostname: String,
        family: ProtocolFamily,
        port: u16,
        address: String,
    },
    DefineMacros {
        cmdcode: char,
        macros: Vec<MilterMacro>,
    },
    EndOfBody,
    EndOfHeader,
    Header {
        name: String,
        value: String,
    },
    Helo {
        msg: String,
    },
    MailFrom {
        sender: String,
        args: Vec<String>,
    },
    OptionNegotiation {
        version: u32,
        actions: MilterActions,
        protocol: MilterProtocol,
    },
    QuitCommunication,
    RecipientInformation {
        recipient: String,
        args: Vec<String>,
    },
}

impl TryFrom<&[u8]> for MilterMessage {
    type Error = MilterError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        match value {
            [b'A'] => Ok(MilterMessage::AbortFilterChecks),
            [b'B', rest @ ..] => Ok(MilterMessage::BodyChunk {
                value: String::from_utf8_lossy(rest).into(),
            }),
            [b'C', rest @ ..] => {
                let hostname_end = rest
                    .iter()
                    .position(|b| b == &0u8)
                    .ok_or(MilterError::IncompleteMessage)?;

                let hostname = String::from_utf8_lossy(&rest[..hostname_end]).into();
                let family = match rest
                    .get(hostname_end + 1)
                    .ok_or(MilterError::IncompleteMessage)?
                {
                    b'L' => Ok(ProtocolFamily::UnixSocket),
                    b'4' => Ok(ProtocolFamily::Inet4),
                    b'6' => Ok(ProtocolFamily::Inet6),
                    _ => Err(MilterError::IncompleteMessage),
                }?;
                let port =
                    u16::from_be_bytes(rest[hostname_end + 2..=hostname_end + 3].try_into()?);
                let address = String::from_utf8_lossy(&rest[hostname_end + 4..rest.len() - 1]);

                Ok(MilterMessage::ConnectionInformation {
                    hostname,
                    family,
                    port,
                    address: address.into(),
                })
            }
            [b'D', cmdcode, rest @ ..] => {
                if !rest.is_empty() {
                    let buf = rest[..rest.len() - 1].split(|b| b == &0u8);
                    let (names, values): (Vec<_>, Vec<_>) =
                        buf.enumerate().partition(|(i, _)| i % 2 == 0);

                    if names.len() != values.len() {
                        Err(MilterError::IncompleteMessage)
                    } else {
                        let mut macros = Vec::with_capacity(names.len());

                        for (i, name) in names {
                            if let Some((_, value)) = values.get(i) {
                                macros.push(MilterMacro {
                                    name: String::from_utf8_lossy(name).into(),
                                    value: String::from_utf8_lossy(value).into(),
                                });
                            }
                        }
                        Ok(MilterMessage::DefineMacros {
                            cmdcode: char::from(*cmdcode),
                            macros,
                        })
                    }
                } else {
                    Ok(MilterMessage::DefineMacros {
                        cmdcode: char::from(*cmdcode),
                        macros: Vec::new(),
                    })
                }
            }
            [b'E'] => Ok(MilterMessage::EndOfBody),
            [b'H', rest @ ..] => Ok(MilterMessage::Helo {
                msg: String::from_utf8_lossy(&rest[..rest.len() - 1]).into(),
            }),
            [b'L', rest @ ..] => {
                let mut buf = rest.split(|b| b == &0u8);
                let name = buf.next().ok_or(MilterError::IncompleteMessage)?;
                let value = buf.next().ok_or(MilterError::IncompleteMessage)?;

                Ok(MilterMessage::Header {
                    name: String::from_utf8_lossy(name).into(),
                    value: decode(String::from_utf8_lossy(value)),
                })
            }
            [b'M', rest @ ..] => {
                let mut buf = rest.split(|b| b == &0u8);
                let sender =
                    String::from_utf8_lossy(buf.next().ok_or(MilterError::IncompleteMessage)?);

                let args = buf
                    .map(|split| String::from_utf8_lossy(split).into())
                    .collect();

                Ok(MilterMessage::MailFrom {
                    sender: sender.into(),
                    args,
                })
            }
            [b'N'] => Ok(MilterMessage::EndOfHeader),
            [b'O', rest @ ..] if rest.len() == 12 => Ok(MilterMessage::OptionNegotiation {
                version: u32::from_be_bytes(rest[0..=3].try_into()?),
                actions: MilterActions::from_bits_truncate(u32::from_be_bytes(
                    rest[4..=7].try_into()?,
                )),
                protocol: MilterProtocol::from_bits_truncate(u32::from_be_bytes(
                    rest[8..=11].try_into()?,
                )),
            }),
            [b'Q'] => Ok(MilterMessage::QuitCommunication),
            [b'R', rest @ ..] => {
                let mut buf = rest.split(|b| b == &0u8);
                let recipient =
                    String::from_utf8_lossy(buf.next().ok_or(MilterError::IncompleteMessage)?);

                let args = buf
                    .map(|split| String::from_utf8_lossy(split).into())
                    .collect();

                Ok(MilterMessage::RecipientInformation {
                    recipient: recipient.into(),
                    args,
                })
            }
            [identifier, ..] => Err(MilterError::UnknowMessageIdentifier(char::from(
                *identifier,
            ))),
            _ => Err(MilterError::MissingMessageIdentifier),
        }
    }
}

/// A macro defined by the MTA.
#[derive(Debug)]
pub struct MilterMacro {
    /// The name of the macro.
    name: String,
    /// The macro value.
    value: String,
}

/// The protocol family used (currently only Inet4 and Inet6 are supported).
#[derive(Debug)]
pub enum ProtocolFamily {
    /// Unix socket.
    UnixSocket,
    /// IPv4
    Inet4,
    /// IPv6
    Inet6,
}

bitflags! {
    pub(crate) struct MilterActions: u32 {
        const ADD_HEADERS = 1;
        const CHANGE_BODY = 1 << 1;
        const ADD_RECIPIENTS = 1 << 2;
        const REMOVE_RECIPIENTS = 1 << 3;
        const CHANGE_HEADERS = 1 << 4;
        const QUARANTINE = 1 << 5;
    }
}

bitflags! {
    /// Used for defining which message parts should be excluded for the Milter
    #[derive(Default)]
    pub struct MilterProtocol: u32 {
        const NO_CONNECT = 1;
        const NO_HELO = 1 << 1;
        const NO_MAIL = 1 << 2;
        const NO_RECIPIENT = 1 << 3;
        const NO_BODY = 1 << 4;
        const NO_HEADER = 1 << 5;
        const NO_EOH = 1 << 6;
    }
}

#[derive(Debug)]
pub(crate) struct ResponseMessage {
    content: Vec<u8>,
}

impl From<AcceptRejectAction> for ResponseMessage {
    fn from(action: AcceptRejectAction) -> ResponseMessage {
        ResponseMessage {
            content: match action {
                AcceptRejectAction::Accept => {
                    let mut buf = Vec::with_capacity(5);
                    buf.append(&mut u32::to_be_bytes(1).to_vec());
                    buf.push(b'a');
                    buf
                }
                AcceptRejectAction::Continue => {
                    let mut buf = Vec::with_capacity(5);
                    buf.append(&mut u32::to_be_bytes(1).to_vec());
                    buf.push(b'c');
                    buf
                }
                AcceptRejectAction::Discard => {
                    let mut buf = Vec::with_capacity(5);
                    buf.append(&mut u32::to_be_bytes(1).to_vec());
                    buf.push(b'd');
                    buf
                }
                AcceptRejectAction::Reject => {
                    let mut buf = Vec::with_capacity(5);
                    buf.append(&mut u32::to_be_bytes(1).to_vec());
                    buf.push(b'r');
                    buf
                }
                AcceptRejectAction::Tempfail => {
                    let mut buf = Vec::with_capacity(5);
                    buf.append(&mut u32::to_be_bytes(1).to_vec());
                    buf.push(b't');
                    buf
                }
            },
        }
    }
}

impl ResponseMessage {
    pub(crate) fn get_content(&self) -> &[u8] {
        &self.content
    }

    pub(crate) fn option_negotiation(
        version: u32,
        actions: MilterActions,
        protocol: &MilterProtocol,
    ) -> Self {
        // OPTNEG buffer length is always 17
        let mut buf = Vec::with_capacity(17);

        // OPTNEG length is always 13
        let mut length = u32::to_be_bytes(13).to_vec();

        buf.append(&mut length);
        buf.push(b'O');

        buf.append(&mut version.to_be_bytes().to_vec());
        buf.append(&mut actions.bits().to_be_bytes().to_vec());
        buf.append(&mut protocol.bits.to_be_bytes().to_vec());

        Self { content: buf }
    }
}

fn decode<S: AsRef<str>>(s: S) -> String {
    lazy_static! {
        static ref REGEX: Regex =
            Regex::new(r"(?P<start>=\?)(?P<charset>.*)\?(?P<transfer_encoding>.*)\?(?P<encoded_value>.*)(?P<end>\?=)")
                .expect("Can't compile regex for decoding");
    }

    let mut res = String::with_capacity(s.as_ref().len());
    let mut last_end = 0;

    for capture in REGEX.captures_iter(s.as_ref()) {
        if let Some(decoded_string) = decode_captures(capture) {
            if decoded_string.start > last_end {
                let rest: String = s
                    .as_ref()
                    .chars()
                    .skip(last_end)
                    .take(decoded_string.start - last_end)
                    .collect();
                res.push_str(&rest);
            }

            res.push_str(&decoded_string.value);
            last_end = decoded_string.end;
        }
    }

    // Append rest (if any)
    let input_len = s.as_ref().chars().count();
    if input_len > last_end {
        let rest: String = s
            .as_ref()
            .chars()
            .skip(last_end)
            .take(input_len - last_end)
            .collect();
        res.push_str(&rest);
    }

    res
}

fn decode_captures(c: regex::Captures) -> Option<DecodedString> {
    let start = c.name("start")?.start();
    let end = c.name("end")?.end();
    let charset = c.name("charset")?;

    if let Some(charset) = charset::Charset::for_label_no_replacement(charset.as_str().as_bytes()) {
        let transfer_encoding = c.name("transfer_encoding")?.as_str();
        let encoded_value = c.name("encoded_value")?.as_str();

        let decoded = match transfer_encoding {
            "b" | "B" => Some(base64::decode(encoded_value).ok()?),
            "q" | "Q" => Some(
                quoted_printable::decode(
                    encoded_value.replace("_", " "),
                    quoted_printable::ParseMode::Robust,
                )
                .ok()?,
            ),
            _ => None,
        };

        if let Some(decoded) = decoded {
            let (decoded, _) = charset.decode_without_bom_handling(&decoded);

            Some(DecodedString {
                start,
                end,
                value: decoded.to_string(),
            })
        } else {
            None
        }
    } else {
        None
    }
}

struct DecodedString {
    pub start: usize,
    pub end: usize,
    pub value: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_milter_actions_add_recipients() {
        let x: [u8; 4] = [0, 0, 0, 4];
        let res = MilterActions::from_bits_truncate(u32::from_be_bytes(x));
        let comp = MilterActions::ADD_RECIPIENTS;

        assert_eq!(comp, res);
    }

    #[test]
    fn create_milter_actions_add_headers_and_quarantine() {
        let x: [u8; 4] = [0, 0, 0, 33];
        let res = MilterActions::from_bits_truncate(u32::from_be_bytes(x));
        let comp = MilterActions::ADD_HEADERS | MilterActions::QUARANTINE;

        assert_eq!(comp, res);
    }

    #[test]
    fn create_milter_protocol_no_mail() {
        let x: [u8; 4] = [0, 0, 0, 4];
        let res = MilterProtocol::from_bits_truncate(u32::from_be_bytes(x));
        let comp = MilterProtocol::NO_MAIL;

        assert_eq!(comp, res);
    }

    #[test]
    fn create_milter_protocol_no_body() {
        let x: [u8; 4] = [0, 0, 0, 16];
        let res = MilterProtocol::from_bits_truncate(u32::from_be_bytes(x));
        let comp = MilterProtocol::NO_BODY;

        assert_eq!(comp, res);
    }

    #[test]
    fn create_milter_protocol_no_connect_and_header() {
        let x: [u8; 4] = [0, 0, 0, 33];
        let res = MilterProtocol::from_bits_truncate(u32::from_be_bytes(x));
        let comp = MilterProtocol::NO_CONNECT | MilterProtocol::NO_HEADER;

        assert_eq!(comp, res);
    }

    #[test]
    fn decode_utf8_base64() {
        // Taken from an actual spam mail which contained padding chars
        let input = "=?utf-8?B?IkjDtmhsZSBkZXIgTMO2d2VuIiBTeXN0ZW0gbWFjaHQgRGV1dHNjaGUgQsO8cmdlciByZWljaCE=?=";
        let res = decode(input);
        let comp = "\"Höhle der Löwen\" System macht Deutsche Bürger reich!";

        assert_eq!(comp, res);
    }

    #[test]
    fn decode_utf8_base64_with_not_encoded() {
        // Taken from an actual spam mail and added 'not encoded' to test that we keep non-encoded
        // data
        let input = "not encoded=?utf-8?B?4oCeSMO2aGxlIGRlciBMw7Z3ZW7igJwgU3lzdGVtIG1hY2h0IERldXRzY2hlIELDvHJnZXIgcmVpY2gh?=not encoded";
        let res = decode(input);
        let comp = "not encoded„Höhle der Löwen“ System macht Deutsche Bürger reich!not encoded";

        assert_eq!(comp, res);
    }

    /// Used for testing that we keep the original input with broken encoding
    #[test]
    fn decode_utf8_base64_broken_encoding() {
        let input =
            "not encoded=?utf-8?B?w7Z3ZW7igJ2h0IERldXRzY2hlIELDvHJnZXIgcmVpY2gh?=not encoded";
        let res = decode(input);

        assert_eq!(input, res);
    }

    #[test]
    fn decode_utf8_quoted_printable() {
        let input = "=?utf-8?Q?Endlich_was_extrem_hartes_f=C3=BCr_Sie.?=";
        let res = decode(input);
        let comp = "Endlich was extrem hartes für Sie.";

        assert_eq!(comp, res);
    }
}
