use std::io::{self, Read, Write};
use std::fmt::Debug;
use anyhow::{Result, bail};

use super::io_trait::{IO, InputIO, OutputIO, IOCapabilities};

/// Stub network IO implementation
#[derive(Debug)]
pub struct NetworkIO {
    _address: String,
    closed: bool,
    capabilities: IOCapabilities,
}

impl NetworkIO {
    /// Create a new NetworkIO stub for the given address.
    pub fn new(address: String) -> Self {
        let caps = IOCapabilities {
            supports_input: false,    // stub does not support input
            supports_output: true,    // stub supports output
            supports_sync: false,     // network IO usually async
            supports_async: true,
            supports_seek: false,
            supports_filtering: false,
        };

        NetworkIO { _address: address, closed: false, capabilities: caps }
    }
}

impl IO for NetworkIO {
    fn name(&self) -> &str {
        "network_io"
    }

    fn capabilities(&self) -> IOCapabilities {
        self.capabilities.clone()
    }

    fn is_ready(&self) -> bool {
        !self.closed
    }

    fn close(&mut self) -> Result<()> {
        self.closed = true;
        Ok(())
    }
}

impl Read for NetworkIO {
    fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
        Err(io::Error::other("NetworkIO read not implemented"))
    }
}

impl Write for NetworkIO {
    fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
        Err(io::Error::other("NetworkIO write not implemented"))
    }

    fn flush(&mut self) -> io::Result<()> {
        Err(io::Error::other("NetworkIO flush not implemented"))
    }
}

impl InputIO for NetworkIO {
    fn read_line(&mut self) -> Result<String> {
        bail!("NetworkIO read_line not implemented")
    }

    fn has_data_available(&self) -> bool {
        false
    }
}

impl OutputIO for NetworkIO {
    fn write_line(&mut self, _line: &str) -> Result<()> {
        bail!("NetworkIO write_line not implemented")
    }

    fn flush_output(&mut self) -> Result<()> {
        bail!("NetworkIO flush_output not implemented")
    }
} 