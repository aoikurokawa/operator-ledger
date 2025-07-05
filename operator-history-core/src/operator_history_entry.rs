use std::net::{IpAddr, Ipv4Addr};

use bytemuck::{Pod, Zeroable};
use jito_bytemuck::types::{PodU16, PodU32, PodU64};
use shank::ShankType;

use crate::client_version::ClientVersion;

/// Operator History Entry
#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable, ShankType)]
#[repr(C)]
pub struct OperatorHistotyEntry {
    /// Activated stake lamports
    activated_stake_lamports: PodU64,

    /// Rank of opeator by stake amount
    rank: PodU32,

    /// Operator fee BPS
    operator_fee_bps: PodU16,

    /// Epoch
    epoch: PodU16,

    /// Version
    version: ClientVersion,

    /// IP address
    ip: [u8; 4],

    /// Reserved space
    reserved_space: [u8; 328],
}

impl Default for OperatorHistotyEntry {
    fn default() -> Self {
        Self {
            activated_stake_lamports: PodU64::default(),
            rank: PodU32::default(),
            operator_fee_bps: PodU16::default(),
            epoch: PodU16::default(),
            version: ClientVersion::default(),
            ip: [0; 4],
            reserved_space: [0; 328],
        }
    }
}

impl OperatorHistotyEntry {
    /// Construct a new [`OperatorHistoryEntry`]
    pub fn new(
        activated_stake_lamports: u64,
        rank: u32,
        operator_fee_bps: u16,
        epoch: u16,
        version: ClientVersion,
        ip: [u8; 4],
    ) -> Self {
        Self {
            activated_stake_lamports: PodU64::from(activated_stake_lamports),
            rank: PodU32::from(rank),
            operator_fee_bps: PodU16::from(operator_fee_bps),
            epoch: PodU16::from(epoch),
            version,
            ip,
            reserved_space: [0; 328],
        }
    }

    /// Activated stake lamports
    pub fn activated_stake_lamports(&self) -> u64 {
        self.activated_stake_lamports.into()
    }

    /// Rank
    pub fn rank(&self) -> u32 {
        self.rank.into()
    }

    /// Rank
    pub fn operator_fee_bps(&self) -> u16 {
        self.operator_fee_bps.into()
    }

    /// Epoch
    pub fn epoch(&self) -> u16 {
        self.epoch.into()
    }

    /// Version
    pub const fn version(&self) -> ClientVersion {
        self.version
    }

    /// IP address
    pub const fn ip_address(&self) -> IpAddr {
        IpAddr::V4(Ipv4Addr::new(
            self.ip[0], self.ip[1], self.ip[2], self.ip[3],
        ))
    }
}
