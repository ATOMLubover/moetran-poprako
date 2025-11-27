pub trait ResultExt<T, E> {
    fn log_err(self) -> Self;
}

impl<T, E: std::fmt::Display> ResultExt<T, E> for Result<T, E> {
    fn log_err(self) -> Self {
        if let Err(ref err) = self {
            tracing::error!("{}", err);
        }

        self
    }
}
