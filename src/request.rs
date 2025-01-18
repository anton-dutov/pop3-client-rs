#[derive(Debug, Eq, PartialEq)]
pub enum Command<'a> {
    Apop { id: &'a str, token: &'a str },
    Auth,
    Noop,
    Uidl { id: Option<u64>},
    Top  { id: u64, lines: u64 },
    Dele { id: u64 },
    Retr { id: u64},
    Rset,
    List { id: Option<u64>},
    Stat,
    User { data: &'a str },
    Pass { data: &'a str },
    Quit,
    Capa,
    Greet,
}

impl<'a> Command <'a> {
    pub fn is_response_multiline(&self) -> bool {
        match self {
            Self::Top   { .. } => true,
            Self::Retr  { .. } => true,
            Self::List  { id } => id.is_none(),
            Self::Uidl  { id } => id.is_none(),
            _ => {
                false
            }
        }
    }

    pub fn to_request(&self) -> String {
        match self {
            Self::Apop { id, token } => format!("APOP {id} {token}\r\n"),
            Self::Auth               => "".into(),
            Self::Capa               => "CAPA".into(),
            Self::Greet => "".into(),
            Self::User { data }      => format!("USER {data}\r\n"),
            Self::Pass { data }      => format!("PASS {data}\r\n"),
            Self::Noop               => "NOOP\r\n".into(),
            Self::Top  { id, lines } => format!("TOP {id} {lines}\r\n"),
            Self::Dele { id }        => format!("DELE {id}\r\n"),
            Self::Retr { id }        => format!("RETR {id}\r\n"),
            Self::Rset               => "RSET\r\n".into(),
            Self::List { id }        => if let Some(v) = id {format!("LIST {v}\r\n")} else {"LIST\r\n".into()},
            Self::Stat               => "STAT\r\n".into(),
            Self::Uidl { id }        => if let Some(v) = id {format!("UIDL {v}\r\n")} else {"UIDL\r\n".into()},
            Self::Quit               => "QUIT\r\n".into(),
        }
    }
}

