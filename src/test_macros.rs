macro_rules! not_err {
    ($e:expr) => {
        match $e {
            Ok(e) => e,
            Err(e) => panic!("{} failed with {:?}", stringify!($e), e),
        }
    };
}

macro_rules! is_err {
    ($e:expr) => {
        match $e {
            Ok(e) => panic!(
                "{} did not return with an error, but with {:?}",
                stringify!($e),
                e
            ),
            Err(e) => e,
        }
    };
}

macro_rules! assert_matches {
    ($e:expr, $p:pat) => {
        assert_matches!($e, $p, ())
    };
    ($e:expr, $p:pat, $f:expr) => {
        match $e {
            $p => $f,
            e => panic!(
                "{}: Expected pattern {} \ndoes not match {:?}",
                stringify!($e),
                stringify!($p),
                e
            ),
        }
    };
}
