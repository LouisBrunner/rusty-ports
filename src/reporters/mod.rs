pub mod console;

pub trait Reporter {
    fn started(&self);

    fn server_started(&self, port: u16);

    fn client_connected(&self, id: usize, port: u16);
    fn client_message_received(&self, id: usize, msg: &[u8]);
    fn client_disconnected(&self, id: usize);

    fn server_stopping(&self, port: u16);
    fn server_stopped(&self, port: u16);

    fn stopping(&self);
    fn stopped(&self);

    fn error(&self, msg: String);
    fn warning(&self, msg: String);
}
