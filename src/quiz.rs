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
use rand::prelude::*;

const QUIZ_PREFIX: &str = "quiz:";
const QUESTION_TYPES: &[&str] = &["add", "sub", "mul"];

/// Generates a random math challenge (add, sub, mul) and stores the answer and question type in KV for the IP.
pub fn serve_quiz<S: KeyValueStore>(store: &S, ip: &str) -> Response {
    let mut rng = rand::rng();
    let a: u32 = rng.random_range(10..=99);
    let b: u32 = rng.random_range(10..=99);
    let qtype = *QUESTION_TYPES.choose(&mut rng).unwrap_or(&"add");
    let (question, answer) = match qtype {
        "add" => (format!("{a} + {b}"), a + b),
        "sub" => {
            let (x, y) = if a > b { (a, b) } else { (b, a) };
            (format!("{x} - {y}"), x - y)
        },
        "mul" => {
            let a = rng.random_range(2..=12);
            let b = rng.random_range(2..=12);
            (format!("{a} × {b}"), a * b)
        },
        _ => (format!("{a} + {b}"), a + b),
    };
    let key = format!("{}{}", QUIZ_PREFIX, ip);
    let value = format!("{}:{}", answer, qtype);
    let _ = store.set(&key, value.as_bytes());
    let html = format!(r#"
        <html><head><style>
        body {{ font-family: sans-serif; background: #f9f9f9; margin: 2em; }}
        .quiz-container {{ background: #fff; padding: 2em; border-radius: 8px; box-shadow: 0 2px 8px #ccc; max-width: 400px; margin: auto; }}
        label {{ font-size: 1.2em; }}
        input[type=number] {{ font-size: 1.2em; width: 80px; }}
        button {{ font-size: 1em; padding: 0.5em 1em; }}
        </style></head><body>
        <div class="quiz-container">
        <h2>Are you human?</h2>
        <form method='POST' action='/quiz'>
            <label>Solve: {question} = </label>
            <input name='answer' type='number' required autofocus />
            <input type='hidden' name='ip' value='{ip}' />
            <button type='submit'>Submit</button>
        </form>
        <p style="color: #888; font-size: 0.9em;">Prove you are not a bot to regain access.</p>
        </div>
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
            if let Ok(stored) = String::from_utf8(val) {
                let mut parts = stored.splitn(2, ':');
                if let (Some(expected), _) = (parts.next(), parts.next()) {
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
    }
    let html = "<html><body><h2 style='color:red;'>Incorrect answer. Please try again.</h2><a href='/quiz'>Back to quiz</a></body></html>";
    Response::new(403, html)
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
