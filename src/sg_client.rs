use errors::SendgridResult;

use mail::{Mail,Destination};

use std::io::Read;

use reqwest::header::{Authorization, Bearer, ContentType, Headers, UserAgent};
use reqwest::Client;

use serde_urlencoded;

static API_URL: &'static str = "https://api.sendgrid.com/api/mail.send.json?";

/// This is the struct that allows you to authenticate to the SendGrid API.
/// It's only field is the API key which allows you to send messages.
pub struct SGClient {
    api_key: String,
}

impl SGClient {
    /// Makes a new SendGrid cient with the specified API key.
    pub fn new(key: String) -> SGClient {
        SGClient { api_key: key }
    }

    /// Sends a messages through the SendGrid API. It takes a Mail struct as an
    /// argument. It returns the string response from the API as JSON.
    /// It sets the Content-Type to be application/x-www-form-urlencoded.
    pub fn send(&self, mail_info: Mail) -> SendgridResult<String> {
        let client = Client::new();
        let mut headers = Headers::new();
        headers.set(Authorization(Bearer {
            token: self.api_key.to_owned(),
        }));
        headers.set(ContentType::form_url_encoded());
        headers.set(UserAgent::new("sendgrid-rs"));

        let post_body = serde_urlencoded::to_string(mail_info)?;
        let mut res = client
            .post(API_URL)
            .headers(headers)
            .body(post_body)
            .send()?;
        let mut body = String::new();
        res.read_to_string(&mut body)?;
        Ok(body)
    }
}

#[test]
fn basic_message_body() {
    let m = Mail::new(Destination { address: "test@example.com", name: "Testy mcTestFace" },
                      "Test",
                      Destination { address: "me@example.com", name: "Example sender" })
        .add_text("It works");

    let body = serde_urlencoded::to_string(m);
    let want = "to%5B%5D=test%40example.com&toname%5B%5D=Testy+mcTestFace&from=me%40example.com&subject=Test&\
                html=&text=It+works&fromname=&replyto=&date=&headers=%7B%7D&x-smtpapi=";
    assert_eq!(body.unwrap(), want);
}
