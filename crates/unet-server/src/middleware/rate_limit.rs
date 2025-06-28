//! Enhanced rate limiting middleware with DOS protection

use axum::{
    extract::{ConnectInfo, Request},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use dashmap::DashMap;
use std::{
    net::SocketAddr,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};
use tokio::time::interval;
use tracing::{debug, warn};

/// Rate limiting bucket for tracking request rates
#[derive(Debug)]
struct RateLimitBucket {
    /// Number of tokens currently in the bucket
    tokens: AtomicU64,
    /// Last time the bucket was refilled
    last_refill: tokio::sync::Mutex<Instant>,
    /// Maximum number of tokens in the bucket
    capacity: u64,
    /// Rate at which tokens are refilled (per second)
    refill_rate: u64,
}

impl RateLimitBucket {
    fn new(capacity: u64, refill_rate: u64) -> Self {
        Self {
            tokens: AtomicU64::new(capacity),
            last_refill: tokio::sync::Mutex::new(Instant::now()),
            capacity,
            refill_rate,
        }
    }

    /// Try to consume a token from the bucket
    async fn try_consume(&self, tokens: u64) -> bool {
        self.refill().await;

        let current_tokens = self.tokens.load(Ordering::Acquire);
        if current_tokens >= tokens {
            let remaining = current_tokens - tokens;
            // Use compare_exchange to avoid race conditions
            self.tokens
                .compare_exchange(
                    current_tokens,
                    remaining,
                    Ordering::Release,
                    Ordering::Relaxed,
                )
                .is_ok()
        } else {
            false
        }
    }

    /// Refill tokens based on elapsed time
    async fn refill(&self) {
        let mut last_refill = self.last_refill.lock().await;
        let now = Instant::now();
        let elapsed = now.duration_since(*last_refill);

        if elapsed >= Duration::from_secs(1) {
            let tokens_to_add = (elapsed.as_secs() * self.refill_rate).min(self.capacity);
            let current_tokens = self.tokens.load(Ordering::Acquire);
            let new_tokens = (current_tokens + tokens_to_add).min(self.capacity);

            self.tokens.store(new_tokens, Ordering::Release);
            *last_refill = now;
        }
    }

    /// Get remaining tokens
    fn remaining_tokens(&self) -> u64 {
        self.tokens.load(Ordering::Acquire)
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum requests per window
    pub max_requests: u64,
    /// Rate of token refill per second
    pub refill_rate: u64,
    /// Enable rate limiting by IP address
    pub per_ip: bool,
    /// Enable rate limiting by user ID (requires authentication)
    pub per_user: bool,
    /// Enable global rate limiting
    pub global: bool,
    /// Burst allowance (bucket capacity)
    pub burst_size: u64,
    /// Whitelist of IP addresses that bypass rate limiting
    pub whitelist: Vec<std::net::IpAddr>,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,
            refill_rate: 10, // 10 requests per second
            per_ip: true,
            per_user: false,
            global: true,
            burst_size: 50,
            whitelist: vec!["127.0.0.1".parse().unwrap(), "::1".parse().unwrap()],
        }
    }
}

/// Enhanced rate limiter with DOS protection
#[derive(Debug)]
pub struct EnhancedRateLimiter {
    /// Per-IP rate limiting buckets
    ip_buckets: DashMap<std::net::IpAddr, Arc<RateLimitBucket>>,
    /// Per-user rate limiting buckets
    user_buckets: DashMap<String, Arc<RateLimitBucket>>,
    /// Global rate limiting bucket
    global_bucket: Arc<RateLimitBucket>,
    /// Configuration
    config: RateLimitConfig,
    /// Suspicious IP tracking
    suspicious_ips: DashMap<std::net::IpAddr, SuspiciousActivity>,
}

/// Tracking for suspicious activity
#[derive(Debug)]
struct SuspiciousActivity {
    /// Number of rate limit violations
    violations: AtomicU64,
    /// First violation time
    first_violation: tokio::sync::Mutex<Option<Instant>>,
    /// Last violation time
    last_violation: tokio::sync::Mutex<Instant>,
    /// Whether IP is temporarily blocked
    blocked_until: tokio::sync::Mutex<Option<Instant>>,
}

impl SuspiciousActivity {
    fn new() -> Self {
        Self {
            violations: AtomicU64::new(0),
            first_violation: tokio::sync::Mutex::new(None),
            last_violation: tokio::sync::Mutex::new(Instant::now()),
            blocked_until: tokio::sync::Mutex::new(None),
        }
    }

    async fn record_violation(&self) {
        let violations = self.violations.fetch_add(1, Ordering::Relaxed) + 1;
        let now = Instant::now();

        *self.last_violation.lock().await = now;

        if self.first_violation.lock().await.is_none() {
            *self.first_violation.lock().await = Some(now);
        }

        // Block IP for increasing durations based on violations
        let block_duration = match violations {
            1..=5 => None,
            6..=10 => Some(Duration::from_secs(60)), // 1 minute
            11..=20 => Some(Duration::from_secs(300)), // 5 minutes
            21..=50 => Some(Duration::from_secs(900)), // 15 minutes
            _ => Some(Duration::from_secs(3600)),    // 1 hour
        };

        if let Some(duration) = block_duration {
            *self.blocked_until.lock().await = Some(now + duration);
            warn!("IP temporarily blocked due to {} violations", violations);
        }
    }

    async fn is_blocked(&self) -> bool {
        let blocked_until = self.blocked_until.lock().await;
        if let Some(until) = *blocked_until {
            Instant::now() < until
        } else {
            false
        }
    }
}

impl EnhancedRateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        let global_bucket = Arc::new(RateLimitBucket::new(
            config.burst_size * 10, // Global bucket is larger
            config.refill_rate * 10,
        ));

        Self {
            ip_buckets: DashMap::new(),
            user_buckets: DashMap::new(),
            global_bucket,
            config,
            suspicious_ips: DashMap::new(),
        }
    }

    /// Check if request should be allowed
    pub async fn check_request(
        &self,
        ip: std::net::IpAddr,
        user_id: Option<&str>,
    ) -> RateLimitResult {
        // Check if IP is whitelisted
        if self.config.whitelist.contains(&ip) {
            return RateLimitResult::Allowed;
        }

        // Check if IP is blocked due to suspicious activity
        if let Some(activity) = self.suspicious_ips.get(&ip) {
            if activity.is_blocked().await {
                return RateLimitResult::Blocked(
                    "IP temporarily blocked due to suspicious activity",
                );
            }
        }

        // Check global rate limit
        if self.config.global {
            if !self.global_bucket.try_consume(1).await {
                warn!("Global rate limit exceeded");
                return RateLimitResult::RateLimited("Global rate limit exceeded");
            }
        }

        // Check per-IP rate limit
        if self.config.per_ip {
            let bucket = self
                .ip_buckets
                .entry(ip)
                .or_insert_with(|| {
                    Arc::new(RateLimitBucket::new(
                        self.config.burst_size,
                        self.config.refill_rate,
                    ))
                })
                .clone();

            if !bucket.try_consume(1).await {
                // Record violation for suspicious activity tracking
                let activity = self
                    .suspicious_ips
                    .entry(ip)
                    .or_insert_with(|| SuspiciousActivity::new());
                activity.record_violation().await;

                debug!("IP rate limit exceeded for {}", ip);
                return RateLimitResult::RateLimited("IP rate limit exceeded");
            }
        }

        // Check per-user rate limit
        if self.config.per_user {
            if let Some(user_id) = user_id {
                let bucket = self
                    .user_buckets
                    .entry(user_id.to_string())
                    .or_insert_with(|| {
                        Arc::new(RateLimitBucket::new(
                            self.config.burst_size,
                            self.config.refill_rate,
                        ))
                    })
                    .clone();

                if !bucket.try_consume(1).await {
                    debug!("User rate limit exceeded for {}", user_id);
                    return RateLimitResult::RateLimited("User rate limit exceeded");
                }
            }
        }

        RateLimitResult::Allowed
    }

    /// Start cleanup task to remove old entries
    pub fn start_cleanup_task(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(300)); // Cleanup every 5 minutes

            loop {
                interval.tick().await;

                // Clean up old IP buckets
                self.ip_buckets
                    .retain(|_, bucket| bucket.remaining_tokens() < bucket.capacity / 2);

                // Clean up old user buckets
                self.user_buckets
                    .retain(|_, bucket| bucket.remaining_tokens() < bucket.capacity / 2);

                // Clean up old suspicious activity records
                let now = Instant::now();
                self.suspicious_ips.retain(|_, activity| {
                    // Keep records for recent violations (last 24 hours)
                    let last_violation = activity.last_violation.try_lock();
                    if let Ok(last) = last_violation {
                        now.duration_since(*last) < Duration::from_secs(86400)
                    } else {
                        true // Keep if we can't acquire lock
                    }
                });

                debug!("Rate limiter cleanup completed");
            }
        });
    }

    /// Get rate limit status for debugging
    pub async fn get_status(&self, ip: std::net::IpAddr, user_id: Option<&str>) -> RateLimitStatus {
        let ip_tokens = if let Some(bucket) = self.ip_buckets.get(&ip) {
            Some(bucket.remaining_tokens())
        } else {
            None
        };

        let user_tokens = if let Some(user_id) = user_id {
            if let Some(bucket) = self.user_buckets.get(user_id) {
                Some(bucket.remaining_tokens())
            } else {
                None
            }
        } else {
            None
        };

        let global_tokens = self.global_bucket.remaining_tokens();

        let is_blocked = if let Some(activity) = self.suspicious_ips.get(&ip) {
            activity.is_blocked().await
        } else {
            false
        };

        RateLimitStatus {
            ip_tokens,
            user_tokens,
            global_tokens,
            is_blocked,
        }
    }
}

/// Rate limit check result
#[derive(Debug)]
pub enum RateLimitResult {
    Allowed,
    RateLimited(&'static str),
    Blocked(&'static str),
}

/// Rate limit status for debugging
#[derive(Debug)]
pub struct RateLimitStatus {
    pub ip_tokens: Option<u64>,
    pub user_tokens: Option<u64>,
    pub global_tokens: u64,
    pub is_blocked: bool,
}

/// Rate limiting middleware
pub async fn rate_limit_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Create default rate limiter (in production, this would be injected)
    static RATE_LIMITER: tokio::sync::OnceCell<Arc<EnhancedRateLimiter>> =
        tokio::sync::OnceCell::const_new();
    let rate_limiter = RATE_LIMITER
        .get_or_init(|| async {
            let config = RateLimitConfig::default();
            let limiter = Arc::new(EnhancedRateLimiter::new(config));
            limiter.clone().start_cleanup_task();
            limiter
        })
        .await;

    let ip = addr.ip();

    // Extract user ID from request if authenticated
    let user_id = extract_user_id_from_headers(&headers);

    // Check rate limits
    match rate_limiter.check_request(ip, user_id.as_deref()).await {
        RateLimitResult::Allowed => {
            // Add rate limit headers to response
            let mut response = next.run(request).await;
            add_rate_limit_headers(&mut response, rate_limiter.as_ref(), ip, user_id.as_deref())
                .await;
            Ok(response)
        }
        RateLimitResult::RateLimited(reason) => {
            warn!("Rate limit exceeded for {}: {}", ip, reason);
            Err(StatusCode::TOO_MANY_REQUESTS)
        }
        RateLimitResult::Blocked(reason) => {
            warn!("Request blocked for {}: {}", ip, reason);
            Err(StatusCode::FORBIDDEN)
        }
    }
}

/// Extract user ID from request headers (if authenticated)
fn extract_user_id_from_headers(headers: &HeaderMap) -> Option<String> {
    // In a real implementation, you would decode the JWT token or check session
    // For now, return None to indicate no user authentication
    None
}

/// Add rate limit information to response headers
async fn add_rate_limit_headers(
    response: &mut Response,
    rate_limiter: &EnhancedRateLimiter,
    ip: std::net::IpAddr,
    user_id: Option<&str>,
) {
    let status = rate_limiter.get_status(ip, user_id).await;
    let headers = response.headers_mut();

    // Add standard rate limit headers
    headers.insert(
        "X-RateLimit-Limit",
        format!("{}", rate_limiter.config.max_requests)
            .parse()
            .unwrap(),
    );

    if let Some(ip_tokens) = status.ip_tokens {
        headers.insert(
            "X-RateLimit-Remaining",
            format!("{}", ip_tokens).parse().unwrap(),
        );
    }

    headers.insert(
        "X-RateLimit-Reset",
        format!("{}", chrono::Utc::now().timestamp() + 60)
            .parse()
            .unwrap(),
    );

    if status.is_blocked {
        headers.insert("X-RateLimit-Blocked", "true".parse().unwrap());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_rate_limit_bucket() {
        let bucket = RateLimitBucket::new(5, 2); // 5 tokens, refill 2 per second

        // Should allow initial requests
        assert!(bucket.try_consume(1).await);
        assert!(bucket.try_consume(2).await);
        assert!(bucket.try_consume(2).await);

        // Should block when empty
        assert!(!bucket.try_consume(1).await);

        // Should refill after time passes
        sleep(Duration::from_secs(1)).await;
        bucket.refill().await;
        assert!(bucket.try_consume(1).await);
    }

    #[tokio::test]
    async fn test_enhanced_rate_limiter() {
        let config = RateLimitConfig {
            max_requests: 3,
            refill_rate: 1,
            burst_size: 3,
            per_ip: true,
            per_user: false,
            global: false,
            whitelist: vec![],
        };

        let limiter = EnhancedRateLimiter::new(config);
        let ip = "192.168.1.1".parse().unwrap();

        // Should allow initial requests
        assert!(matches!(
            limiter.check_request(ip, None).await,
            RateLimitResult::Allowed
        ));
        assert!(matches!(
            limiter.check_request(ip, None).await,
            RateLimitResult::Allowed
        ));
        assert!(matches!(
            limiter.check_request(ip, None).await,
            RateLimitResult::Allowed
        ));

        // Should block further requests
        assert!(matches!(
            limiter.check_request(ip, None).await,
            RateLimitResult::RateLimited(_)
        ));
    }

    #[tokio::test]
    async fn test_whitelist() {
        let config = RateLimitConfig {
            max_requests: 1,
            refill_rate: 1,
            burst_size: 1,
            per_ip: true,
            per_user: false,
            global: false,
            whitelist: vec!["127.0.0.1".parse().unwrap()],
        };

        let limiter = EnhancedRateLimiter::new(config);
        let whitelisted_ip = "127.0.0.1".parse().unwrap();
        let regular_ip = "192.168.1.1".parse().unwrap();

        // Whitelisted IP should always be allowed
        for _ in 0..10 {
            assert!(matches!(
                limiter.check_request(whitelisted_ip, None).await,
                RateLimitResult::Allowed
            ));
        }

        // Regular IP should be rate limited
        assert!(matches!(
            limiter.check_request(regular_ip, None).await,
            RateLimitResult::Allowed
        ));
        assert!(matches!(
            limiter.check_request(regular_ip, None).await,
            RateLimitResult::RateLimited(_)
        ));
    }
}
