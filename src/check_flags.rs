use crate::utils::{is_root, pid_link};

/// Modifies the process info table populate behaviour to discard processes with different root
#[derive(Clone)]
pub enum CheckRoot {
    Yes(String),
    No,
}

impl From<bool> for CheckRoot {
    fn from(value: bool) -> Self {
        if is_root() && value {
            pid_link(std::process::id() as i32, "root").map_or(Self::No, Self::Yes)
        } else {
            Self::No
        }
    }
}

/// Modifies process match behaviour to also match names of running scripts
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum CheckScripts {
    Yes,
    No,
}

impl From<bool> for CheckScripts {
    fn from(value: bool) -> Self {
        if value {
            Self::Yes
        } else {
            Self::No
        }
    }
}

/// Modifies process match behaviour to also matche thread names
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum CheckThreads {
    Yes,
    No,
}

impl From<bool> for CheckThreads {
    fn from(value: bool) -> Self {
        if value {
            Self::Yes
        } else {
            Self::No
        }
    }
}

/// Modifies process match behaviour to also matche kernel workers
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum CheckWorkers {
    Yes,
    No,
}

impl From<bool> for CheckWorkers {
    fn from(value: bool) -> Self {
        if value {
            Self::Yes
        } else {
            Self::No
        }
    }
}
