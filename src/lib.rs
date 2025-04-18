pub mod message;
pub mod command;
pub mod channel;
pub mod connection;
pub mod types;


fn enable_logging() {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "debug");
    }
    env_logger::init();
}
