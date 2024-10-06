use crate::{
    search::{self, SearchId, SearchParameters, SharedSearchId},
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
    pub(crate) receiver: Receiver<SearchMessage>,
    pub(crate) current_search_id: SharedSearchId,
}

impl Default for SearchEngine {
    fn default() -> Self {
        let (sender, receiver) = flume::unbounded();

        SearchEngine {
            sender,
            receiver,
            current_search_id: Arc::new(AtomicUsize::new(0)),
        }
    }
}

impl SearchEngine {
    pub fn receiver(&self) -> Receiver<SearchMessage> {
        self.receiver.clone()
    }

    pub fn search(&self, params: SearchParameters) {
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
            receiver: self.receiver.clone(),
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
