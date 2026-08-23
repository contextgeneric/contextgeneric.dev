use core::cell::RefCell;

use cgp::prelude::*;

#[cgp_component(EmailSender)]
pub trait CanSendEmail {
    fn send_email(&self, to: &str, body: &str);
}

#[cgp_impl(new RecordEmails)]
impl EmailSender {
    fn send_email(&self, #[implicit] sent_emails: &RefCell<Vec<String>>, to: &str, body: &str) {
        sent_emails.borrow_mut().push(format!("{to}: {body}"));
    }
}

// The mistake: `BrokenApp` has no `sent_emails` field, though `RecordEmails` reads one.
#[derive(HasField)]
pub struct BrokenApp {
    pub smtp_server: String,
}

delegate_components! { BrokenApp { EmailSenderComponent: RecordEmails } }

fn main() {
    let app = BrokenApp {
        smtp_server: "localhost".to_owned(),
    };
    // The hidden failure: `E0599` at the call site, naming nothing about the missing field.
    app.send_email("a@b.c", "hi");
}
