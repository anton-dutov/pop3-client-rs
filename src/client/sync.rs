use super::*;

use std::io::BufRead;
use std::io::{BufReader, Write};
use std::net::TcpStream;



use bytes::{Bytes, BytesMut, Buf, BufMut};

/// The key structure for the crate, delineating capabilities of the POP3 client as per the protocol [RFC]
///
/// # Errors and problems
/// **All** the methods this `Client` has are susceptible to errors. The common reasons for those are:
/// - Not possible to establish connection
/// - The server does not support the protocol
/// - Connection aborted
/// - Some data got lost or modified, and now it's not possible to decode the obtained message
/// - The server does not recognize the command. This might happen even if by [RFC], the command is mandatory, as most of the servers do not follow the protocol letter by letter
/// - The command was sent on the wrong stage. In other words, you tried to do something before you authorized.
/// - The server returned an error response. We'll look at those within each separate method
///
/// To find out more, read the output of the error you've got -- it's always a string!
///
/// [RFC]: https://tools.ietf.org/html/rfc1081
pub struct SyncClient {
    #[cfg(feature = "with-rustls")]
    client: BufReader<StreamOwned<ClientSession, TcpStream>>,
    #[cfg(not(feature = "with-rustls"))]
    client: BufReader<TcpStream>,
    authorized: bool,
}

impl SyncClient {
    /// ```no_run
    /// # use std::result::Result;
    /// # use pop3_client::AsyncClient;
    /// #
    /// # fn main() -> Result<(), String> {
    ///let client = AsyncClient::connect("pop3.mailtrap.io", 1100)?;
    ///
    /// #    Ok(())
    /// # }
    /// ```
    ///
    /// [`ClientConfig`]: https://docs.rs/rustls/0.15.2/rustls/struct.ClientConfig.html
    pub fn connect(host: &str, port: u16) -> Result<Self> {
        let mut client = TcpStream::connect((host, port))
            .map(|client| Self {
                client: BufReader::new(client),
                authorized: false,
            })
            .map_err(Pop3Error::Io)?;

        client.read_response(false)?;

        Ok(client)
    }

    /// Authorization through plaintext login and password
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use std::result::Result;
    /// # use pop3_client::AsyncClient;
    /// #
    /// # fn main() -> Result<(), String> {
    /// # let mut client = AsyncClient::connect("pop3.mailtrap.io", 1100)?;
    /// client.login("sweet_username", "very_secret_password")?;
    /// #    Ok(())
    /// # }
    /// ```
    /// # Errors
    /// The server may return an error response if:
    /// - the username was not found
    /// - the password does not match the username
    /// - the connection to this mailbox has been locked by another device -- so you won't be able to connect until the lock is released.
    pub fn login(&mut self, username: &str, password: &str) -> Result<()> {
        if self.authorized {
            return Err(Pop3Error::AlreadyAuthenticated);
        }

        self.request(&Command::User { data: username })?;
        self.request(&Command::Pass { data: password })

            .map(|_| {
                self.authorized = true;
                ()
            })
    }

    /// End the session, consuming the client
    ///
    /// # Example
    ///
    /// ```compile_fail
    /// # use std::result::Result;
    /// #
    /// # use pop3_client::AsyncClient;
    /// #
    /// # fn main() -> Result<(), String> {
    /// # let mut client = AsyncClient::connect("pop3.mailtrap.io", 1100)?;
    ///client.quit()?;
    ///client.noop()?; // Shouldn't compile, as the client has been consumed upon quitting
    /// #    Ok(())
    /// # }
    /// ```
    pub fn quit(mut self) -> Result<()> {
        self.request(&Command::Quit)

            .map(|_| ())
    }

    /// Display the statistics for the mailbox (that's what the `STAT` command does).
    ///
    /// In the resulting u32 tuple, the first number is the number of messages, and the second one is number of octets in those messages.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use std::result::Result;
    /// #
    /// # use pop3_client::AsyncClient;
    /// #
    /// # fn main() -> Result<(), String> {
    /// # let mut client = AsyncClient::connect("pop3.mailtrap.io", 1100)?;
    /// let (messages, octets) = client.stat()?;
    /// assert_eq!(messages, 2);
    /// assert_eq!(octets, 340);
    /// #    Ok(())
    /// # }
    /// ```
    pub fn stat(&mut self) -> Result<(u64, u64)> {

        let stat = self.request(&Command::Stat)
            .and_then(|r| r.to_string())?;

        let mut s = stat
            .trim()
            .split(' ')
            .map(|i| i.parse::<u64>().map_err(Pop3Error::InvalidNumber));

        Ok((
            s.next().ok_or(Pop3Error::InvalidResponse)??,
            s.next().ok_or(Pop3Error::InvalidResponse)??,
        ))
    }

    /// Show the statistical information on a chosen letter, or all letters. The information in question always required to start with the letter size, but use of additional stats is not regimented in any way.
    ///
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use std::result::Result;
    /// #
    /// # use pop3_client::AsyncClient;
    /// #
    /// # fn main() -> Result<(), String> {
    /// # let mut client = AsyncClient::connect("pop3.mailtrap.io", 1100)?;
    /// let single_stats = client.list(Some(1))?; // show info on the letter number 1
    /// let all_stats = client.list(None)?; // show info on all letters
    ///
    /// #    Ok(())
    /// # }
    /// ```
    /// # Errors
    /// The server may return an error response if:
    /// - The letter under the given index does not exist in the mailbox
    /// - The letter under the given index has been marked deleted
    pub fn list(&mut self, id: Option<u64>) -> Result<Response> {
        self.request(&Command::List { id })
    }

    /// Show the full content of the chosen message
    ///
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use std::result::Result;
    /// #
    /// # use pop3_client::AsyncClient;
    /// #
    /// # fn main() -> Result<(), String> {
    /// # let mut client = AsyncClient::connect("pop3.mailtrap.io", 1100)?;
    /// let letter_content = client.retr(5)?;
    ///
    /// #    Ok(())
    /// # }
    /// ```
    /// # Errors
    /// The server may return an error response if:
    /// - The letter under the given index does not exist in the mailbox
    /// - The letter under the given index has been marked deleted
    pub fn retr(&mut self, id: u64) -> Result<Bytes> {
        self.request(&Command::Retr { id })

            .map(|s| {
                let tmp = join_bytes(
                    &s.raw()[..]
                        .split(|&b| b == b'\n')
                        .skip(1)
                        .collect::<Vec<&[u8]>>(),
                    b'\n'
                );

                Bytes::copy_from_slice(&tmp)
            })
    }


    /// Mark the chosen message as deleted
    ///
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use std::result::Result;
    /// #
    /// # use pop3_client::AsyncClient;
    /// #
    /// # fn main() -> Result<(), String> {
    /// # let mut client = AsyncClient::connect("pop3.mailtrap.io", 1100)?;
    /// client.dele(3)?; // now, the THIRD message is marked as deleted, and no new manipulations on it are possible
    ///
    /// #    Ok(())
    /// # }
    /// ```
    /// # Errors
    /// The server may return an error response if:
    /// - The letter under the given index does not exist in the mailbox
    /// - The letter under the given index has been marked deleted
    pub fn dele(&mut self, id: u64) -> Result<Response> {
        self.request(&Command::Dele { id })
    }


    /// Do nothing and return a positive response
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use std::result::Result;
    /// #
    /// # use pop3_client::AsyncClient;
    /// #
    /// # fn main() -> Result<(), String> {
    /// # let mut client = AsyncClient::connect("pop3.mailtrap.io", 1100)?;
    /// assert!(client.noop().is_ok());
    ///
    /// #    Ok(())
    /// # }
    /// ```
    pub fn noop(&mut self) -> Result<()> {
        self.request(&Command::Noop)

            .map(|_| ())
    }

    /// Reset the session state, unmarking the items marked as deleted
    ///
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use std::result::Result;
    /// #
    /// # use pop3_client::AsyncClient;
    /// #
    /// # fn main() -> Result<(), String> {
    /// # let mut client = AsyncClient::connect("pop3.mailtrap.io", 1100)?;
    /// client.dele(3)?;
    /// client.dele(4)?;
    /// client.rset()?; // undo all the previous deletions
    /// #    Ok(())
    /// # }
    /// ```
    pub fn rset(&mut self) -> Result<Response> {
        self.request(&Command::Rset)
    }

    /// Show top n lines of a chosen message
    ///
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use std::result::Result;
    /// #
    /// # use pop3_client::AsyncClient;
    /// #
    /// # fn main() -> Result<(), String> {
    /// # let mut client = AsyncClient::connect("pop3.mailtrap.io", 1100)?;
    /// let top = client.top(1, 2)?; // Get TWO first lines of the FIRST message
    ///
    /// #    Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    /// The server may return an error response if:
    /// - The letter under the given index does not exist in the mailbox
    /// - The letter under the given index has been marked deleted
    pub fn top(&mut self, id: u64, lines: u64) -> Result<Response> {
        self.request(&Command::Top { id, lines })
    }

    /// Show the unique ID listing for the chosen message or for all the messages. Unlike message numbering, this ID does not change between sessions.
    ///
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use std::result::Result;
    /// #
    /// # use pop3_client::AsyncClient;
    /// #
    /// # fn main() -> Result<(), String> {
    /// # let mut client = AsyncClient::connect("pop3.mailtrap.io", 1100)?;
    /// let uidl_all = client.uidl(None)?;
    /// let uidl_one = client.uidl(Some(1))?;
    ///
    /// #    Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    /// The server may return an error response if:
    /// - The letter under the given index does not exist in the mailbox
    /// - The letter under the given index has been marked deleted
    pub fn uidl(&mut self, id: Option<u64>) -> Result<Response> {
        self.request(&Command::Uidl { id })
    }

    /// Authorise using the APOP method
    ///
    /// Refer to the POP3 [RFC] for details.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use std::result::Result;
    /// #
    /// # use pop3_client::AsyncClient;
    /// #
    /// # fn main() -> Result<(), String> {
    /// # let mut client = AsyncClient::connect("pop3.mailtrap.io", 1100)?;
    /// client.apop("another_sweet_username", "c4c9334bac560ecc979e58001b3e22fb")?;
    ///
    /// #    Ok(())
    /// # }
    /// ```
    /// # Errors
    /// The server will return error if permission was denied.
    ///
    /// [RFC]: https://tools.ietf.org/html/rfc1081
    pub fn apop(&mut self, id: &str, token: &str) -> Result<Response> {
        if self.authorized {
            return Err(Pop3Error::AlreadyAuthenticated);
        }
        self.request(&Command::Apop { id, token })
            .map(|s| {
                self.authorized = true;
                s
            })
    }

    #[cfg(feature = "with-rustls")]
    fn connect_rustls(host: &str, port: u16, config: Arc<ClientConfig>) -> Result<Self> {
        let hostname = DNSNameRef::try_from_ascii_str(host).map_err(|_| "DNS_NAMEREF_FAILED")?;

        let session = ClientSession::new(&config, hostname);
        let socket = TcpStream::connect((host, port))
            .map(BufReader::new)
            .map_err(Pop3Error::Io)
            .and_then(|mut client| {
                let mut buf = String::new();
                client
                    .read_line(&mut buf)
                    .map_err(|e| e.to_string())
                    .and_then(|_| {
                        if buf.starts_with("+OK") {
                            Ok(buf[4..].to_owned())
                        } else {
                            Err(buf[5..].to_owned())
                        }
                    })
                    .map(|_| client)
            })
            .and_then(|mut client| {
                client
                    .get_mut()
                    .write_all("STLS\r\n".as_bytes())
                    .map_err(|e| e.to_string())
                    .and_then(|_| {
                        let mut buf = String::new();
                        client
                            .read_line(&mut buf)
                            .map_err(|e| e.to_string())
                            .and_then(|_| {
                                println!("STLS: {}", &buf);
                                if buf.starts_with("+OK") {
                                    Ok(buf[4..].to_owned())
                                } else {
                                    Err(buf[5..].to_owned())
                                }
                            })
                    })
                    .map(|_| client.into_inner())
            })?;

        let tls_stream = StreamOwned::new(session, socket);

        Ok(Self {
            client: BufReader::new(tls_stream),
            authorized: false,
        })
    }

    fn read_response(&mut self, multiline: bool) -> Result<Response> {
        let mut response = BytesMut::new();
        let mut buffer   = vec![];

        let amount = self.client
            .read_until(b'\n', &mut buffer)
            .map_err(Pop3Error::Io)?;

        if amount == 0 {
            return Err(Pop3Error::ConnectionClosed)
        }

        if buffer.starts_with(b"+OK") {
            response.put(&buffer[4..]);
        } else {
            let error_msg = std::str::from_utf8(
                if buffer.len() < 6 { &buffer } else { &buffer[5..] },
            );

            let err = match error_msg {
                Ok(v)  => Pop3Error::other(v),
                Err(e) => Pop3Error::InvalidString(e),
            };

            return Err(err)
        }

        if multiline {
            loop {
                buffer.clear();

                let amount = self.client
                    .read_until(b'\n', &mut buffer)
                    .map_err(Pop3Error::Io)?;

                if amount == 0 {
                    return Err(Pop3Error::ConnectionClosed)
                }

                if buffer == b".\r\n" {
                    break;
                }

                response.put(&buffer[..]);
            }
        }

        Ok(Response::new(response.freeze()))
    }

    fn request(&mut self, cmd: &Command<'_>) -> Result<Response> {
        self.client
            .get_mut()
            .write_all(cmd.to_request().as_bytes())
            .map_err(Pop3Error::Io)?;

        self.read_response(cmd.is_response_multiline())

    }
}