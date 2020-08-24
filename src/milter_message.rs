use std::convert::{TryFrom, TryInto};

use crate::accept_reject_action::AcceptRejectAction;
use crate::milter_error::MilterError;

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
                actions: MilterActions::from(&rest[4..=7].try_into()?),
                protocol: MilterProtocol::from(&rest[8..=11].try_into()?),
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

#[derive(Debug)]
pub struct MilterMacro {
    name: String,
    value: String,
}

#[derive(Debug)]
pub enum ProtocolFamily {
    UnixSocket,
    Inet4,
    Inet6,
}

#[derive(Debug, PartialEq)]
pub(crate) struct MilterActions {
    add_headers: bool,
    change_body: bool,
    add_recipients: bool,
    remove_recipients: bool,
    change_headers: bool,
    quarantine: bool,
}

impl From<&[u8; 4]> for MilterActions {
    fn from(val: &[u8; 4]) -> Self {
        let v = u32::from_be_bytes(*val);
        Self {
            add_headers: v & (1 << 0) != 0,
            change_body: v & (1 << 1) != 0,
            add_recipients: v & (1 << 2) != 0,
            remove_recipients: v & (1 << 3) != 0,
            change_headers: v & (1 << 4) != 0,
            quarantine: v & (1 << 5) != 0,
        }
    }
}

impl From<MilterActions> for Vec<u8> {
    fn from(m: MilterActions) -> Self {
        let mut val: u32 = 0;

        val |= u32::from(m.add_headers);
        val |= u32::from(m.change_body) << 1;
        val |= u32::from(m.add_recipients) << 2;
        val |= u32::from(m.remove_recipients) << 3;
        val |= u32::from(m.change_headers) << 4;
        val |= u32::from(m.quarantine) << 5;

        val.to_be_bytes().into()
    }
}

impl MilterActions {
    #[cfg(test)]
    fn new(
        add_headers: bool,
        change_body: bool,
        add_recipients: bool,
        remove_recipients: bool,
        change_headers: bool,
        quarantine: bool,
    ) -> Self {
        MilterActions {
            add_headers,
            change_body,
            add_recipients,
            remove_recipients,
            change_headers,
            quarantine,
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct MilterProtocol {
    no_connect: bool,
    no_helo: bool,
    no_mail: bool,
    no_recipient: bool,
    no_body: bool,
    no_header: bool,
    no_eoh: bool,
}

impl From<&[u8; 4]> for MilterProtocol {
    fn from(val: &[u8; 4]) -> Self {
        let v = u32::from_be_bytes(*val);
        Self {
            no_connect: v & (1 << 0) != 0,
            no_helo: v & (1 << 1) != 0,
            no_mail: v & (1 << 2) != 0,
            no_recipient: v & (1 << 3) != 0,
            no_body: v & (1 << 4) != 0,
            no_header: v & (1 << 5) != 0,
            no_eoh: v & (1 << 6) != 0,
        }
    }
}

impl From<MilterProtocol> for Vec<u8> {
    fn from(m: MilterProtocol) -> Self {
        let mut val: u32 = 0;

        val |= u32::from(m.no_connect);
        val |= u32::from(m.no_helo) << 1;
        val |= u32::from(m.no_mail) << 2;
        val |= u32::from(m.no_recipient) << 3;
        val |= u32::from(m.no_body) << 4;
        val |= u32::from(m.no_header) << 5;
        val |= u32::from(m.no_eoh) << 6;

        val.to_be_bytes().into()
    }
}

impl MilterProtocol {
    pub(crate) fn new(
        no_connect: bool,
        no_helo: bool,
        no_mail: bool,
        no_recipient: bool,
        no_body: bool,
        no_header: bool,
        no_eoh: bool,
    ) -> Self {
        MilterProtocol {
            no_connect,
            no_helo,
            no_mail,
            no_recipient,
            no_body,
            no_header,
            no_eoh,
        }
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
        protocol: MilterProtocol,
    ) -> Self {
        // OPTNEG buffer length is always 17
        let mut buf = Vec::with_capacity(17);

        // OPTNEG length is always 13
        let mut length = u32::to_be_bytes(13).to_vec();

        buf.append(&mut length);
        buf.push(b'O');

        buf.append(&mut version.to_be_bytes().to_vec());
        buf.append(&mut actions.into());
        buf.append(&mut protocol.into());

        Self { content: buf }
    }
}

fn decode<S: AsRef<str>>(s: S) -> String {
    let mut res = String::with_capacity(s.as_ref().len());

    for split in s.as_ref().split("=?") {
        if let Some(encoded_end) = split.find("?=") {
            let encoded = &split[..encoded_end];
            let parts: Vec<&str> = encoded.split('?').collect();

            if parts.len() == 3 {
                if let Some(charset) =
                    charset::Charset::for_label_no_replacement(parts[0].as_bytes())
                {
                    let transfer_encoding = parts[1];
                    let encoded_value = parts[2];

                    let decoded = match transfer_encoding {
                        "b" | "B" => base64::decode(encoded_value)
                            .unwrap_or_else(|_| encoded.as_bytes().to_vec()),
                        "q" | "Q" => quoted_printable::decode(
                            encoded_value.replace("_", " "),
                            quoted_printable::ParseMode::Robust,
                        )
                        .unwrap_or_else(|_| encoded.as_bytes().to_vec()),
                        _ => encoded_value.as_bytes().to_vec(),
                    };

                    let (decoded, _) = charset.decode_without_bom_handling(&decoded);

                    res.push_str(&decoded);
                } else {
                    res.push_str(split);
                }
            }

            // Append rest (if any)
            // println!("Encoded_end: {}, split.len(): {}", encoded_end, split.len());
            if encoded_end + 2 < split.len() {
                res.push_str(&split[encoded_end + 2..]);
            }
        } else {
            res.push_str(split);
        }
    }

    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_milter_actions_add_recipients() {
        let x: [u8; 4] = [0, 0, 0, 4];
        let res = MilterActions::from(&x);
        let comp = MilterActions::new(false, false, true, false, false, false);

        assert_eq!(comp, res);
    }

    #[test]
    fn create_milter_actions_add_headers_and_quarantine() {
        let x: [u8; 4] = [0, 0, 0, 33];
        let res = MilterActions::from(&x);
        let comp = MilterActions::new(true, false, false, false, false, true);

        assert_eq!(comp, res);
    }

    #[test]
    fn create_milter_protocol_no_mail() {
        let x: [u8; 4] = [0, 0, 0, 4];
        let res = MilterProtocol::from(&x);
        let comp = MilterProtocol::new(false, false, true, false, false, false, false);

        assert_eq!(comp, res);
    }

    #[test]
    fn create_milter_protocol_no_body() {
        let x: [u8; 4] = [0, 0, 0, 16];
        let res = MilterProtocol::from(&x);
        let comp = MilterProtocol::new(false, false, false, false, true, false, false);

        assert_eq!(comp, res);
    }

    #[test]
    fn create_milter_protocol_no_connect_and_header() {
        let x: [u8; 4] = [0, 0, 0, 33];
        let res = MilterProtocol::from(&x);
        let comp = MilterProtocol::new(true, false, false, false, false, true, false);

        assert_eq!(comp, res);
    }

    #[test]
    fn decode_utf8_base64() {
        // Taken from an actual spam mail and added 'not encoded' to test that we keep non-encoded
        // data
        let input = "not encoded=?utf-8?B?4oCeSMO2aGxlIGRlciBMw7Z3ZW7igJwgU3lzdGVtIG1hY2h0IERldXRzY2hlIELDvHJnZXIgcmVpY2gh?=not encoded";
        let res = decode(input);
        let comp = "not encoded„Höhle der Löwen“ System macht Deutsche Bürger reich!not encoded";

        assert_eq!(comp, res);
    }

    #[test]
    fn decode_utf8_quoted_printable() {
        let input = "=?utf-8?Q?Endlich_was_extrem_hartes_f=C3=BCr_Sie.?=";
        let res = decode(input);
        let comp = "Endlich was extrem hartes für Sie.";

        assert_eq!(comp, res);
    }
}
