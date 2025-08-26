#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Health {
    Nominal,
    Degraded(String),
    Critical(String),
    NonFunctional(String),
    Irrecoverable(String),
}
