use crate::{
    search::{self, SearchId, SearchParameter, SharedSearchId},
    SearchMessage,
};
use flume::{Receiver, Sender};
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    thread,
};

pub struct SearchEngine {
    pub(crate) sender: Sender<SearchMessage>,
    pub(crate) current_search_id: SharedSearchId,
}

impl SearchEngine {
    pub fn new() -> (Self, Receiver<SearchMessage>) {
        let (sender, receiver) = flume::unbounded();

        (
            SearchEngine {
                sender,
                current_search_id: Arc::new(AtomicUsize::new(0)),
            },
            receiver,
        )
    }

    pub fn search(&self, params: SearchParameter) {
        self.current_search_id.fetch_add(1, Ordering::Release);

        let engine = self.clone();
        thread::spawn(move || search::run(engine, params));
    }

    pub fn cancel(&self) {
        self.current_search_id.fetch_add(1, Ordering::Release);
    }

    pub fn is_current(&self, message: &SearchMessage) -> bool {
        let current = self.current_search_id.load(Ordering::Acquire);

        message.search() == current
    }

    pub(crate) fn clone(&self) -> Self {
        SearchEngine {
            sender: self.sender.clone(),
            current_search_id: self.current_search_id.clone(),
        }
    }

    pub(crate) fn send_error(
        &self,
        search: SearchId,
        path: PathBuf,
        message: String,
    ) -> Result<(), flume::SendError<SearchMessage>> {
        self.sender
            .send(SearchMessage::Error(crate::result::SearchError {
                search,
                path,
                message,
            }))
    }
}
