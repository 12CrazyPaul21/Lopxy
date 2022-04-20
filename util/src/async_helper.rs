#[macro_export]
macro_rules! joins_result {
    ( $($x:expr), * ) => {
        (
            $(
                $x.await,
            )*
        )
    };
}

#[macro_export]
macro_rules! joins {
    ( $($x:expr), * ) => {
        $(
            match $x.await { _ => {} };
        )*
    };
}