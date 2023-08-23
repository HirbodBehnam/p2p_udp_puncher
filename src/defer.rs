pub struct ScopeCall<F: FnMut()> {
    pub c: F
}
impl<F: FnMut()> Drop for ScopeCall<F> {
    fn drop(&mut self) {
        (self.c)();
    }
}

macro_rules! defer {
    ($e:expr) => (
        let _scope_call = ScopeCall { c: || -> () { $e; } };
    )
}

pub(crate) use defer;