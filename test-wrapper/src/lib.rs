use std::env::var;
use std::fmt;
use std::panic::RefUnwindSafe;
use std::sync::Arc;

pub use test_wrapper_code_gen::test_;

#[derive(Clone)]
pub struct Test {
    pub name: &'static str,
    pub line: u32,
    pub file: &'static str,
    pub handler: Arc<Box<dyn Fn() + RefUnwindSafe>>,
}

impl Test {
    fn run(&self) -> TestOutput {
        let result = std::panic::catch_unwind(|| {
            (self.handler)();
        });

        match result {
            Ok(()) => {
                println!("{} OK", self.name);
                TestOutput::Pass
            }
            Err(_) => {
                println!("{} failed", self.name);
                TestOutput::Fail
            }
        }
    }
}

impl fmt::Debug for Test {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Test")
            .field("name", &self.name)
            .field("line", &self.line)
            .field("file", &self.file)
            .field("handler", &"{closure}")
            .finish()
    }
}

#[derive(Debug)]
enum TestOutput {
    Pass,
    Fail,
}

inventory::collect!(Test);

pub fn run_all_tests() {
    let mut all_tests = gather_all_tests();

    match (var("TEST_WRAPPER_FILE"), var("TEST_WRAPPER_LINE")) {
        (Err(_), Ok(_)) => {
            panic!("You must also set TEST_WRAPPER_FILE when using TEST_WRAPPER_LINE")
        }
        _ => {}
    }

    if let Ok(file) = var("TEST_WRAPPER_FILE") {
        all_tests = all_tests
            .into_iter()
            .filter(|test| test.file == file)
            .collect();

        if let Ok(line) = var("TEST_WRAPPER_LINE") {
            all_tests = find_test_closest_to_line(
                all_tests,
                line.parse()
                    .expect("expected TEST_WRAPPER_LINE to be a number"),
            )
            .into_iter()
            .collect();
        }
    }

    run_tests_and_print_output(all_tests);
}

fn gather_all_tests() -> Vec<Test> {
    let mut all_tests = Vec::new();
    for test in inventory::iter::<Test> {
        all_tests.push(test.clone());
    }
    all_tests
}

fn run_tests_and_print_output(tests: Vec<Test>) {
    let mut passed_tests = Vec::new();
    let mut failed_tests = Vec::new();

    for test in tests {
        let output = test.run();
        match output {
            TestOutput::Pass => {
                passed_tests.push(test);
            }
            TestOutput::Fail => {
                failed_tests.push(test);
            }
        }
    }

    if !failed_tests.is_empty() {
        println!(
            "{} passed, {} failures",
            passed_tests.len(),
            failed_tests.len()
        );
        std::process::exit(1);
    } else {
        println!(
            "{} passed, {} failures",
            passed_tests.len(),
            failed_tests.len()
        );
    }
}

#[macro_export]
macro_rules! register_test {
    ( $($t:tt)* ) => {
        inventory::submit! {
            $($t)*
        }
    }
}

#[macro_export]
macro_rules! setup {
    () => {
        #[test]
        fn test_wrapper_run_all_tests() {
            $crate::run_all_tests();
        }
    };
}

fn find_test_closest_to_line(tests: Vec<Test>, line: u32) -> Option<Test> {
    tests
        .into_iter()
        .filter(|test| test.line <= line)
        .min_by_key(|test| line - test.line)
}

#[cfg(test)]
mod test {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn no_tests() {
        let all_tests = vec![];

        let test_found = find_test_closest_to_line(all_tests, 1);

        assert!(test_found.is_none());
    }

    #[test]
    fn only_one_test_exact_match() {
        let all_tests = vec![Test {
            name: "foo",
            line: 1,
            file: "bar",
            handler: Arc::new(Box::new(|| {})),
        }];

        let test_found = find_test_closest_to_line(all_tests.clone(), 1);

        assert_eq!(test_found.unwrap().name, all_tests[0].name);
    }

    #[test]
    fn only_one_test_not_exact_match() {
        let all_tests = vec![Test {
            name: "foo",
            line: 1,
            file: "bar",
            handler: Arc::new(Box::new(|| {})),
        }];

        let test_found = find_test_closest_to_line(all_tests.clone(), 10);

        assert_eq!(test_found.unwrap().name, all_tests[0].name);
    }

    #[test]
    fn more_than_one() {
        let all_tests = vec![
            Test {
                name: "foo",
                line: 1,
                file: "bar",
                handler: Arc::new(Box::new(|| {})),
            },
            Test {
                name: "bar",
                line: 10,
                file: "bar",
                handler: Arc::new(Box::new(|| {})),
            },
        ];

        let test_found = find_test_closest_to_line(all_tests, 10);

        assert_eq!(test_found.unwrap().name, "bar");
    }

    #[test]
    fn line_zero() {
        let all_tests = vec![Test {
            name: "foo",
            line: 1,
            file: "bar",
            handler: Arc::new(Box::new(|| {})),
        }];

        let test_found = find_test_closest_to_line(all_tests, 0);

        assert!(test_found.is_none());
    }

    #[test]
    fn very_high_line_number() {
        let all_tests = vec![Test {
            name: "foo",
            line: 1,
            file: "bar",
            handler: Arc::new(Box::new(|| {})),
        }];

        let test_found = find_test_closest_to_line(all_tests, 1337);

        assert_eq!(test_found.unwrap().name, "foo");
    }
}
