macro_rules! call {
    { $fn:ident($($arg:expr),*) } => {
        {
            let result = unsafe { $fn($($arg),*) };
            match $crate::Error::from_code(result) {
                None => Ok(()),
                Some(err) => Err(err),
            }
        }
    };
}

pub(crate) use call;
