pub struct WarnDefer<'s> {
    pub failed: bool,
    pub content: &'s str,
}

impl<'s> WarnDefer<'s> {
    pub fn new(content: &'s str) -> Self {
        Self {
            failed: true,
            content,
        }
    }

    pub fn success(&mut self) {
        self.failed = false;
    }
}

impl Drop for WarnDefer<'_> {
    fn drop(&mut self) {
        if self.failed {
            tracing::warn!("{}.interupted", self.content);
        }
    }
}
