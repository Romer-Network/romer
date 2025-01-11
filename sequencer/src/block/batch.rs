use tokio::sync::mpsc;
use tokio::time::{self, Duration, Instant};
use crate::fix::types::ValidatedMessage;
use std::sync::Arc;
use parking_lot::Mutex;

/// Represents a collection of FIX messages ready to be formed into a block
#[derive(Debug, Clone)]
pub struct MessageBatch {
    /// The messages in this batch
    pub messages: Vec<ValidatedMessage>,
    /// When this batch started collecting messages
    pub start_time: Instant,
    /// When this batch was finalized
    pub end_time: Instant,
    /// Sequence number for this batch
    pub sequence: u64,
}

/// Manages the collection of FIX messages into batches
pub struct BatchManager {
    /// Currently accumulating messages
    current_batch: Arc<Mutex<Vec<ValidatedMessage>>>,
    /// When the current batch started
    batch_start: Arc<Mutex<Instant>>,
    /// Channel for sending completed batches
    batch_sender: mpsc::Sender<MessageBatch>,
    /// Maximum messages per batch
    max_batch_size: usize,
    /// Maximum time to wait for a batch
    max_batch_time: Duration,
    /// Current batch sequence number
    sequence: Arc<Mutex<u64>>,
}

impl BatchManager {
    /// Create a new batch manager with specified limits
    pub fn new(
        batch_sender: mpsc::Sender<MessageBatch>,
        max_batch_size: usize,
        max_batch_time: Duration,
    ) -> Self {
        Self {
            current_batch: Arc::new(Mutex::new(Vec::with_capacity(max_batch_size))),
            batch_start: Arc::new(Mutex::new(Instant::now())),
            batch_sender,
            max_batch_size,
            max_batch_time,
            sequence: Arc::new(Mutex::new(0)),
        }
    }

    /// Start the batch management process
    pub async fn run(&self) {
        let mut interval = time::interval(Duration::from_millis(10));

        loop {
            interval.tick().await;
            self.check_batch().await;
        }
    }

    /// Add a new message to the current batch
    pub async fn add_message(&self, message: ValidatedMessage) {
        let should_flush = {
            let mut batch = self.current_batch.lock();
            batch.push(message);
            batch.len() >= self.max_batch_size
        };

        if should_flush {
            self.flush_batch().await;
        }
    }

    /// Check if the current batch should be flushed based on time
    async fn check_batch(&self) {
        let start = *self.batch_start.lock();
        if start.elapsed() >= self.max_batch_time {
            self.flush_batch().await;
        }
    }

    /// Flush the current batch and start a new one
    async fn flush_batch(&self) {
        let mut batch = self.current_batch.lock();
        // Only create a batch if we have messages
        if !batch.is_empty() {
            let messages = std::mem::replace(batch.deref_mut(), Vec::with_capacity(self.max_batch_size));
            let start_time = *self.batch_start.lock();
            let end_time = Instant::now();
            
            // Get sequence number and increment
            let sequence = {
                let mut seq = self.sequence.lock();
                let current = *seq;
                *seq += 1;
                current
            };

            let message_batch = MessageBatch {
                messages,
                start_time,
                end_time,
                sequence,
            };

            // Send the batch, ignoring errors if receiver is closed
            let _ = self.batch_sender.send(message_batch).await;
        }

        // Reset the batch start time
        *self.batch_start.lock() = Instant::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fix::types::MessageType;
    use tokio::time::sleep;

    async fn create_test_message() -> ValidatedMessage {
        // Create a simple test message
        ValidatedMessage {
            msg_type: MessageType::NewOrderSingle,
            message: fefix::tagvalue::Message::new(fefix::Dictionary::fix42()),
            sender_comp_id: "SENDER".to_string(),
            target_comp_id: "TARGET".to_string(),
            msg_seq_num: 1,
        }
    }

    #[tokio::test]
    async fn test_batch_size_trigger() {
        let (sender, mut receiver) = mpsc::channel(100);
        let manager = BatchManager::new(sender, 2, Duration::from_secs(1));
        
        // Add two messages (should trigger size-based flush)
        manager.add_message(create_test_message().await).await;
        manager.add_message(create_test_message().await).await;

        // Should receive a batch
        let batch = receiver.recv().await.unwrap();
        assert_eq!(batch.messages.len(), 2);
        assert_eq!(batch.sequence, 0);
    }

    #[tokio::test]
    async fn test_batch_time_trigger() {
        let (sender, mut receiver) = mpsc::channel(100);
        let manager = BatchManager::new(sender, 10, Duration::from_millis(100));
        
        // Start the batch manager
        let manager_clone = manager.clone();
        tokio::spawn(async move {
            manager_clone.run().await;
        });

        // Add one message and wait
        manager.add_message(create_test_message().await).await;
        sleep(Duration::from_millis(150)).await;

        // Should receive a batch due to time
        let batch = receiver.recv().await.unwrap();
        assert_eq!(batch.messages.len(), 1);
    }
}