use bytemuck::{Pod, Zeroable};
use jito_bytemuck::types::{PodBool, PodU64};
use operator_history_sdk::error::OperatorHistoryError;
use shank::ShankType;

use crate::{operator_history_entry::OperatorHistotyEntry, OPERATOR_HISTORY_ENTRY_MAX_ITEMS};

pub fn find_insert_position(
    arr: &[OperatorHistotyEntry],
    idx: usize,
    epoch: u16,
) -> Result<Option<usize>, OperatorHistoryError> {
    let len = arr.len();
    if len == 0 {
        return Ok(None);
    }

    let insert_pos = if idx != len.checked_sub(1).ok_or(OperatorHistoryError::Arithmetic)?
        && arr[idx.checked_add(1).ok_or(OperatorHistoryError::Arithmetic)?].epoch()
            == OperatorHistotyEntry::default().epoch()
    {
        // If the circ buf still has default values in it, we do a normal binary search without factoring for wraparound.
        let len = idx.checked_add(1).ok_or(OperatorHistoryError::Arithmetic)?;
        let mut left = 0;
        let mut right = len;
        while left < right {
            let mid = left
                .checked_add(right)
                .and_then(|x| x.checked_div(2))
                .ok_or(OperatorHistoryError::Arithmetic)?;
            match arr[mid].epoch().cmp(&epoch) {
                std::cmp::Ordering::Equal => return Ok(None),
                std::cmp::Ordering::Less => {
                    left = mid.checked_add(1).ok_or(OperatorHistoryError::Arithmetic)?
                }
                std::cmp::Ordering::Greater => right = mid,
            }
        }
        left.checked_rem(arr.len())
            .ok_or(OperatorHistoryError::Arithmetic)?
    } else {
        // Binary search with wraparound
        let mut left = 0;
        let mut right = len;
        while left < right {
            let mid = left
                .checked_add(right)
                .and_then(|x| x.checked_div(2))
                .ok_or(OperatorHistoryError::Arithmetic)?;
            // idx + 1 is the index of the smallest epoch in the array

            let mid_idx = idx
                .checked_add(1)
                .and_then(|x| x.checked_add(mid))
                .and_then(|y| y.checked_rem(len))
                .ok_or(OperatorHistoryError::Arithmetic)?;
            match arr[mid_idx].epoch().cmp(&epoch) {
                std::cmp::Ordering::Equal => return Ok(None),
                std::cmp::Ordering::Less => {
                    left = mid.checked_add(1).ok_or(OperatorHistoryError::Arithmetic)?
                }
                std::cmp::Ordering::Greater => right = mid,
            }
        }

        idx.checked_add(1)
            .and_then(|x| x.checked_add(left))
            .and_then(|y| y.checked_rem(len))
            .ok_or(OperatorHistoryError::Arithmetic)?
    };

    if arr[insert_pos].epoch() == epoch {
        return Ok(None);
    }

    Ok(Some(insert_pos))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable, ShankType)]
#[repr(C)]
pub struct CircBuf {
    /// Index
    index: PodU64,

    /// Is empty
    is_empty: PodBool,

    /// Array of operator history
    arr: [OperatorHistotyEntry; OPERATOR_HISTORY_ENTRY_MAX_ITEMS],

    /// Reserved space
    reserved_space: [u8; 328],
}

impl Default for CircBuf {
    fn default() -> Self {
        Self::new()
    }
}

impl CircBuf {
    pub fn new() -> Self {
        Self {
            index: PodU64::from(0),
            is_empty: PodBool::from_bool(false),
            arr: [OperatorHistotyEntry::default(); OPERATOR_HISTORY_ENTRY_MAX_ITEMS],
            reserved_space: [0; 328],
        }
    }

    /// Index
    pub fn index(&self) -> u64 {
        self.index.into()
    }

    /// Is empty
    pub fn is_empty(&self) -> bool {
        self.is_empty.into()
    }

    /// Push an [`OperatorHistoryEntry`] item
    pub fn push(&mut self, item: OperatorHistotyEntry) -> Result<(), OperatorHistoryError> {
        let index = self
            .index()
            .checked_add(1)
            .and_then(|x| x.checked_rem(self.arr.len() as u64))
            .ok_or(OperatorHistoryError::Arithmetic)?;

        self.index = PodU64::from(index);
        self.arr[self.index() as usize] = item;
        self.is_empty = PodBool::from_bool(false);

        Ok(())
    }

    /// Fetch last [`OperatorHistoryEntry`] element
    pub fn last(&self) -> Option<&OperatorHistotyEntry> {
        if self.is_empty() {
            None
        } else {
            Some(&self.arr[self.index() as usize])
        }
    }

    /// Fetch last [`OperatorHistoryEntry`] element
    pub fn last_mut(&mut self) -> Option<&mut OperatorHistotyEntry> {
        if self.is_empty() {
            None
        } else {
            Some(&mut self.arr[self.index() as usize])
        }
    }

    /// Fetch mutable array of [`OperatorHistoryEntry`]
    pub const fn arr_mut(&mut self) -> &mut [OperatorHistotyEntry] {
        &mut self.arr
    }

    /// Given a new entry and epoch, inserts the entry into the buffer in sorted order
    /// Will not insert if the epoch is out of range or already exists in the buffer
    pub fn insert(
        &mut self,
        entry: OperatorHistotyEntry,
        epoch: u16,
    ) -> Result<(), OperatorHistoryError> {
        if self.is_empty() {
            return Err(OperatorHistoryError::EpochOutOfRange);
        }

        // Find the lowest epoch in the buffer to ensure the new epoch is valid
        let min_epoch = {
            let next_i = (self.index() as usize)
                .checked_add(1)
                .and_then(|x| x.checked_rem(self.arr.len()))
                .ok_or(OperatorHistoryError::Arithmetic)?;
            if self.arr[next_i].epoch() == OperatorHistotyEntry::default().epoch() {
                self.arr[0].epoch()
            } else {
                self.arr[next_i].epoch()
            }
        };

        // If epoch is less than min_epoch or greater than max_epoch in the buffer, return error
        if epoch < min_epoch || epoch > self.arr[self.index() as usize].epoch() {
            return Err(OperatorHistoryError::EpochOutOfRange);
        }

        let insert_pos = find_insert_position(&self.arr, self.index() as usize, epoch)?
            .ok_or(OperatorHistoryError::DuplicateEpoch)?;

        // If idx < insert_pos, the shifting needs to wrap around
        let end_index = if self.index() < insert_pos as u64 {
            (self.index() as usize)
                .checked_add(self.arr.len())
                .ok_or(OperatorHistoryError::Arithmetic)?
        } else {
            self.index() as usize
        };

        // Shift all elements to the right to make space for the new entry, starting with current idx
        for i in (insert_pos..=end_index).rev() {
            let i = i
                .checked_rem(self.arr.len())
                .ok_or(OperatorHistoryError::Arithmetic)?;
            let next_i = i
                .checked_add(1)
                .and_then(|x| x.checked_rem(self.arr.len()))
                .ok_or(OperatorHistoryError::Arithmetic)?;

            self.arr[next_i] = self.arr[i];
        }

        self.arr[insert_pos] = entry;

        let index = self
            .index()
            .checked_add(1)
            .and_then(|x| x.checked_rem(self.arr.len() as u64))
            .ok_or(OperatorHistoryError::Arithmetic)?;

        self.index = PodU64::from(index);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;

    use crate::{
        client_version::ClientVersion, operator_history_entry::OperatorHistotyEntry,
        OPERATOR_HISTORY_ENTRY_MAX_ITEMS,
    };

    use super::CircBuf;

    #[test]
    fn test_new_circbuf() {
        let buf = CircBuf::new();
        assert_eq!(buf.index(), 0);
        assert!(!buf.is_empty());
    }

    #[test]
    fn test_push_single_item() {
        let mut buf = CircBuf::new();
        let entry =
            OperatorHistotyEntry::new(1, 1, 1, 1, ClientVersion::new(0, 0, 1), [1, 1, 1, 1]);

        let result = buf.push(entry);
        assert!(result.is_ok());
        assert_eq!(buf.index(), 1);
        assert!(!buf.is_empty());

        let last = buf.last().unwrap();
        assert_eq!(last.epoch(), 1);
    }

    #[test]
    fn test_push_multiple_items() {
        let mut buf = CircBuf::new();

        for i in 0..5 {
            let entry = OperatorHistotyEntry::new(
                i as u64,
                i as u32,
                i as u16,
                i as u16,
                ClientVersion::new(i, i, i),
                [i, i, i, i],
            );
            buf.push(entry).unwrap();
        }

        assert_eq!(buf.index(), 5);

        let last = buf.last().unwrap();
        assert_eq!(last.activated_stake_lamports(), 4);
        assert_eq!(last.rank(), 4);
        assert_eq!(last.operator_fee_bps(), 4);
        assert_eq!(last.version(), ClientVersion::new(4, 4, 4));
        assert_eq!(last.ip_address(), Ipv4Addr::new(4, 4, 4, 4));
    }

    #[test]
    fn test_push_wraparound() {
        let mut buf = CircBuf::new();
        let max_items = OPERATOR_HISTORY_ENTRY_MAX_ITEMS;

        for i in 0..max_items {
            let entry = OperatorHistotyEntry::new(
                i as u64,
                i as u32,
                i as u16,
                i as u16,
                ClientVersion::new(i as u8, i as u8, i as u8),
                [i as u8, i as u8, i as u8, i as u8],
            );
            buf.push(entry).unwrap();
        }

        let entry = OperatorHistotyEntry::new(
            1000,
            1000,
            1000,
            1000,
            ClientVersion::new(255, 255, 255),
            [255, 255, 255, 255],
        );
        buf.push(entry).unwrap();

        assert_eq!(buf.index(), 1);

        let last = buf.last().unwrap();
        assert_eq!(last.activated_stake_lamports(), 1000);
        assert_eq!(last.rank(), 1000);
        assert_eq!(last.operator_fee_bps(), 1000);
        assert_eq!(last.version(), ClientVersion::new(255, 255, 255));
        assert_eq!(last.ip_address(), Ipv4Addr::new(255, 255, 255, 255));
    }

    #[test]
    fn test_insert_valid_epoch() {
        let mut buf = CircBuf::new();

        for i in 0..5 {
            if i == 3 {
                continue;
            }

            let entry = OperatorHistotyEntry::new(
                i as u64,
                i as u32,
                i as u16,
                i as u16,
                ClientVersion::new(i, i, i),
                [i, i, i, i],
            );
            buf.push(entry).unwrap();
        }

        let initial_index = buf.index();

        let entry = OperatorHistotyEntry::new(
            100,
            100,
            100,
            3,
            ClientVersion::new(100, 100, 100),
            [100, 100, 100, 100],
        );
        buf.insert(entry, 3).unwrap();

        assert_eq!(
            buf.index(),
            (initial_index + 1) % (OPERATOR_HISTORY_ENTRY_MAX_ITEMS as u64)
        )
    }
}
