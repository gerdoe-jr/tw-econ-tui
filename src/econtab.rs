use tw_econ::connection::Connection;

pub type EconId = u8;

pub struct EconTab {
    pub name: String,
    pub connection: Connection<2048, 1>,
    pub messages: Vec<String>,
    pub buffer: String,
    pub scroll: u16
}
