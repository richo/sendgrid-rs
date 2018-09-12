use errors::{SendgridErrorKind, SendgridResult};

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use serde_json;

macro_rules! add_field {
    // Create a setter that destructures a destination and appends.
    ($method:ident << $field:ident, $fieldname:ident) => {
        pub fn $method(mut self, data: Destination<'a>) -> Mail<'a> {
            let Destination {
                address,
                name,
            } = data;
            self.$field.push(address);
            self.$fieldname.push(name);
            self
        }
    };

    // Create a setter that stores
    ($method:ident = $field:ident: $ty:ty) => {
        pub fn $method(mut self, data: $ty) -> Mail<'a> {
            self.$field = Some(data);
            self
        }
    };

    // Create a setter that inserts into a map
    ($method:ident <- $field:ident: $ty:ty) => {
        pub fn $method(mut self, id: String, data: $ty) -> Mail<'a> {
            self.$field.insert(id, data);
            self
        }
    };
}

#[derive(Debug)]
pub struct Destination<'a> {
    pub address: &'a str,
    pub name: &'a str,
}

#[derive(Debug,Serialize)]
/// This is a representation of a valid SendGrid message. It has support for
/// all of the fields in the V2 API.
pub struct Mail<'a> {
    pub to: Vec<&'a str>,
    pub toname: Vec<&'a str>,
    pub cc: Vec<&'a str>,
    pub ccname: Vec<&'a str>,
    pub bcc: Vec<&'a str>,
    pub bccname: Vec<&'a str>,
    pub from: &'a str,
    pub fromname: &'a str,
    pub subject: &'a str,
    pub html: Option<&'a str>,
    pub text: Option<&'a str>,
    pub replyto: Option<&'a str>,
    pub date: Option<&'a str>,
    pub attachments: HashMap<String, String>,
    pub content: HashMap<String, &'a str>,
    pub headers: HashMap<String, &'a str>,
    pub x_smtpapi: Option<&'a str>,
}

impl<'a> Mail<'a> {
    /// Returns a new Mail struct to send with a client. All of the fields are
    /// initially empty.
    pub fn new(to: Destination<'a>, subject: &'a str, from: Destination<'a>) -> Mail<'a> {
        // We take the bare minimum number of arguments here to avoid having to check them later
        let Destination {
            address: fromaddress,
            name: fromname,
        } = from;

        let Destination {
            address: toaddress,
            name: toname,
        } = to;

        Mail {
            to: vec![toaddress],
            toname: vec![toname],
            cc: Vec::new(),
            ccname: Vec::new(),
            bcc: Vec::new(),
            bccname: Vec::new(),
            from: fromaddress,
            fromname: fromname,
            subject: subject,
            html: None,
            text: None,
            replyto: None,
            date: None,
            attachments: HashMap::new(),
            content: HashMap::new(),
            headers: HashMap::new(),
            x_smtpapi: None,
        }
    }

    /// Adds a CC recipient to the Mail struct.
    add_field!(add_cc << cc, ccname);

    /// Adds a to recipient to the Mail struct.
    add_field!(add_to << to, toname);

    /// Add a BCC address to the message.
    add_field!(add_bcc << bcc, bccname);

    /// This function sets the HTML content for the message.
    add_field!(add_html = html: &'a str);

    /// Set the text content of the message.
    add_field!(add_text = text: &'a str);

    /// Set the reply to address for the message.
    add_field!(add_reply_to = replyto: &'a str);

    /// Set the date for the message. This must be a valid RFC 822 timestamp.
    // TODO(richo) Should this be a chronos::Utc ?
    add_field!(add_date = date: &'a str);

    /// Convenience method when using Mail as a builder
    pub fn build(self) -> Mail<'a> {
        assert!(self.text.is_some() || self.html.is_some(), "Need exactly one of text or html set");
        self
    }

    /// Add an attachment for the message. You can pass the name of a file as a
    /// path on the file system.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let message = Mail::new()
    ///     .add_attachment("/path/to/file/contents.txt");
    /// ```
    pub fn add_attachment<P: AsRef<Path>>(mut self, path: P) -> SendgridResult<Mail<'a>> {
        let mut file = File::open(&path)?;
        let mut data = String::new();
        file.read_to_string(&mut data)?;

        if let Some(name) = path.as_ref().to_str() {
            self.attachments.insert(String::from(name), data);
        } else {
            return Err(SendgridErrorKind::InvalidFilename.into());
        }

        Ok(self)
    }

    /// Add content for inline images in the message.
    add_field!(add_content <- content: &'a str);

    /// Add a custom header for the message. These are usually prefixed with
    /// 'X' or 'x' per the RFC specifications.
    add_field!(add_header <- headers: &'a str);

    /// Used internally for string encoding. Not needed for message building.
    pub(crate) fn make_header_string(&mut self) -> SendgridResult<String> {
        let string = serde_json::to_string(&self.headers)?;
        Ok(string)
    }

    /// Add an X-SMTPAPI string to the message. This can be done by using the
    /// 'rustc_serialize' crate and JSON encoding a map or custom struct. Or
    /// a regular String type can be escaped and used.
    add_field!(add_x_smtpapi = x_smtpapi: &'a str);
}
