// src/quiz_tests.rs
// Unit tests for the quiz (math challenge) logic

#[cfg(test)]
mod tests {
    use super::super::quiz::*;
    use std::collections::HashMap;
    use spin_sdk::http::{Request, Method};

    // Simple in-memory mock store for testing
    use std::cell::RefCell;
    #[derive(Default)]
    struct TestStore {
        map: RefCell<HashMap<String, Vec<u8>>>,
    }
    impl super::super::quiz::KeyValueStore for TestStore {
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
    fn test_serve_quiz_stores_answer() {
        let store = TestStore::default();
        let ip = "1.2.3.4";
        let resp = serve_quiz(&store, ip);
        assert_eq!(*resp.status(), 200u16);
        // Should have stored an answer for this IP
        let key = format!("quiz:{}", ip);
        let val = store.get(&key).unwrap().expect("Quiz answer should be stored");
        let stored = String::from_utf8(val).unwrap();
        let answer_str = stored.split(':').next().unwrap();
        let answer: u32 = answer_str.parse().unwrap();
        assert!(answer >= 4 && answer <= 198); // 2*2 to 99+99
    }

    #[test]
    fn test_handle_quiz_submit_correct_and_incorrect() {
        let store = TestStore::default();
        let ip = "1.2.3.4";
        let answer = 42;
        let key = format!("quiz:{}", ip);
        store.set(&key, answer.to_string().as_bytes()).unwrap();
        // Correct answer
        let body = format!("answer=42&ip={}", ip);
        let req = Request::builder()
            .method(Method::Post)
            .uri("/quiz")
            .header("content-type", "application/x-www-form-urlencoded")
            .body(body.as_bytes().to_vec())
            .build();
        let resp = handle_quiz_submit(&store, &req);
        assert_eq!(*resp.status(), 200u16);
        assert!(resp.body().windows(b"Thank you!".len()).any(|w| w == b"Thank you!"));
        // Should have deleted the quiz key
        assert!(store.get(&key).unwrap().is_none());
        // Incorrect answer
        store.set(&key, answer.to_string().as_bytes()).unwrap();
        let body = format!("answer=99&ip={}", ip);
        let req = Request::builder()
            .method(Method::Post)
            .uri("/quiz")
            .header("content-type", "application/x-www-form-urlencoded")
            .body(body.as_bytes().to_vec())
            .build();
        let resp = handle_quiz_submit(&store, &req);
        assert_eq!(*resp.status(), 403u16);
        assert!(resp.body().windows(b"Incorrect answer".len()).any(|w| w == b"Incorrect answer"));
    }
}
