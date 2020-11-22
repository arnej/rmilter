use std::convert::{TryFrom, TryInto};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};

use crate::message_handler::MessageHandler;
use crate::milter_error::MilterError;
use crate::milter_message::{MilterMessage, MilterProtocol, ResponseMessage};

/// This is the main struct that opens the milter connection.
///
/// Also holds the `MessageHandler`.
pub struct Milter<'a> {
    message_handler: &'a mut dyn MessageHandler,
    protocol: Option<MilterProtocol>,
}

impl<'a> Milter<'a> {
    fn handle_message(&mut self, s: &mut TcpStream, buffer: &[u8]) -> Result<bool, MilterError> {
        let mut keep_open = true;

        match MilterMessage::try_from(buffer) {
            Ok(message) => {
                match message {
                    MilterMessage::AbortFilterChecks => self.message_handler.abort_filter_checks(),
                    MilterMessage::BodyChunk { value } => {
                        let action = self.message_handler.body_chunk(&value);
                        self.send_response(s, action)?;
                    }
                    MilterMessage::ConnectionInformation {
                        hostname,
                        family,
                        port,
                        address,
                    } => {
                        let action = self
                            .message_handler
                            .connection(&hostname, &family, &port, &address);
                        self.send_response(s, action)?;
                    }
                    MilterMessage::DefineMacros { cmdcode, macros } => {
                        self.message_handler.define_macros(&cmdcode, macros);
                    }
                    MilterMessage::EndOfBody => {
                        let action = self.message_handler.end_of_body();
                        self.send_response(s, action)?;
                    }
                    MilterMessage::EndOfHeader => {
                        let action = self.message_handler.end_of_header();
                        self.send_response(s, action)?;
                    }
                    MilterMessage::Header { name, value } => {
                        let action = self.message_handler.header(&name, &value);
                        self.send_response(s, action)?;
                    }
                    MilterMessage::Helo { msg } => {
                        let action = self.message_handler.helo(&msg);
                        self.send_response(s, action)?;
                    }
                    MilterMessage::MailFrom { sender, args } => {
                        let action = self.message_handler.mail_from(&sender, &args);
                        self.send_response(s, action)?;
                    }
                    MilterMessage::OptionNegotiation {
                        version,
                        actions,
                        protocol: _,
                    } => {
                        let response_msg = ResponseMessage::option_negotiation(
                            version,
                            actions,
                            self.protocol.as_ref().unwrap_or(&MilterProtocol::default()),
                        );

                        self.send_response(s, response_msg)?;
                    }
                    MilterMessage::QuitCommunication => {
                        keep_open = false;
                    }
                    MilterMessage::RecipientInformation { recipient, args } => {
                        let action = self.message_handler.recipient(&recipient, &args);
                        self.send_response(s, action)?;
                    }
                };
            }
            Err(_e) => {
                let mut response = Vec::with_capacity(5);
                response.append(&mut u32::to_be_bytes(1).to_vec());
                response.push(b'c');

                s.write_all(&response)?;
            }
        }

        Ok(keep_open)
    }

    fn handle_stream(&mut self, mut stream: TcpStream) -> Result<(), MilterError> {
        let u32_size = std::mem::size_of::<u32>();
        let mut buffer = [0; 128];
        let mut collected_bytes = Vec::new();

        loop {
            let mut keep_open = true;

            match stream.read(&mut buffer) {
                Ok(0) => {
                    println!("Closing connection");
                    break;
                }
                Ok(len) => {
                    // First, add everything read to collected_bytes
                    collected_bytes.extend_from_slice(&buffer[..len]);

                    if collected_bytes.len() >= u32_size {
                        let mut msg_len: usize =
                            u32::from_be_bytes(collected_bytes[..u32_size].try_into()?)
                                .try_into()?;

                        while collected_bytes.len() >= u32_size + msg_len {
                            // Only remove first 4 bytes when the complete message is available
                            collected_bytes.drain(..u32_size);
                            let msg: Vec<u8> = collected_bytes.drain(..msg_len).collect();

                            if !self.handle_message(&mut stream, &msg)? {
                                keep_open = false;
                                break;
                            }

                            if collected_bytes.len() >= std::mem::size_of::<u32>() {
                                msg_len =
                                    u32::from_be_bytes(collected_bytes[..u32_size].try_into()?)
                                        .try_into()?;
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error while receiving data: {}", e);
                    break;
                }
            }

            if !keep_open {
                break;
            }
        }
        Ok(())
    }

    pub(crate) fn new(
        message_handler: &'a mut dyn MessageHandler,
        protocol: Option<MilterProtocol>,
    ) -> Self {
        Self {
            message_handler,
            protocol,
        }
    }

    /// Opens the connection to the MTA service.
    ///
    /// - `address` defines the socket address of the MTA.
    pub fn run<S: ToSocketAddrs>(&'a mut self, address: S) -> Result<(), MilterError> {
        let listener = TcpListener::bind(address)?;

        for stream in listener.incoming() {
            self.handle_stream(stream?)?;
        }

        Ok(())
    }

    fn send_response<R: Into<ResponseMessage>>(
        &self,
        s: &mut TcpStream,
        response_msg: R,
    ) -> Result<(), MilterError> {
        let response_msg = response_msg.into();
        let response = response_msg.get_content();

        s.write_all(&response)?;
        s.flush()?;

        Ok(())
    }
}
