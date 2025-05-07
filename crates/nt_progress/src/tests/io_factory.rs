use crate::io::{DefaultIOFactory, IOFactory, IOMode};
use std::path::Path;

#[test]
fn test_default_factory_file_io_read_mode() {
    let factory = DefaultIOFactory::default();
    let mut io = factory.create_file_io(Path::new("test"), IOMode::Read).unwrap();
    assert_eq!(io.name(), "memory_io");
    let caps = io.capabilities();
    assert!(caps.supports_input, "Read mode should support input");
    assert!(!caps.supports_output, "Read mode should not support output");
    assert!(io.is_ready());
    io.close().unwrap();
    assert!(!io.is_ready(), "Closed IO should not be ready");
}

#[test]
fn test_default_factory_file_io_write_mode() {
    let factory = DefaultIOFactory::default();
    let io = factory.create_file_io(Path::new("test"), IOMode::Write).unwrap();
    assert_eq!(io.name(), "memory_io");
    let caps = io.capabilities();
    assert!(!caps.supports_input, "Write mode should not support input");
    assert!(caps.supports_output, "Write mode should support output");
}

#[test]
fn test_default_factory_file_io_append_mode() {
    let factory = DefaultIOFactory::default();
    let io = factory.create_file_io(Path::new("test"), IOMode::Append).unwrap();
    let caps = io.capabilities();
    assert!(!caps.supports_input, "Append mode should not support input");
    assert!(caps.supports_output, "Append mode should support output");
}

#[test]
fn test_default_factory_file_io_readwrite_mode() {
    let factory = DefaultIOFactory::default();
    let io = factory.create_file_io(Path::new("test"), IOMode::ReadWrite).unwrap();
    let caps = io.capabilities();
    assert!(caps.supports_input, "ReadWrite mode should support input");
    assert!(caps.supports_output, "ReadWrite mode should support output");
}

#[test]
fn test_default_factory_memory_io_modes() {
    let factory = DefaultIOFactory::default();
    for mode in &[IOMode::Read, IOMode::Write, IOMode::Append, IOMode::ReadWrite] {
        let io = factory.create_memory_io(None, *mode).unwrap();
        assert_eq!(io.name(), "memory_io");
        let caps = io.capabilities();
        assert_eq!(caps.supports_input, matches!(*mode, IOMode::Read | IOMode::ReadWrite), "Mode {:?} input capability mismatch", mode);
        assert_eq!(caps.supports_output, matches!(*mode, IOMode::Write | IOMode::Append | IOMode::ReadWrite), "Mode {:?} output capability mismatch", mode);
    }
} 