//! <https://github.com/EOSIO/eosio.cdt/blob/796ff8bee9a0fc864f665a0a4d018e0ff18ac383/libraries/eosiolib/contracts/eosio/producer_schedule.hpp#L54-L69>
use crate::{NumBytes, ProducerKey, Read, Write};

/// Defines both the order, account name, and signing keys of the active set
/// of producers.
#[derive(Read, Write, NumBytes, Clone, Default)]
#[eosio_core_root_path = "crate"]
pub struct ProducerSchedule {
    /// Version number of the schedule. It is sequentially incrementing
    /// version number.
    pub version: u32,
    /// List of producers for this schedule, including its signing key
    pub producers: Vec<ProducerKey>,
}
