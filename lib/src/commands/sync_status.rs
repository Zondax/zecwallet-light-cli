use json::object;
use zcash_primitives::consensus;

use crate::commands::Command;
use crate::lightclient::LightClient;
use crate::RT;

pub struct SyncStatusCommand {}

impl<P: consensus::Parameters + Send + Sync + 'static> Command<P> for SyncStatusCommand {
    fn help(&self) -> String {
        let mut h = vec![];
        h.push("Get the sync status of the wallet");
        h.push("Usage:");
        h.push("syncstatus");
        h.push("");

        h.join("\n")
    }

    fn short_help(&self) -> String {
        "Get the sync status of the wallet".to_string()
    }

    fn exec(
        &self,
        _args: &[&str],
        lightclient: &LightClient<P>,
    ) -> String {
        RT.block_on(async move {
            let status = lightclient.do_sync_status().await;

            let o = if status.in_progress {
                object! {
                    "sync_id" => status.sync_id,
                    "in_progress" => status.in_progress,
                    "last_error" => status.last_error,
                    "start_block" => status.start_block,
                    "end_block" => status.end_block,
                    "synced_blocks" => status.blocks_done,
                    "trial_decryptions_blocks" => status.trial_dec_done,
                    "txn_scan_blocks" => status.txn_scan_done,
                    "total_blocks" => status.blocks_total,
                    "batch_num" => status.batch_num,
                    "batch_total" => status.batch_total
                }
            } else {
                object! {
                    "sync_id" => status.sync_id,
                    "in_progress" => status.in_progress,
                    "last_error" => status.last_error,
                }
            };
            o.pretty(2)
        })
    }
}
