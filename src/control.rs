use signal_hook::consts::SIGINT;
use signal_hook::SigId;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Default)]
pub(crate) struct Control {
    admission: AtomicBool,
    fatal: AtomicBool,
    interrupted: Arc<AtomicBool>,
}

pub(crate) struct SignalRegistration(SigId);
impl Drop for SignalRegistration {
    fn drop(&mut self) {
        signal_hook::low_level::unregister(self.0);
    }
}

impl Control {
    pub fn register_sigint(&self) -> std::io::Result<SignalRegistration> {
        signal_hook::flag::register(SIGINT, self.interrupted.clone()).map(SignalRegistration)
    }
    pub fn requested(&self) -> bool {
        self.admission.load(Ordering::Acquire) || self.interrupted()
    }
    pub fn interrupted(&self) -> bool {
        self.interrupted.load(Ordering::Acquire)
    }
    pub fn fatal(&self) -> bool {
        self.fatal.load(Ordering::Acquire)
    }
    pub fn graceful(&self) {
        self.admission.store(true, Ordering::Release);
    }
    pub fn abort(&self) {
        self.fatal.store(true, Ordering::Release);
        self.graceful();
    }
    #[cfg(test)]
    pub fn interrupt(&self) {
        self.interrupted.store(true, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dropping_registration_removes_its_callback() {
        let removed = Control::default();
        let registration = removed.register_sigint().unwrap();
        let probe = Control::default();
        let _probe_registration = probe.register_sigint().unwrap();
        drop(registration);
        signal_hook::low_level::raise(SIGINT).unwrap();
        assert!(!removed.interrupted());
        assert!(probe.interrupted());
    }
}
