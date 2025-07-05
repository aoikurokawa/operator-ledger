use bytemuck::{Pod, Zeroable};
use shank::ShankType;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Pod, Zeroable, ShankType)]
#[repr(C)]
pub struct ClientVersion {
    /// Major
    major: u8,

    /// Minor
    minor: u8,

    /// Patch
    patch: u8,
}

impl ClientVersion {
    pub const fn new(major: u8, minor: u8, patch: u8) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// Fetch major number
    pub const fn major(&self) -> u8 {
        self.major
    }

    /// Fetch minor number
    pub const fn minor(&self) -> u8 {
        self.minor
    }

    /// Fetch patch number
    pub const fn patch(&self) -> u8 {
        self.patch
    }
}
