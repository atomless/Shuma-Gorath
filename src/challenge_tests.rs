// src/challenge_tests.rs
// Unit tests for the challenge (math) logic

#[cfg(test)]
mod tests {
    use super::super::challenge::*;
    use std::collections::HashMap;
    use spin_sdk::http::{Request, Method};

    // Simple in-memory mock store for testing
    use std::cell::RefCell;
    #[derive(Default)]
    struct TestStore {
        map: RefCell<HashMap<String, Vec<u8>>>,
    }
    impl super::super::challenge::KeyValueStore for TestStore {
        fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ()> {
            Ok(self.map.borrow().get(key).cloned())
        }
        fn set(&self, key: &str, value: &[u8]) -> Result<(), ()> {
            self.map.borrow_mut().insert(key.to_string(), value.to_vec());
            Ok(())
        }
        fn delete(&self, key: &str) -> Result<(), ()> {
            self.map.borrow_mut().remove(key);
            Ok(())
        }
    }

    #[test]
    fn test_serve_challenge_stores_answer() {
        let store = TestStore::default();
        let ip = "1.2.3.4";
        let resp = serve_challenge(&store, ip);
        assert_eq!(*resp.status(), 200u16);
        // Should have stored an answer for this IP
        let key = format!("challenge:{}", ip);
        let val = store.get(&key).unwrap().expect("Challenge answer should be stored");
        let stored = String::from_utf8(val).unwrap();
        let mut parts = stored.split(':');
        let answer_str = parts.next().unwrap();
        let qtype = parts.next().unwrap_or("");
        let answer: u32 = answer_str.parse().unwrap();
        match qtype {
            "add" => assert!(answer >= 20 && answer <= 198, "add: {} not in 20..=198", answer),
            "sub" => assert!(answer <= 89, "sub: {} not in 0..=89", answer),
            "mul" => assert!(answer >= 4 && answer <= 144, "mul: {} not in 4..=144", answer),
            _ => panic!("Unknown challenge type: {}", qtype),
        }
    }

    #[test]
    fn test_handle_challenge_submit_correct_and_incorrect() {
        let store = TestStore::default();
        let ip = "1.2.3.4";
        // Test all challenge formats: add, sub, mul
        let cases = vec![
            ("75:sub", "75"),
            ("33:add", "33"),
            ("144:mul", "144"),
            ("007:add", "7"), // leading zero
        ];
        for (stored, submitted) in &cases {
            let key = format!("challenge:{}", ip);
            store.set(&key, stored.as_bytes()).unwrap();
            let body = format!("answer={}&ip={}", submitted, ip);
            let req = Request::builder()
                .method(Method::Post)
                .uri("/challenge")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(body.as_bytes().to_vec())
                .build();
            let resp = handle_challenge_submit(&store, &req);
            assert_eq!(*resp.status(), 200u16, "Should accept correct answer for stored={:?} submitted={:?}", stored, submitted);
            assert!(resp.body().windows(b"Thank you!".len()).any(|w| w == b"Thank you!"));
            // Should have deleted the challenge key
            assert!(store.get(&key).unwrap().is_none());
        }
        // Incorrect answer
        let key = format!("challenge:{}", ip);
        store.set(&key, b"42:add").unwrap();
        let body = format!("answer=99&ip={}", ip);
        let req = Request::builder()
            .method(Method::Post)
            .uri("/challenge")
            .header("content-type", "application/x-www-form-urlencoded")
            .body(body.as_bytes().to_vec())
            .build();
        let resp = handle_challenge_submit(&store, &req);
        assert_eq!(*resp.status(), 403u16);
        assert!(resp.body().windows(b"Incorrect answer".len()).any(|w| w == b"Incorrect answer"));
    }
}
