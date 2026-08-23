pub trait CanSendEmail {
    fn send_email(&self, to: &str, body: &str);
}

pub trait HasSmtpConfig {}
pub trait HasRecordedEmails {}

impl<T: HasSmtpConfig>     CanSendEmail for T { fn send_email(&self, to: &str, body: &str) {} }
impl<T: HasRecordedEmails> CanSendEmail for T { fn send_email(&self, to: &str, body: &str) {} }   // error[E0119]

fn main() {}
