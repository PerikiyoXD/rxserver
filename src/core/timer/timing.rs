// SPDX-License-Identifier: Apache-2.0
// RX-Completion-Status: Complete

//! Timer implementation

use crate::core::timer::types::Timer;
use crate::core::timer::constants::MS_PER_SECOND;

impl Timer {
    /// Start a new timer with the given name
    ///
    /// # Arguments
    /// * `name` - A human-readable identifier for this timer
    ///
    /// # Returns
    /// A new Timer instance that immediately begins measuring elapsed time
    ///
    /// # Examples
    /// ```rust
    /// let timer = Timer::start("database_query");
    /// ```
    pub fn start(name: &str) -> Self {
        Self {
            start: std::time::Instant::now(),
            stop: None,
            name: name.to_string(),
        }
    }

    /// Stop the timer and log the elapsed time using tracing
    ///
    /// This method consumes the timer and logs the elapsed time at debug level.
    /// The elapsed time is automatically converted to milliseconds for readability.
    ///
    /// # Examples
    /// ```rust
    /// let timer = Timer::start("operation");
    /// // ... perform work ...
    /// timer.stop(); // Logs: "Timer 'operation': 123.45ms"
    /// ```
    pub fn stop(mut self) {
        self.stop = Some(std::time::Instant::now());
        let elapsed = self.start.elapsed();
        tracing::debug!(
            "Timer '{}': {:.2}ms",
            self.name,
            elapsed.as_secs_f64() * MS_PER_SECOND
        );
    }

    /// Stop the timer and return the elapsed time in milliseconds
    ///
    /// This method consumes the timer, logs the elapsed time, and returns
    /// the elapsed duration as a floating-point number of milliseconds.
    ///
    /// # Returns
    /// The elapsed time in milliseconds as an f64
    ///
    /// # Examples
    /// ```rust
    /// let timer = Timer::start("calculation");
    /// // ... perform calculation ...
    /// let elapsed_ms = timer.stop_and_return();
    /// println!("Calculation took {:.2}ms", elapsed_ms);
    /// ```
    pub fn stop_and_return(mut self) -> f64 {
        self.stop = Some(std::time::Instant::now());
        let elapsed = self.start.elapsed();
        let ms = elapsed.as_secs_f64() * MS_PER_SECOND;
        tracing::debug!("Timer '{}': {:.2}ms", self.name, ms);
        ms
    }

    /// Get the current elapsed time without stopping the timer
    ///
    /// This method allows checking the elapsed time while keeping the timer running.
    /// Useful for intermediate measurements or progress reporting.
    ///
    /// # Returns
    /// The current elapsed time in milliseconds as an f64
    ///
    /// # Examples
    /// ```rust
    /// let timer = Timer::start("long_operation");
    /// // ... some work ...
    /// let intermediate = timer.elapsed_ms();
    /// println!("Intermediate time: {:.2}ms", intermediate);
    /// // ... more work ...
    /// timer.stop();
    /// ```
    pub fn elapsed_ms(&self) -> f64 {
        self.start.elapsed().as_secs_f64() * MS_PER_SECOND
    }

    /// Get the timer's name
    ///
    /// # Returns
    /// A reference to the timer's name string
    pub fn name(&self) -> &str {
        &self.name
    }
}
