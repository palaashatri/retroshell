use crate::{BusMessage, Result};

pub trait Transport: Send + Sync {
    fn send(&self, message: BusMessage) -> Result<()>;
    fn receive(&self) -> Result<Option<BusMessage>>;
    fn connect(&mut self, endpoint: &str) -> Result<()>;
    fn disconnect(&mut self) -> Result<()>;
    fn is_connected(&self) -> bool;
}

pub struct LocalTransport {
    connected: bool,
    #[allow(dead_code)]
    messages: Vec<BusMessage>,
}

impl LocalTransport {
    pub fn new() -> Self {
        Self { connected: false, messages: vec![] }
    }
}

impl Transport for LocalTransport {
    fn send(&self, message: BusMessage) -> Result<()> {
        tracing::debug!("[LocalTransport] sent: {:?}", message);
        Ok(())
    }

    fn receive(&self) -> Result<Option<BusMessage>> {
        Ok(None)
    }

    fn connect(&mut self, _endpoint: &str) -> Result<()> {
        self.connected = true;
        Ok(())
    }

    fn disconnect(&mut self) -> Result<()> {
        self.connected = false;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }
}

#[cfg(feature = "dbus")]
pub mod dbus_transport {
    use crate::{BusMessage, Result, Transport};

    pub struct DBusTransport {
        connected: bool,
    }

    impl DBusTransport {
        pub fn new() -> Self {
            Self { connected: false }
        }
    }

    impl Transport for DBusTransport {
        fn send(&self, message: BusMessage) -> Result<()> {
            tracing::debug!("[DBus] sending: {:?}", message);
            Ok(())
        }

        fn receive(&self) -> Result<Option<BusMessage>> {
            Ok(None)
        }

        fn connect(&mut self, _endpoint: &str) -> Result<()> {
            self.connected = true;
            Ok(())
        }

        fn disconnect(&mut self) -> Result<()> {
            self.connected = false;
            Ok(())
        }

        fn is_connected(&self) -> bool {
            self.connected
        }
    }
}
