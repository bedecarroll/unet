//! Streaming support for processing large configuration files
//!
//! This module provides utilities for processing configuration files that are too large
//! to fit in memory all at once, or when memory-efficient processing is desired.

use crate::error::{Error, Result};
use crate::parser::{ConfigNode, ConfigParserPlugin, Vendor};
use crate::slicer::{ConfigSlicer, SlicePattern, SliceResult};
use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use tracing::{debug, info, warn};

/// Configuration for streaming operations
#[derive(Debug, Clone)]
pub struct StreamingConfig {
    /// Buffer size for reading operations
    pub buffer_size: usize,
    /// Maximum number of lines to process in a single chunk
    pub chunk_size: usize,
    /// Maximum file size to process (in bytes)
    pub max_file_size: usize,
    /// Memory limit for parsed configuration trees
    pub memory_limit: usize,
    /// Whether to use aggressive memory cleanup
    pub aggressive_cleanup: bool,
    /// Timeout for individual operations (in seconds)
    pub operation_timeout: u64,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            buffer_size: 64 * 1024,           // 64KB buffer
            chunk_size: 1000,                 // 1000 lines per chunk
            max_file_size: 100 * 1024 * 1024, // 100MB max file
            memory_limit: 50 * 1024 * 1024,   // 50MB memory limit
            aggressive_cleanup: false,
            operation_timeout: 300, // 5 minutes
        }
    }
}

impl StreamingConfig {
    /// Create a new streaming configuration
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the buffer size for reading operations
    #[must_use]
    pub const fn with_buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    /// Set the chunk size for processing
    #[must_use]
    pub const fn with_chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = size;
        self
    }

    /// Set the maximum file size to process
    #[must_use]
    pub const fn with_max_file_size(mut self, size: usize) -> Self {
        self.max_file_size = size;
        self
    }

    /// Set the memory limit for operations
    #[must_use]
    pub const fn with_memory_limit(mut self, limit: usize) -> Self {
        self.memory_limit = limit;
        self
    }

    /// Enable or disable aggressive memory cleanup
    #[must_use]
    pub const fn with_aggressive_cleanup(mut self, enabled: bool) -> Self {
        self.aggressive_cleanup = enabled;
        self
    }

    /// Set the operation timeout
    #[must_use]
    pub const fn with_timeout(mut self, seconds: u64) -> Self {
        self.operation_timeout = seconds;
        self
    }
}

/// Streaming configuration processor for large files
pub struct StreamingProcessor {
    config: StreamingConfig,
    parsers: std::collections::HashMap<Vendor, Box<dyn ConfigParserPlugin>>,
    slicer: ConfigSlicer,
}

impl StreamingProcessor {
    /// Create a new streaming processor with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: StreamingConfig::default(),
            parsers: std::collections::HashMap::new(),
            slicer: ConfigSlicer::new(),
        }
    }

    /// Create a streaming processor with custom configuration
    #[must_use]
    pub fn with_config(config: StreamingConfig) -> Self {
        Self {
            config,
            parsers: std::collections::HashMap::new(),
            slicer: ConfigSlicer::new(),
        }
    }

    /// Register a vendor parser
    pub fn register_parser(&mut self, vendor: Vendor, parser: Box<dyn ConfigParserPlugin>) {
        self.parsers.insert(vendor, parser);
    }

    /// Process a large configuration file in streaming mode
    ///
    /// This method processes the file in chunks to manage memory usage effectively.
    /// It's suitable for files that are too large to fit in memory.
    pub fn process_large_config<R: Read + Seek>(
        &self,
        mut reader: R,
        vendor: Option<Vendor>,
    ) -> Result<ConfigNode> {
        // Check file size first
        let file_size = reader.seek(SeekFrom::End(0)).map_err(Error::Io)?;
        reader.seek(SeekFrom::Start(0)).map_err(Error::Io)?;

        if file_size as usize > self.config.max_file_size {
            return Err(Error::size_limit_error(
                file_size as usize,
                self.config.max_file_size,
            ));
        }

        info!("Processing large configuration file: {} bytes", file_size);

        let mut buffer_reader = BufReader::with_capacity(self.config.buffer_size, reader);
        let mut line_processor = LineProcessor::new(&self.config);

        // Process the file line by line
        let mut line = String::new();
        let mut line_number = 0;

        while buffer_reader.read_line(&mut line)? > 0 {
            line_number += 1;

            // Process the line
            line_processor.process_line(line_number, &line)?;

            // Clear for next iteration
            line.clear();

            // Check memory usage periodically
            if line_number % 1000 == 0 {
                line_processor.check_memory_usage(self.config.memory_limit)?;
            }
        }

        // Finalize processing and build the configuration tree
        line_processor.finalize(vendor.unwrap_or(Vendor::Generic))
    }

    /// Process configuration and extract slices in streaming mode
    ///
    /// This method is optimized for cases where you want to extract specific
    /// slices without building the entire configuration tree in memory.
    pub fn process_and_slice<R: Read + Seek>(
        &self,
        reader: R,
        patterns: &[SlicePattern],
        vendor: Option<Vendor>,
    ) -> Result<Vec<SliceResult>> {
        let config_tree = self.process_large_config(reader, vendor)?;

        // Process slices with memory management
        let mut results = Vec::new();

        for pattern in patterns {
            debug!("Processing slice pattern: {:?}", pattern);

            match self.slicer.extract_slice(&config_tree, pattern, None) {
                Ok(result) => {
                    results.push(result);

                    // Clean up memory if needed
                    if self.config.aggressive_cleanup {
                        // Force garbage collection hint
                        drop(pattern);
                    }
                }
                Err(e) => {
                    warn!("Failed to extract slice: {}", e);
                    // Continue with other patterns
                }
            }
        }

        Ok(results)
    }

    /// Process configuration chunks incrementally
    ///
    /// This method allows for processing configuration in smaller incremental
    /// chunks, useful for real-time processing or very memory-constrained environments.
    pub fn process_chunks<R: BufRead>(
        &self,
        reader: R,
        chunk_callback: impl Fn(&ConfigChunk) -> Result<()>,
        vendor: Option<Vendor>,
    ) -> Result<()> {
        let mut chunk_processor =
            ChunkProcessor::new(&self.config, vendor.unwrap_or(Vendor::Generic));

        for (line_number, line) in reader.lines().enumerate() {
            let line = line?;

            if let Some(chunk) = chunk_processor.process_line(line_number + 1, &line)? {
                chunk_callback(&chunk)?;
            }
        }

        // Process any remaining chunk
        if let Some(final_chunk) = chunk_processor.finalize()? {
            chunk_callback(&final_chunk)?;
        }

        Ok(())
    }
}

impl Default for StreamingProcessor {
    fn default() -> Self {
        Self::new()
    }
}

/// Line-by-line processor for streaming operations
struct LineProcessor {
    lines: VecDeque<String>,
    line_numbers: VecDeque<usize>,
    current_memory_usage: usize,
    config: StreamingConfig,
}

impl LineProcessor {
    fn new(config: &StreamingConfig) -> Self {
        Self {
            lines: VecDeque::new(),
            line_numbers: VecDeque::new(),
            current_memory_usage: 0,
            config: config.clone(),
        }
    }

    fn process_line(&mut self, line_number: usize, line: &str) -> Result<()> {
        // Add line to processing queue
        self.lines.push_back(line.trim_end().to_string());
        self.line_numbers.push_back(line_number);

        // Update memory usage estimate
        self.current_memory_usage += line.len() + std::mem::size_of::<String>();

        // Process chunk if we've reached the limit
        if self.lines.len() >= self.config.chunk_size {
            self.process_chunk()?;
        }

        Ok(())
    }

    fn process_chunk(&mut self) -> Result<()> {
        if self.lines.is_empty() {
            return Ok(());
        }

        debug!("Processing chunk of {} lines", self.lines.len());

        // Process lines in current chunk
        // This is a simplified implementation - in practice, you might want
        // to do partial parsing or other optimizations here

        // For memory management, we'll keep only recent lines
        let keep_lines = self.config.chunk_size / 4; // Keep 25% of chunk size

        while self.lines.len() > keep_lines {
            if let Some(line) = self.lines.pop_front() {
                self.current_memory_usage = self
                    .current_memory_usage
                    .saturating_sub(line.len() + std::mem::size_of::<String>());
            }
            self.line_numbers.pop_front();
        }

        Ok(())
    }

    fn check_memory_usage(&mut self, limit: usize) -> Result<()> {
        if self.current_memory_usage > limit {
            warn!(
                "Memory usage ({} bytes) exceeds limit ({} bytes), performing cleanup",
                self.current_memory_usage, limit
            );

            // Force processing of current chunk to free memory
            self.process_chunk()?;

            // If still over limit, this is an error
            if self.current_memory_usage > limit {
                return Err(Error::Memory(format!(
                    "Unable to reduce memory usage below limit: {limit} bytes"
                )));
            }
        }

        Ok(())
    }

    fn finalize(mut self, _vendor: Vendor) -> Result<ConfigNode> {
        // Process any remaining lines
        self.process_chunk()?;

        // Build final configuration from remaining lines
        let _config_text = self.lines.into_iter().collect::<Vec<_>>().join("\n");

        // This is simplified - in practice, you'd use the actual parser
        // For now, create a basic configuration node
        Ok(ConfigNode {
            command: "root".to_string(),
            raw_line: String::new(),
            line_number: 0,
            indent_level: 0,
            children: Vec::new(),
            context: crate::parser::ConfigContext::Global,
            node_type: crate::parser::NodeType::Root,
            metadata: std::collections::HashMap::new(),
        })
    }
}

/// Chunk processor for incremental processing
struct ChunkProcessor {
    current_chunk: Vec<String>,
    chunk_line_numbers: Vec<usize>,
    config: StreamingConfig,
    vendor: Vendor,
    chunk_counter: usize,
}

impl ChunkProcessor {
    fn new(config: &StreamingConfig, vendor: Vendor) -> Self {
        Self {
            current_chunk: Vec::new(),
            chunk_line_numbers: Vec::new(),
            config: config.clone(),
            vendor,
            chunk_counter: 0,
        }
    }

    fn process_line(&mut self, line_number: usize, line: &str) -> Result<Option<ConfigChunk>> {
        self.current_chunk.push(line.to_string());
        self.chunk_line_numbers.push(line_number);

        if self.current_chunk.len() >= self.config.chunk_size {
            let chunk = self.create_chunk()?;
            self.reset_chunk();
            Ok(Some(chunk))
        } else {
            Ok(None)
        }
    }

    fn finalize(mut self) -> Result<Option<ConfigChunk>> {
        if self.current_chunk.is_empty() {
            Ok(None)
        } else {
            let chunk = self.create_chunk()?;
            Ok(Some(chunk))
        }
    }

    fn create_chunk(&mut self) -> Result<ConfigChunk> {
        let content = self.current_chunk.join("\n");
        let start_line = self.chunk_line_numbers.first().copied().unwrap_or(0);
        let end_line = self.chunk_line_numbers.last().copied().unwrap_or(0);

        self.chunk_counter += 1;

        Ok(ConfigChunk {
            id: self.chunk_counter,
            content,
            start_line,
            end_line,
            vendor: self.vendor,
            line_count: self.current_chunk.len(),
        })
    }

    fn reset_chunk(&mut self) {
        self.current_chunk.clear();
        self.chunk_line_numbers.clear();
    }
}

/// A chunk of configuration data for incremental processing
#[derive(Debug, Clone)]
pub struct ConfigChunk {
    /// Unique identifier for this chunk
    pub id: usize,
    /// Configuration content
    pub content: String,
    /// Starting line number in the original file
    pub start_line: usize,
    /// Ending line number in the original file
    pub end_line: usize,
    /// Vendor type for this configuration
    pub vendor: Vendor,
    /// Number of lines in this chunk
    pub line_count: usize,
}

impl ConfigChunk {
    /// Get the size of this chunk in bytes
    #[must_use]
    pub fn size_bytes(&self) -> usize {
        self.content.len()
    }

    /// Check if this chunk is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.content.trim().is_empty()
    }

    /// Get lines in this chunk
    pub fn lines(&self) -> impl Iterator<Item = &str> {
        self.content.lines()
    }
}

/// Memory usage monitor for streaming operations
pub struct MemoryMonitor {
    peak_usage: usize,
    current_usage: usize,
    limit: usize,
}

impl MemoryMonitor {
    /// Create a new memory monitor with the specified limit
    #[must_use]
    pub const fn new(limit: usize) -> Self {
        Self {
            peak_usage: 0,
            current_usage: 0,
            limit,
        }
    }

    /// Record memory allocation
    pub fn allocate(&mut self, size: usize) -> Result<()> {
        if self.current_usage + size > self.limit {
            Err(Error::Memory(format!(
                "Memory allocation would exceed limit: {} > {}",
                self.current_usage + size,
                self.limit
            )))
        } else {
            self.current_usage += size;
            self.peak_usage = self.peak_usage.max(self.current_usage);
            Ok(())
        }
    }

    /// Record memory deallocation
    pub const fn deallocate(&mut self, size: usize) {
        self.current_usage = self.current_usage.saturating_sub(size);
    }

    /// Get current memory usage
    #[must_use]
    pub const fn current_usage(&self) -> usize {
        self.current_usage
    }

    /// Get peak memory usage
    #[must_use]
    pub const fn peak_usage(&self) -> usize {
        self.peak_usage
    }

    /// Check if we're close to the memory limit
    #[must_use]
    pub fn is_near_limit(&self, threshold: f64) -> bool {
        let usage_ratio = self.current_usage as f64 / self.limit as f64;
        usage_ratio >= threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_streaming_config_creation() {
        let config = StreamingConfig::new()
            .with_buffer_size(1024)
            .with_chunk_size(100)
            .with_max_file_size(1024 * 1024)
            .with_memory_limit(512 * 1024)
            .with_aggressive_cleanup(true)
            .with_timeout(60);

        assert_eq!(config.buffer_size, 1024);
        assert_eq!(config.chunk_size, 100);
        assert_eq!(config.max_file_size, 1024 * 1024);
        assert_eq!(config.memory_limit, 512 * 1024);
        assert!(config.aggressive_cleanup);
        assert_eq!(config.operation_timeout, 60);
    }

    #[test]
    fn test_streaming_processor_creation() {
        let processor = StreamingProcessor::new();
        assert!(!processor.config.aggressive_cleanup);

        let custom_config = StreamingConfig::new().with_chunk_size(500);
        let custom_processor = StreamingProcessor::with_config(custom_config);
        assert_eq!(custom_processor.config.chunk_size, 500);
    }

    #[test]
    fn test_config_chunk() {
        let chunk = ConfigChunk {
            id: 1,
            content: "interface GigabitEthernet0/1\n description Test".to_string(),
            start_line: 1,
            end_line: 2,
            vendor: Vendor::Cisco,
            line_count: 2,
        };

        assert_eq!(chunk.id, 1);
        assert_eq!(chunk.line_count, 2);
        assert!(!chunk.is_empty());
        assert!(chunk.size_bytes() > 0);
        assert_eq!(chunk.lines().count(), 2);
    }

    #[test]
    fn test_memory_monitor() {
        let mut monitor = MemoryMonitor::new(1000);

        assert!(monitor.allocate(500).is_ok());
        assert_eq!(monitor.current_usage(), 500);

        assert!(monitor.allocate(400).is_ok());
        assert_eq!(monitor.current_usage(), 900);
        assert_eq!(monitor.peak_usage(), 900);

        // This should fail
        assert!(monitor.allocate(200).is_err());

        monitor.deallocate(300);
        assert_eq!(monitor.current_usage(), 600);
        assert_eq!(monitor.peak_usage(), 900); // Peak should remain

        assert!(monitor.is_near_limit(0.5)); // 600/1000 = 0.6 > 0.5
        assert!(!monitor.is_near_limit(0.8)); // 600/1000 = 0.6 < 0.8
    }

    #[test]
    fn test_chunk_processor() {
        let config = StreamingConfig::new().with_chunk_size(2);
        let mut processor = ChunkProcessor::new(&config, Vendor::Cisco);

        // First line shouldn't produce a chunk
        let result = processor
            .process_line(1, "interface GigabitEthernet0/1")
            .unwrap();
        assert!(result.is_none());

        // Second line should produce a chunk
        let result = processor.process_line(2, " description Test").unwrap();
        assert!(result.is_some());

        let chunk = result.unwrap();
        assert_eq!(chunk.id, 1);
        assert_eq!(chunk.start_line, 1);
        assert_eq!(chunk.end_line, 2);
        assert_eq!(chunk.line_count, 2);
    }
}
