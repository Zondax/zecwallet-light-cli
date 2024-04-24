#[derive(Debug, Clone)]
pub struct SendProgress {
    pub id: u32,
    pub is_send_in_progress: bool,
    pub progress: u32,
    pub total: u32,
    pub last_error: Option<String>,
    pub last_txid: Option<String>,
}

impl SendProgress {
    pub(crate) fn new(id: u32) -> Self {
        SendProgress { id, is_send_in_progress: false, progress: 0, total: 0, last_error: None, last_txid: None }
    }
}
