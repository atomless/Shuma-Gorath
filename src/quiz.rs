// src/quiz.rs
// Interactive math challenge for banned users

use spin_sdk::http::{Request, Response};
use spin_sdk::key_value::Store;

pub trait KeyValueStore {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ()>;
    fn set(&self, key: &str, value: &[u8]) -> Result<(), ()>;
    fn delete(&self, key: &str) -> Result<(), ()>;
}

impl KeyValueStore for Store {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ()> {
        Store::get(self, key).map_err(|_| ())
    }
    fn set(&self, key: &str, value: &[u8]) -> Result<(), ()> {
        Store::set(self, key, value).map_err(|_| ())
    }
    fn delete(&self, key: &str) -> Result<(), ()> {
        Store::delete(self, key).map_err(|_| ())
    }
}
use rand::Rng;
use percent_encoding;

const QUIZ_PREFIX: &str = "quiz:";

/// Generates a simple math challenge and stores the answer in KV for the IP.
pub fn serve_quiz<S: KeyValueStore>(store: &S, ip: &str) -> Response {
    let mut rng = rand::rng();
    let a: u32 = rng.random_range(10..=99);
    let b: u32 = rng.random_range(10..=99);
    let answer = a + b;
    let key = format!("{}{}", QUIZ_PREFIX, ip);
    let _ = store.set(&key, answer.to_string().as_bytes());
    let html = format!(r#"
        <html><body>
        <h2>Are you human?</h2>
        <form method='POST' action='/quiz'>
            <label>Solve: {a} + {b} = </label>
            <input name='answer' type='number' required />
            <input type='hidden' name='ip' value='{ip}' />
            <button type='submit'>Submit</button>
        </form>
        </body></html>
    "#);
    Response::new(200, html)
}

/// Validates the quiz answer. If correct, unbans the IP and returns a success page.
pub fn handle_quiz_submit<S: KeyValueStore>(store: &S, req: &Request) -> Response {
    let form = String::from_utf8_lossy(req.body()).to_string();
    let answer = get_form_field(&form, "answer");
    let ip = get_form_field(&form, "ip");
    if let (Some(answer), Some(ip)) = (answer, ip) {
        let key = format!("{}{}", QUIZ_PREFIX, ip);
        if let Ok(Some(val)) = store.get(&key) {
            if let Ok(expected) = String::from_utf8(val) {
                if answer == expected {
                    // Unban the IP
                    let ban_key = format!("ban:default:{}", ip);
                    let _ = store.delete(&ban_key);
                    let _ = store.delete(&key);
                    return Response::new(200, "<html><body><h2>Thank you! You are unbanned. Please reload the page.</h2></body></html>");
                }
            }
        }
    }
    Response::new(403, "<html><body><h2>Incorrect answer. Please try again.</h2><a href='/quiz'>Back to quiz</a></body></html>")
}

fn get_form_field(form: &str, name: &str) -> Option<String> {
    for pair in form.split('&') {
        let mut parts = pair.splitn(2, '=');
        if let (Some(k), Some(v)) = (parts.next(), parts.next()) {
            if k == name {
                return Some(url_decode(v));
            }
        }
    }
    None
}

fn url_decode(s: &str) -> String {
    percent_encoding::percent_decode_str(s).decode_utf8_lossy().to_string()
}
