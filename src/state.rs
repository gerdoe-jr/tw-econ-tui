use crate::stringarray::StringArray;

#[derive(Debug, Clone, Copy)]
pub enum Screen {
    Welcome,
    Main(Main),
    AddConnection(AddConnection),
    Exit
}

#[derive(Debug, Clone, Copy)]
pub struct Main {
    pub active: MainElements,
    pub connection: u8
}

impl Main {
    pub fn new() -> Self {
        Self { active: MainElements::Add, connection: 0 }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AddConnection {
    pub active: AddConnectionElements,
    pub fields: [StringArray<64>; 3]
}

impl AddConnection {
    pub fn new() -> Self {
        Self {
            active: AddConnectionElements::Name,
            fields: [StringArray::new(); 3]
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MainElements {
    Connections,
    Console,
    Input,
    Add
}

impl MainElements {
    pub fn next(self) -> Self {
        match self {
            Self::Connections => Self::Console,
            Self::Console => Self::Input,
            Self::Input => Self::Add,
            Self::Add => Self::Connections,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            Self::Connections => Self::Add,
            Self::Console => Self::Connections,
            Self::Input => Self::Console,
            Self::Add => Self::Input,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AddConnectionElements {
    Name,
    Address,
    Password,
    OkButton
}

impl AddConnectionElements {
    pub fn next(self) -> Self {
        match self {
            Self::Name => Self::Address,
            Self::Address => Self::Password,
            Self::Password => Self::OkButton,
            Self::OkButton => Self::Name
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            Self::Name => Self::OkButton,
            Self::Address => Self::Name,
            Self::Password => Self::Address,
            Self::OkButton => Self::Password
        }
    }
}
