#[macro_export]
macro_rules! call {
    { $fn:ident($($arg:expr),*) } => {
        {
            let result = unsafe { $fn($($arg),*) };
            match $crate::error::Error::new(result) {
                None => Ok(()),
                Some(err) => Err(err),
            }
        }
    };
}
