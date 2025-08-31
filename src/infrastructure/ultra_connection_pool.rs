use anyhow::Result;
use crossbeam::queue::SegQueue;
use dashmap::DashMap;
use flume::{Receiver, Sender};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::sync::Semaphore;
use tokio::time::interval;
use tracing::{info, warn};
use uuid::Uuid;

// Ultra-fast connection pool with zero-allocation design
pub struct UltraConnectionPool {
    // Lock-free connection tracking
    active_connections: Arc<DashMap<String, ConnectionInfo>>,
    connection_queue: Arc<SegQueue<String>>,
    
    // Ultra-fast semaphore for rate limiting
    connection_semaphore: Arc<Semaphore>,
    
    // Performance counters
    total_connections: AtomicU64,
    peak_connections: AtomicU64,
    connection_reuses: AtomicU64,
    
    // Connection pool settings
    max_connections: usize,
    connection_timeout: Duration,
    
    // Ultra-fast cleanup
    cleanup_queue: Arc<SegQueue<String>>,
}

pub struct ConnectionInfo {
    pub id: String,
    pub created_at: Instant,
    pub last_activity: Instant,
    pub message_count: AtomicU64,
    pub bytes_sent: AtomicU64,
    pub bytes_received: AtomicU64,
}

#[derive(Debug, Clone)]
pub struct PoolMetrics {
    pub active_connections: usize,
    pub total_connections: u64,
    pub peak_connections: u64,
    pub connection_reuses: u64,
    pub available_slots: usize,
    pub cleanup_queue_size: usize,
}

impl UltraConnectionPool {
    pub fn new(max_connections: usize, connection_timeout: Duration) -> Self {
        let pool = Self {
            active_connections: Arc::new(DashMap::with_capacity(max_connections)),
            connection_queue: Arc::new(SegQueue::new()),
            connection_semaphore: Arc::new(Semaphore::new(max_connections)),
            total_connections: AtomicU64::new(0),
            peak_connections: AtomicU64::new(0),
            connection_reuses: AtomicU64::new(0),
            max_connections,
            connection_timeout,
            cleanup_queue: Arc::new(SegQueue::new()),
        };
        
        // Start ultra-fast cleanup task
        pool.start_ultra_cleanup();
        
        pool
    }
    
    // Ultra-fast connection acquisition
    pub async fn acquire_connection(&self) -> Result<ConnectionSlot<'_>> {
        // Try to acquire semaphore permit
        let permit = self.connection_semaphore
            .acquire()
            .await
            .map_err(|_| anyhow::anyhow!("Connection pool exhausted"))?;
        
        // Generate ultra-fast connection ID
        let connection_id = self.generate_fast_id();
        
        // Create connection info
        let connection_info = ConnectionInfo {
            id: connection_id.clone(),
            created_at: Instant::now(),
            last_activity: Instant::now(),
            message_count: AtomicU64::new(0),
            bytes_sent: AtomicU64::new(0),
            bytes_received: AtomicU64::new(0),
        };
        
        // Insert into active connections
        self.active_connections.insert(connection_id.clone(), connection_info);
        
        // Update counters
        let current = self.total_connections.fetch_add(1, Ordering::Relaxed) + 1;
        let peak = self.peak_connections.load(Ordering::Relaxed);
        if current > peak {
            self.peak_connections.store(current, Ordering::Relaxed);
        }
        
        Ok(ConnectionSlot {
            id: connection_id,
            pool: self.clone(),
            _permit: permit,
        })
    }
    
    // Ultra-fast connection release
    pub fn release_connection(&self, connection_id: &str) {
        // Remove from active connections
        if let Some((_, connection_info)) = self.active_connections.remove(connection_id) {
            // Add to cleanup queue for background processing
            self.cleanup_queue.push(connection_id.to_string());
            
            // Update reuse counter if connection was active for a while
            if connection_info.created_at.elapsed() > Duration::from_secs(1) {
                self.connection_reuses.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
    
    // Update connection activity (ultra-fast)
    pub fn update_activity(&self, connection_id: &str, bytes_sent: u64, bytes_received: u64) {
        if let Some(mut connection) = self.active_connections.get_mut(connection_id) {
            connection.last_activity = Instant::now();
            connection.message_count.fetch_add(1, Ordering::Relaxed);
            connection.bytes_sent.fetch_add(bytes_sent, Ordering::Relaxed);
            connection.bytes_received.fetch_add(bytes_received, Ordering::Relaxed);
        }
    }
    
    // Ultra-fast ID generation (faster than UUID)
    fn generate_fast_id(&self) -> String {
        // Use atomic counter + timestamp for ultra-fast unique IDs
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        
        format!("{:x}{:x}", timestamp, counter)
    }
    
    // Get ultra-performance metrics
    pub fn get_metrics(&self) -> PoolMetrics {
        PoolMetrics {
            active_connections: self.active_connections.len(),
            total_connections: self.total_connections.load(Ordering::Relaxed),
            peak_connections: self.peak_connections.load(Ordering::Relaxed),
            connection_reuses: self.connection_reuses.load(Ordering::Relaxed),
            available_slots: self.connection_semaphore.available_permits(),
            cleanup_queue_size: self.cleanup_queue.len(),
        }
    }
    
    // Start ultra-fast background cleanup
    fn start_ultra_cleanup(&self) {
        let active_connections = self.active_connections.clone();
        let cleanup_queue = self.cleanup_queue.clone();
        let connection_timeout = self.connection_timeout;
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(5)); // Fast cleanup every 5s
            
            loop {
                interval.tick().await;
                
                let now = Instant::now();
                let mut cleaned_up = 0;
                
                // Process cleanup queue
                while let Some(connection_id) = cleanup_queue.pop() {
                    // Connection already removed, just count it
                    cleaned_up += 1;
                }
                
                // Clean up timed-out connections
                let mut timed_out_connections = Vec::new();
                
                for connection_ref in active_connections.iter() {
                    let connection = connection_ref.value();
                    if now.duration_since(connection.last_activity) > connection_timeout {
                        timed_out_connections.push(connection.id.clone());
                    }
                }
                
                for connection_id in timed_out_connections {
                    if active_connections.remove(&connection_id).is_some() {
                        cleaned_up += 1;
                    }
                }
                
                if cleaned_up > 0 {
                    info!("Ultra-fast cleanup: {} connections cleaned", cleaned_up);
                }
            }
        });
    }
}

impl Clone for UltraConnectionPool {
    fn clone(&self) -> Self {
        Self {
            active_connections: self.active_connections.clone(),
            connection_queue: self.connection_queue.clone(),
            connection_semaphore: self.connection_semaphore.clone(),
            total_connections: AtomicU64::new(0),
            peak_connections: AtomicU64::new(0),
            connection_reuses: AtomicU64::new(0),
            max_connections: self.max_connections,
            connection_timeout: self.connection_timeout,
            cleanup_queue: self.cleanup_queue.clone(),
        }
    }
}

// Connection slot with automatic cleanup
pub struct ConnectionSlot<'a> {
    pub id: String,
    pool: UltraConnectionPool,
    _permit: tokio::sync::SemaphorePermit<'a>,
}

impl ConnectionSlot<'_> {
    pub fn update_activity(&self, bytes_sent: u64, bytes_received: u64) {
        self.pool.update_activity(&self.id, bytes_sent, bytes_received);
    }
}

impl Drop for ConnectionSlot<'_> {
    fn drop(&mut self) {
        self.pool.release_connection(&self.id);
    }
}