use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

use crate::cache::CacheManager;
use crate::error::AppError;

/// Cache invalidation event types.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CacheInvalidationEvent {
    /// Invalidate all caches.
    All,
    /// Invalidate item-related caches.
    Items,
    /// Invalidate location-related caches.
    Locations,
    /// Invalidate dashboard statistics cache.
    Dashboard,
    /// Invalidate inventory-related caches.
    Inventory,
    /// Invalidate order-related caches (purchase/sales).
    Orders,
    /// Invalidate contract-related caches.
    Contracts,
    /// Invalidate supplier/customer caches.
    Partners,
}

/// A sender for cache invalidation events.
#[derive(Clone)]
pub struct CacheInvalidator {
    sender: mpsc::UnboundedSender<CacheInvalidationEvent>,
}

impl CacheInvalidator {
    /// Create a new cache invalidator with a background worker.
    pub fn new(cache_manager: CacheManager) -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                match event {
                    CacheInvalidationEvent::All => cache_manager.invalidate_all().await,
                    CacheInvalidationEvent::Items => cache_manager.invalidate_items().await,
                    CacheInvalidationEvent::Locations => cache_manager.invalidate_locations().await,
                    CacheInvalidationEvent::Dashboard => cache_manager.invalidate_dashboard().await,
                    CacheInvalidationEvent::Inventory => cache_manager.invalidate_dashboard().await,
                    CacheInvalidationEvent::Orders => cache_manager.invalidate_dashboard().await,
                    CacheInvalidationEvent::Contracts => cache_manager.invalidate_dashboard().await,
                    CacheInvalidationEvent::Partners => cache_manager.invalidate_dashboard().await,
                }
            }
        });

        Self { sender: tx }
    }

    /// Emit a cache invalidation event (non-blocking).
    pub fn emit(&self, event: CacheInvalidationEvent) -> Result<(), AppError> {
        self.sender
            .send(event)
            .map_err(|_| AppError::Internal("Cache invalidator channel closed".into()))
    }

    /// Emit multiple cache invalidation events.
    pub fn emit_all(&self, events: Vec<CacheInvalidationEvent>) -> Result<(), AppError> {
        for event in events {
            self.emit(event)?;
        }
        Ok(())
    }
}

/// Extension trait for services to easily emit cache invalidation events.
pub trait CacheInvalidate {
    fn invalidate_items(&self) -> Result<(), AppError>;
    fn invalidate_inventory(&self) -> Result<(), AppError>;
    fn invalidate_orders(&self) -> Result<(), AppError>;
    fn invalidate_contracts(&self) -> Result<(), AppError>;
    fn invalidate_partners(&self) -> Result<(), AppError>;
    fn invalidate_all(&self) -> Result<(), AppError>;
}

impl CacheInvalidate for CacheInvalidator {
    fn invalidate_items(&self) -> Result<(), AppError> {
        self.emit(CacheInvalidationEvent::Items)
    }

    fn invalidate_inventory(&self) -> Result<(), AppError> {
        self.emit(CacheInvalidationEvent::Inventory)
    }

    fn invalidate_orders(&self) -> Result<(), AppError> {
        self.emit(CacheInvalidationEvent::Orders)
    }

    fn invalidate_contracts(&self) -> Result<(), AppError> {
        self.emit(CacheInvalidationEvent::Contracts)
    }

    fn invalidate_partners(&self) -> Result<(), AppError> {
        self.emit(CacheInvalidationEvent::Partners)
    }

    fn invalidate_all(&self) -> Result<(), AppError> {
        self.emit(CacheInvalidationEvent::All)
    }
}

/// Registry for typed cache invalidation.
/// Allows services to register which cache keys they invalidate for specific operations.
#[derive(Default, Clone)]
pub struct CacheInvalidationRegistry {
    rules: Arc<RwLock<HashMap<String, Vec<CacheInvalidationEvent>>>>,
}

impl CacheInvalidationRegistry {
    /// Create a new registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register invalidation rules for an operation.
    ///
    /// The key format is "entity:operation" e.g., "item:create", "inventory:update"
    pub async fn register(&self, key: &str, events: Vec<CacheInvalidationEvent>) {
        let mut rules = self.rules.write().await;
        rules.insert(key.to_string(), events);
    }

    /// Get invalidation events for an operation.
    pub async fn get_events(&self, key: &str) -> Vec<CacheInvalidationEvent> {
        let rules = self.rules.read().await;
        rules.get(key).cloned().unwrap_or_default()
    }

    /// Emit invalidation events for an operation.
    pub async fn invalidate(
        &self,
        invalidator: &CacheInvalidator,
        key: &str,
    ) -> Result<(), AppError> {
        let events = self.get_events(key).await;
        for event in events {
            invalidator.emit(event)?;
        }
        Ok(())
    }
}

/// Initialize default invalidation rules for common operations.
pub fn init_default_invalidation_rules(registry: &CacheInvalidationRegistry) {
    let registry = registry.clone();
    tokio::spawn(async move {
        // Item operations
        registry
            .register(
                "item:create",
                vec![CacheInvalidationEvent::Items, CacheInvalidationEvent::Dashboard],
            )
            .await;
        registry
            .register(
                "item:update",
                vec![CacheInvalidationEvent::Items, CacheInvalidationEvent::Dashboard],
            )
            .await;
        registry
            .register(
                "item:delete",
                vec![CacheInvalidationEvent::Items, CacheInvalidationEvent::Dashboard],
            )
            .await;

        // Inventory operations
        registry
            .register(
                "inventory:inbound",
                vec![CacheInvalidationEvent::Inventory, CacheInvalidationEvent::Dashboard],
            )
            .await;
        registry
            .register(
                "inventory:outbound",
                vec![CacheInvalidationEvent::Inventory, CacheInvalidationEvent::Dashboard],
            )
            .await;
        registry
            .register(
                "inventory:check",
                vec![CacheInvalidationEvent::Inventory, CacheInvalidationEvent::Dashboard],
            )
            .await;
        registry
            .register(
                "inventory:location",
                vec![
                    CacheInvalidationEvent::Inventory,
                    CacheInvalidationEvent::Locations,
                    CacheInvalidationEvent::Dashboard,
                ],
            )
            .await;

        // Order operations
        registry
            .register(
                "order:purchase",
                vec![CacheInvalidationEvent::Orders, CacheInvalidationEvent::Dashboard],
            )
            .await;
        registry
            .register(
                "order:sales",
                vec![CacheInvalidationEvent::Orders, CacheInvalidationEvent::Dashboard],
            )
            .await;

        // Contract operations
        registry
            .register(
                "contract:create",
                vec![CacheInvalidationEvent::Contracts, CacheInvalidationEvent::Dashboard],
            )
            .await;
        registry
            .register(
                "contract:update",
                vec![CacheInvalidationEvent::Contracts, CacheInvalidationEvent::Dashboard],
            )
            .await;

        // Partner operations
        registry
            .register(
                "partner:supplier",
                vec![CacheInvalidationEvent::Partners, CacheInvalidationEvent::Dashboard],
            )
            .await;
        registry
            .register(
                "partner:customer",
                vec![CacheInvalidationEvent::Partners, CacheInvalidationEvent::Dashboard],
            )
            .await;
    });
}
