use std::task::Poll;
use std::{fmt::Debug, sync::Arc};

use futures::Future;
use futures::Stream;
use libp2p::PeerId;
use rust_hashgraph::algorithm::event::EventWrapper;
use rust_hashgraph::algorithm::{
    datastructure::{self, EventCreateError, Graph},
    PushError,
};
use serde::Serialize;
use thiserror::Error;
use tokio::pin;
use tokio::sync::Notify;

use super::Transaction;

pub struct GraphWrapper<TDataId, TPieceId> {
    // todo: replace parentheses - ()
    inner: Graph<EventPayload<TDataId, TPieceId>, PeerId, (), ()>,
    state_updated: Arc<Notify>,
    transaction_buffer: Vec<Transaction<TDataId, TPieceId, PeerId>>,
}

#[derive(Serialize, PartialEq, Eq, std::hash::Hash, Debug, Clone)]
struct EventPayload<TDataId, TPieceId> {
    transacitons: Vec<Transaction<TDataId, TPieceId, PeerId>>,
}

impl<TDataId, TPieceId> GraphWrapper<TDataId, TPieceId> {
    pub fn inner(&self) -> &Graph<EventPayload<TDataId, TPieceId>, PeerId, (), ()> {
        &self.inner
    }
}

#[derive(Error, Debug)]
pub enum ApplySyncError {
    #[error(transparent)]
    PushError(#[from] PushError<PeerId>),
    #[error("Peer `from` is unknown")]
    UnknownPeer(PeerId),
    #[error("Failed to create new gossip event")]
    CreateError(#[from] EventCreateError<PeerId>),
}

impl<TDataId, TPieceId> GraphWrapper<TDataId, TPieceId>
where
    TDataId: Serialize + Eq + std::hash::Hash + Debug + Clone,
    TPieceId: Serialize + Eq + std::hash::Hash + Debug + Clone,
{
    pub fn apply_sync(
        &mut self,
        from: &PeerId,
        sync_jobs: datastructure::sync::Jobs<EventPayload<TDataId, TPieceId>, PeerId>,
    ) -> Result<(), ApplySyncError> {
        if !sync_jobs.as_linear().is_empty() {
            self.state_updated.notify_one();
        }
        for next_event in sync_jobs.into_linear() {
            self.inner.push_event(next_event)?;
        }
        let payload = EventPayload {
            transacitons: self.transaction_buffer,
        };
        // Retrieving the parent after applying sync, because the latest event is likely
        // to be updated there.
        let other_parent = self
            .inner
            .peer_latest_event(from)
            .clone()
            .ok_or_else(|| ApplySyncError::UnknownPeer(from.clone()))?;
        self.inner.create_event(payload, other_parent.clone())?;
        self.transaction_buffer.clear();
        Ok(())
    }

    pub fn create_standalone_event(&mut self) -> Result<(), EventCreateError<PeerId>> {
        self.state_updated.notify_one();
        let payload = EventPayload {
            transacitons: self.transaction_buffer,
        };
        let self_parent = self
            .inner
            .peer_latest_event(self.inner.self_id())
            .expect("Peer must know itself")
            .clone();
        self.inner.create_event(payload, self_parent)?;
        Ok(())
    }
}

impl<TDataId, TPieceId> Stream for GraphWrapper<TDataId, TPieceId>
where
    TDataId: Serialize + Eq + std::hash::Hash + Debug + Clone,
    TPieceId: Serialize + Eq + std::hash::Hash + Debug + Clone,
{
    type Item = EventWrapper<EventPayload<TDataId, TPieceId>, PeerId>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let state_updated_notification = self.state_updated.notified();
        pin!(state_updated_notification);
        match state_updated_notification.poll(cx) {
            Poll::Ready(()) => {
                self.state_updated.notify_one();
                match self.inner.next_event() {
                    Some(event) => Poll::Ready(Some(event.clone())),
                    None => Poll::Pending,
                }
            }
            Poll::Pending => Poll::Pending,
        }
    }
}
