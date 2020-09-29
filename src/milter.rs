use std::convert::{TryFrom, TryInto};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};

use crate::message_handler::MessageHandler;
use crate::milter_error::MilterError;
use crate::milter_message::{MilterMessage, MilterProtocol, ResponseMessage};

pub struct Milter<'a> {
    message_handler: &'a mut dyn MessageHandler,
    protocol: Option<MilterProtocol>,
}

impl<'a> Milter<'a> {
    fn handle_message(&mut self, s: &mut TcpStream, buffer: &[u8]) -> Result<bool, MilterError> {
        let mut keep_open = true;

        // print!("Raw bytes: ");
        // for b in buffer.iter() {
        //     print!("{} ", b);
        // }
        // println!();

        match MilterMessage::try_from(buffer) {
            Ok(message) => {
                // println!("Code: {}, Data: {:?}", char::from(buffer[0]), message);

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
                        // TODO: Create OptionNegotiation by checking for configured handlers (or
                        // let the user specify)
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
                // println!("Error parsing message: {}", e);

                // print!("raw bytes: ");
                // for b in buffer.iter() {
                //     print!("{} ", b);
                // }
                // println!();

                let mut response = Vec::with_capacity(5);
                response.append(&mut u32::to_be_bytes(1).to_vec());
                response.push(b'c');

                // println!("Response length: {}", response.len());
                // for b in response.iter() {
                //     print!("{} ", b);
                // }
                // println!();

                s.write_all(&response)?;
            }
        }

        Ok(keep_open)
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

    pub fn run<S: ToSocketAddrs>(&'a mut self, address: S) -> Result<(), MilterError> {
        let listener = TcpListener::bind(address)?;

        for stream in listener.incoming() {
            match stream {
                Ok(mut s) => {
                    let mut keep_open = true;

                    while keep_open {
                        let mut buffer = [0; std::mem::size_of::<u32>()];
                        let mut len = s.peek(&mut buffer)?;

                        // Only start reading when at least the message size (4 bytes) is available
                        if len >= std::mem::size_of::<u32>() {
                            len = s.read(&mut buffer)?;

                            if len > 0 {
                                let msg_len = u32::from_be_bytes(buffer);

                                // println!("Message length: {}", msg_len);

                                let mut buffer = vec![0; msg_len.try_into()?];
                                s.read_exact(&mut buffer)?;

                                keep_open = self.handle_message(&mut s, &buffer)?;
                            }
                        }
                    }
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }

        Ok(())
    }

    fn send_response<R: Into<ResponseMessage>>(
        &self,
        s: &mut TcpStream,
        response_msg: R,
    ) -> Result<(), MilterError> {
        let response_msg = response_msg.into();
        // println!("Sending response: {:?}", response_msg);

        let response = response_msg.get_content();

        // println!("Response length: {}", response.len());
        // for b in response.iter() {
        //     print!("{} ", b);
        // }
        // println!();

        s.write_all(&response)?;
        s.flush()?;

        Ok(())
    }
}
