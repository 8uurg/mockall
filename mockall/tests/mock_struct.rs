// vim: tw=80
//! Structs can be mocked with mock!  This is useful when the struct's original
//! definition is not accessible.
#![deny(warnings)]

use mockall::*;

// A struct with a definition like this:
// struct Foo {
//     _x: i16
// }
// impl Foo {
//     fn foo(&self, _x: u32) -> u32 {
//         42
//     }
// }
// Could be mocked like this:
mock!{
    Foo {
        fn foo(&self, x: u32) -> u32;
        fn bar(&self, x: u32);
        fn baz(&self);
    }
}

mod checkpoint {
    use std::panic;
    use super::*;

    #[test]
    fn expect_again() {
        let mut mock = MockFoo::new();
        mock.expect_foo()
            .returning(|_| 5)
            .times(1..3);
        mock.foo(0);
        mock.checkpoint();

        mock.expect_foo()
            .returning(|_| 25);
        assert_eq!(25, mock.foo(0));
    }

    #[test]
    #[should_panic(expected =
        "MockFoo::foo: Expectation(<anything>) called 0 time(s) which is fewer than expected 1")]
    fn not_yet_satisfied() {
        let mut mock = MockFoo::new();
        mock.expect_foo()
            .returning(|_| 42)
            .times(1);
        mock.checkpoint();
        panic!("Shouldn't get here!");
    }

    #[test]
    #[should_panic(expected =
        "MockFoo::foo: Expectation(<anything>) called 1 time(s) which is more than expected 0")]
    fn too_many_calls() {
        let mut mock = MockFoo::default();
        mock.expect_foo()
            .returning(|_| 42)
            .times(0);
        let _ = panic::catch_unwind(|| {
            mock.foo(0);
        });
        mock.checkpoint();
        panic!("Shouldn't get here!");
    }

    #[test]
    fn ok() {
        let mut mock = MockFoo::new();
        mock.expect_foo()
            .returning(|_| 5)
            .times(1..3);
        mock.foo(0);
        mock.checkpoint();
    }

    #[test]
    #[should_panic(expected = "MockFoo::foo(0): No matching expectation found")]
    fn removes_old_expectations() {
        let mut mock = MockFoo::new();
        mock.expect_foo()
            .returning(|_| 42)
            .times(1..3);
        mock.foo(0);
        mock.checkpoint();
        mock.foo(0);
        panic!("Shouldn't get here!");
    }
}

mod r#match {
    use super::*;

    /// Unlike Mockers, Mockall calls should use the oldest matching
    /// expectation, if multiple expectations match
    #[test]
    fn fifo_order() {
        let mut mock = MockFoo::new();
        mock.expect_foo()
            .with(predicate::eq(5))
            .returning(|_| 99);
        mock.expect_foo()
            .with(predicate::always())
            .returning(|_| 42);

        assert_eq!(99, mock.foo(5));
    }

    #[test]
    fn one_match() {
        let mut mock0 = MockFoo::new();
        mock0.expect_foo()
            .with(predicate::eq(5))
            .returning(|_| 99);
        mock0.expect_foo()
            .with(predicate::eq(6))
            .returning(|_| 42);
        assert_eq!(42, mock0.foo(6));

        // And in reverse order
        let mut mock1 = MockFoo::new();
        mock1.expect_foo()
            .with(predicate::eq(5))
            .returning(|_| 99);
        mock1.expect_foo()
            .with(predicate::eq(6))
            .returning(|_| 42);
        assert_eq!(99, mock0.foo(5));
    }

    #[test]
    #[should_panic(expected = "MockFoo::bar(5): No matching expectation found")]
    fn with_no_matches() {
        let mut mock = MockFoo::new();
        mock.expect_bar()
            .with(predicate::eq(4))
            .return_const(());
        mock.bar(5);
    }

    #[test]
    fn with_ok() {
        let mut mock = MockFoo::new();
        mock.expect_bar()
            .with(predicate::eq(5))
            .return_const(());
        mock.bar(5);
    }

    #[test]
    fn withf_ok() {
        let mut mock = MockFoo::new();
        mock.expect_bar()
            .withf(|x: &u32| *x == 5)
            .return_const(());
        mock.bar(5);
    }

    #[test]
    #[should_panic(expected = "MockFoo::bar(5): No matching expectation found")]
    fn withf_no_matches() {
        let mut mock = MockFoo::new();
        mock.expect_bar()
            .withf(|x: &u32| *x == 6)
            .return_const(());
        mock.bar(5);
    }

}

mod never {
    use super::*;

    #[test]
    #[should_panic(expected =
        "MockFoo::bar: Expectation(<anything>) should not have been called")]
    fn fail() {
        let mut mock = MockFoo::new();
        mock.expect_bar()
            .returning(|_| ())
            .never();
        mock.bar(0);
    }

    #[test]
    fn ok() {
        let mut mock = MockFoo::new();
        mock.expect_foo()
            .never();
    }
}

#[test]
fn return_const() {
    let mut mock = MockFoo::new();
    mock.expect_foo()
        .return_const(42u32);
    assert_eq!(42, mock.foo(5));
}

#[cfg_attr(not(feature = "nightly"),
    should_panic(expected = "MockFoo::foo: Expectation(<anything>) Returning default values requires"))]
#[cfg_attr(not(feature = "nightly"), allow(unused_must_use))]
#[test]
fn return_default() {
    let mut mock = MockFoo::new();
    mock.expect_foo();
    let r = mock.foo(5);
    assert_eq!(u32::default(), r);
}

#[test]
fn returning() {
    let mut mock = MockFoo::new();
    mock.expect_foo()
        .returning(|x| x + 1);
    assert_eq!(6, mock.foo(5));
}

mod sequence {
    use super::*;

    #[test]
    #[should_panic(expected = "MockFoo::baz(): Method sequence violation")]
    fn fail() {
        let mut seq = Sequence::new();
        let mut mock = MockFoo::new();
        mock.expect_bar()
            .times(1)
            .returning(|_| ())
            .in_sequence(&mut seq);

        mock.expect_baz()
            .times(1)
            .returning(|| ())
            .in_sequence(&mut seq);

        mock.baz();
        mock.bar(0);
    }

    #[test]
    fn ok() {
        let mut seq = Sequence::new();
        let mut mock = MockFoo::new();
        mock.expect_baz()
            .times(1)
            .returning(|| ())
            .in_sequence(&mut seq);

        mock.expect_bar()
            .times(1)
            .returning(|_| ())
            .in_sequence(&mut seq);

        mock.baz();
        mock.bar(0);
    }

    #[test]
    fn ok_variable_count() {
        let mut seq = Sequence::new();
        let mut mock = MockFoo::new();
        mock.expect_baz()
            .times(1..)
            .returning(|| ())
            .in_sequence(&mut seq);

        mock.expect_bar()
            .times(1..)
            .returning(|_| ())
            .in_sequence(&mut seq);

        mock.baz();
        mock.bar(0);
    }

    #[test]
    fn ok_variable_count_skip() {
        let mut seq = Sequence::new();
        let mut mock = MockFoo::new();

        // All of these may be skipped
        mock.expect_bar()
            .with(predicate::eq(0))
            .returning(|_| ())
            .in_sequence(&mut seq);
        mock.expect_bar()
            .with(predicate::eq(1))
            .returning(|_| ())
            .in_sequence(&mut seq);
        mock.expect_bar()
            .with(predicate::eq(2))
            .returning(|_| ())
            .in_sequence(&mut seq);

        // This one may not        
        mock.expect_baz()
            .times(1..)
            .returning(|| ())
            .in_sequence(&mut seq);

        mock.baz()
    }

    #[test]
    #[should_panic]
    fn err_variable_count_skip() {
        let mut seq = Sequence::new();
        let mut mock = MockFoo::new();

        // All of these may be skipped
        mock.expect_bar()
            .with(predicate::eq(0))
            .returning(|_| ())
            .in_sequence(&mut seq);
        mock.expect_bar()
            .with(predicate::eq(1))
            .returning(|_| ())
            .in_sequence(&mut seq);
        mock.expect_bar()
            .with(predicate::eq(2))
            .returning(|_| ())
            .in_sequence(&mut seq);

        // This one may not        
        mock.expect_baz()
            .times(1..)
            .returning(|| ())
            .in_sequence(&mut seq);

        mock.expect_bar()
            .with(predicate::eq(3))
            .returning(|_| ())
            .in_sequence(&mut seq);

        // mock.baz()
        mock.bar(3);
    }

    /// When adding multiple calls of a single method, with the same arguments,
    /// to a sequence, expectations should not be called after they are done if
    /// there are more expectations to follow.
    #[test]
    fn single_method() {
        let mut seq = Sequence::new();
        let mut mock = MockFoo::new();
        mock.expect_foo()
            .times(1)
            .in_sequence(&mut seq)
            .returning(|_| 1);
        mock.expect_foo()
            .times(1)
            .in_sequence(&mut seq)
            .returning(|_| 2);
        mock.expect_foo()
            .times(1)
            .in_sequence(&mut seq)
            .returning(|_| 3);

        assert_eq!(1, mock.foo(0));
        assert_eq!(2, mock.foo(0));
        assert_eq!(3, mock.foo(0));
    }

    #[test]
    fn single_method_variable_count() {
        let mut seq = Sequence::new();
        let mut mock = MockFoo::new();
        mock.expect_foo()
            .times(1..=2)
            .in_sequence(&mut seq)
            .returning(|_| 1);
        mock.expect_foo()
            .times(1)
            .in_sequence(&mut seq)
            .returning(|_| 3);

        assert_eq!(1, mock.foo(0));
        assert_eq!(1, mock.foo(0));
        assert_eq!(3, mock.foo(0));
    }

    #[test]
    fn single_method_variable_count_mixed() {
        let mut seq = Sequence::new();
        let mut mock = MockFoo::new();
        mock.expect_baz()
            .times(1..)
            .returning(|| ())
            .in_sequence(&mut seq);

        mock.expect_bar()
            .times(1..)
            .returning(|_| ())
            .in_sequence(&mut seq);

        
        mock.expect_baz()
            .times(1..)
            .returning(|| ())
            .in_sequence(&mut seq);

        mock.baz();
        mock.bar(0);
        mock.baz();
    }

}

mod times {
    use super::*;

    #[test]
    fn ok() {
        let mut mock = MockFoo::new();
        mock.expect_baz()
            .returning(|| ())
            .times(2);
        mock.baz();
        mock.baz();
    }

    #[test]
    #[should_panic(expected =
        "MockFoo::bar: Expectation(var == 5) called 1 time(s) which is fewer than expected 2")]
    fn too_few() {
        let mut mock = MockFoo::new();
        mock.expect_bar()
            .with(predicate::eq(5))
            .returning(|_| ())
            .times(2);
        mock.bar(5);
    }

    #[test]
    #[should_panic(expected =
        "MockFoo::baz: Expectation(<anything>) called 3 times which is more than the expected 2")]
    fn too_many() {
        let mut mock = MockFoo::new();
        mock.expect_baz()
            .returning(|| ())
            .times(2);
        mock.baz();
        mock.baz();
        mock.baz();
        // Verify that we panic quickly and don't reach code below this point.
        panic!("Shouldn't get here!");
    }

    #[test]
    fn range_ok() {
        let mut mock = MockFoo::new();
        mock.expect_baz()
            .returning(|| ())
            .times(2..4);
        mock.baz();
        mock.baz();

        mock.expect_bar()
            .returning(|_| ())
            .times(2..4);
        mock.bar(0);
        mock.bar(0);
        mock.bar(0);
    }

    #[test]
    #[should_panic(expected =
        "MockFoo::baz: Expectation(<anything>) called 1 time(s) which is fewer than expected 2")]
    fn range_too_few() {
        let mut mock = MockFoo::new();
        mock.expect_baz()
            .returning(|| ())
            .times(2..4);
        mock.baz();
    }

    #[test]
    #[should_panic(expected =
        "MockFoo::baz: Expectation(<anything>) called 4 times which is more than the expected 3")]
    fn range_too_many() {
        let mut mock = MockFoo::new();
        mock.expect_baz()
            .returning(|| ())
            .times(2..4);
        mock.baz();
        mock.baz();
        mock.baz();
        mock.baz();
        // Verify that we panic quickly and don't reach code below this point.
        panic!("Shouldn't get here!");
    }

    #[test]
    fn rangeto_ok() {
        let mut mock = MockFoo::new();
        mock.expect_bar()
            .returning(|_| ())
            .times(..4);
        mock.bar(0);
        mock.bar(0);
        mock.bar(0);
    }

    #[test]
    #[should_panic(expected =
        "MockFoo::baz: Expectation(<anything>) called 4 times which is more than the expected 3")]
    fn rangeto_too_many() {
        let mut mock = MockFoo::new();
        mock.expect_baz()
            .returning(|| ())
            .times(..4);
        mock.baz();
        mock.baz();
        mock.baz();
        mock.baz();
    }

    #[test]
    fn rangeinclusive_ok() {
        let mut mock = MockFoo::new();
        mock.expect_bar()
            .returning(|_| ())
            .times(2..=4);
        mock.bar(0);
        mock.bar(0);
        mock.bar(0);
        mock.bar(0);
    }

    #[test]
    fn rangefrom_ok() {
        let mut mock = MockFoo::new();
        mock.expect_baz()
            .returning(|| ())
            .times(2..);
        mock.baz();
        mock.baz();

        mock.expect_bar()
            .returning(|_| ())
            .times(2..);
        mock.bar(0);
        mock.bar(0);
        mock.bar(0);
    }

    #[test]
    #[should_panic(expected =
        "MockFoo::baz: Expectation(<anything>) called 1 time(s) which is fewer than expected 2")]
    fn rangefrom_too_few() {
        let mut mock = MockFoo::new();
        mock.expect_baz()
            .returning(|| ())
            .times(2..);
        mock.baz();
    }
}

#[test]
fn times_full() {
    let mut mock = MockFoo::new();
    mock.expect_baz()
        .returning(|| ())
        .times(1)
        .times(..);
    mock.baz();
    mock.baz();
}
