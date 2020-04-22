use std::fmt;
use std::panic::RefUnwindSafe;

pub use test_wrapper_code_gen::test_;

pub struct Test {
    pub name: &'static str,
    pub line: u32,
    pub file: &'static str,
    pub handler: Box<dyn Fn() + RefUnwindSafe>,
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
    let mut passed_tests = Vec::new();
    let mut failed_tests = Vec::new();

    for test in inventory::iter::<Test> {
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
        panic!(
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
