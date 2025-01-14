//! Temporal quantification.
//!
//! Example:
//!
//! ```
//! use std::time::Duration;
//!
//! let five_seconds = Duration::new(5, 0);
//! // both declarations are equivalent
//! assert_eq!(Duration::new(5, 0), Duration::from_secs(5));
//! ```

use core::fmt;
use core::ops::{Add, AddAssign, Sub, SubAssign};
pub use core::time::Duration;

use crate::error::Error;
use crate::file_io::sys::time;
use crate::file_io::sys_common::FromInner;

/// A measurement of a monotonically nondecreasing clock.
/// Opaque and useful only with `Duration`.
///
/// Instants are always guaranteed to be no less than any previously measured
/// instant when created, and are often useful for tasks such as measuring
/// benchmarks or timing how long an operation takes.
///
/// Note, however, that instants are not guaranteed to be **steady**. In other
/// words, each tick of the underlying clock may not be the same length (e.g.
/// some seconds may be longer than others). An instant may jump forwards or
/// experience time dilation (slow down or speed up), but it will never go
/// backwards.
///
/// Instants are opaque types that can only be compared to one another. There is
/// no method to get "the number of seconds" from an instant. Instead, it only
/// allows measuring the duration between two instants (or comparing two
/// instants).
///
/// The size of an `Instant` struct may vary depending on the target operating
/// system.
///
/// Example:
///
/// ```no_run
/// use std::time::{Duration, Instant};
/// use std::thread::sleep;
///
/// fn main() {
///    let now = Instant::now();
///
///    // we sleep for 2 seconds
///    sleep(Duration::new(2, 0));
///    // it prints '2'
///    println!("{}", now.elapsed().as_secs());
/// }
/// ```
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Instant(time::Instant);

/// A measurement of the system clock, useful for talking to
/// external entities like the file system or other processes.
///
/// Distinct from the [`Instant`] type, this time measurement **is not
/// monotonic**. This means that you can save a file to the file system, then
/// save another file to the file system, **and the second file has a
/// `SystemTime` measurement earlier than the first**. In other words, an
/// operation that happens after another operation in real time may have an
/// earlier `SystemTime`!
///
/// Consequently, comparing two `SystemTime` instances to learn about the
/// duration between them returns a [`Result`] instead of an infallible
/// [`Duration`] to indicate that this sort of time drift may happen and needs
/// to be handled.
///
/// Although a `SystemTime` cannot be directly inspected, the [`UNIX_EPOCH`]
/// constant is provided in this module as an anchor in time to learn
/// information about a `SystemTime`. By calculating the duration from this
/// fixed point in time, a `SystemTime` can be converted to a human-readable
/// time, or perhaps some other string representation.
///
/// The size of a `SystemTime` struct may vary depending on the target operating
/// system.
///
/// [`Instant`]: ../../std/time/struct.Instant.html
/// [`Result`]: ../../std/result/enum.Result.html
/// [`Duration`]: ../../std/time/struct.Duration.html
/// [`UNIX_EPOCH`]: ../../std/time/constant.UNIX_EPOCH.html
///
/// Example:
///
/// ```no_run
/// use std::time::{Duration, SystemTime};
/// use std::thread::sleep;
///
/// fn main() {
///    let now = SystemTime::now();
///
///    // we sleep for 2 seconds
///    sleep(Duration::new(2, 0));
///    match now.elapsed() {
///        Ok(elapsed) => {
///            // it prints '2'
///            println!("{}", elapsed.as_secs());
///        }
///        Err(e) => {
///            // an error occurred!
///            println!("Error: {:?}", e);
///        }
///    }
/// }
/// ```
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SystemTime(time::SystemTime);

/// An error returned from the `duration_since` and `elapsed` methods on
/// `SystemTime`, used to learn how far in the opposite direction a system time
/// lies.
///
/// # Examples
///
/// ```no_run
/// use std::thread::sleep;
/// use std::time::{Duration, SystemTime};
///
/// let sys_time = SystemTime::now();
/// sleep(Duration::from_secs(1));
/// let new_sys_time = SystemTime::now();
/// match sys_time.duration_since(new_sys_time) {
///     Ok(_) => {}
///     Err(e) => println!("SystemTimeError difference: {:?}", e.duration()),
/// }
/// ```
#[derive(Clone, Debug)]
pub struct SystemTimeError(Duration);

impl Instant {
	/// Returns an instant corresponding to "now".
	///
	/// # Examples
	///
	/// ```
	/// use std::time::Instant;
	///
	/// let now = Instant::now();
	/// ```

	pub fn now() -> Instant {
		Instant(time::Instant::now())
	}

	/// Returns the amount of time elapsed from another instant to this one.
	///
	/// # Panics
	///
	/// This function will panic if `earlier` is later than `self`.
	///
	/// # Examples
	///
	/// ```no_run
	/// use std::time::{Duration, Instant};
	/// use std::thread::sleep;
	///
	/// let now = Instant::now();
	/// sleep(Duration::new(1, 0));
	/// let new_now = Instant::now();
	/// println!("{:?}", new_now.duration_since(now));
	/// ```

	pub fn duration_since(&self, earlier: Instant) -> Duration {
		self.0
			.checked_sub_instant(&earlier.0)
			.expect("supplied instant is later than self")
	}

	/// Returns the amount of time elapsed from another instant to this one,
	/// or None if that instant is earlier than this one.
	///
	/// # Examples
	///
	/// ```no_run
	/// #![feature(checked_duration_since)]
	/// use std::time::{Duration, Instant};
	/// use std::thread::sleep;
	///
	/// let now = Instant::now();
	/// sleep(Duration::new(1, 0));
	/// let new_now = Instant::now();
	/// println!("{:?}", new_now.checked_duration_since(now));
	/// println!("{:?}", now.checked_duration_since(new_now)); // None
	/// ```

	pub fn checked_duration_since(&self, earlier: Instant) -> Option<Duration> {
		self.0.checked_sub_instant(&earlier.0)
	}

	/// Returns the amount of time elapsed from another instant to this one,
	/// or zero duration if that instant is earlier than this one.
	///
	/// # Examples
	///
	/// ```no_run
	/// #![feature(checked_duration_since)]
	/// use std::time::{Duration, Instant};
	/// use std::thread::sleep;
	///
	/// let now = Instant::now();
	/// sleep(Duration::new(1, 0));
	/// let new_now = Instant::now();
	/// println!("{:?}", new_now.saturating_duration_since(now));
	/// println!("{:?}", now.saturating_duration_since(new_now)); // 0ns
	/// ```

	pub fn saturating_duration_since(&self, earlier: Instant) -> Duration {
		self.checked_duration_since(earlier)
			.unwrap_or_else(|| Duration::new(0, 0))
	}

	/// Returns the amount of time elapsed since this instant was created.
	///
	/// # Panics
	///
	/// This function may panic if the current time is earlier than this
	/// instant, which is something that can happen if an `Instant` is
	/// produced synthetically.
	///
	/// # Examples
	///
	/// ```no_run
	/// use std::thread::sleep;
	/// use std::time::{Duration, Instant};
	///
	/// let instant = Instant::now();
	/// let three_secs = Duration::from_secs(3);
	/// sleep(three_secs);
	/// assert!(instant.elapsed() >= three_secs);
	/// ```

	pub fn elapsed(&self) -> Duration {
		Instant::now() - *self
	}

	/// Returns `Some(t)` where `t` is the time `self + duration` if `t` can be
	/// represented as `Instant` (which means it's inside the bounds of the
	/// underlying data structure), `None` otherwise.

	pub fn checked_add(&self, duration: Duration) -> Option<Instant> {
		self.0.checked_add_duration(&duration).map(Instant)
	}

	/// Returns `Some(t)` where `t` is the time `self - duration` if `t` can be
	/// represented as `Instant` (which means it's inside the bounds of the
	/// underlying data structure), `None` otherwise.

	pub fn checked_sub(&self, duration: Duration) -> Option<Instant> {
		self.0.checked_sub_duration(&duration).map(Instant)
	}
}

impl Add<Duration> for Instant {
	type Output = Instant;

	/// # Panics
	///
	/// This function may panic if the resulting point in time cannot be
	/// represented by the underlying data structure. See [`checked_add`] for a
	/// version without panic.
	///
	/// [`checked_add`]: ../../std/time/struct.Instant.html#method.checked_add
	fn add(self, other: Duration) -> Instant {
		self.checked_add(other)
			.expect("overflow when adding duration to instant")
	}
}

impl AddAssign<Duration> for Instant {
	fn add_assign(&mut self, other: Duration) {
		*self = *self + other;
	}
}

impl Sub<Duration> for Instant {
	type Output = Instant;

	fn sub(self, other: Duration) -> Instant {
		self.checked_sub(other)
			.expect("overflow when subtracting duration from instant")
	}
}

impl SubAssign<Duration> for Instant {
	fn sub_assign(&mut self, other: Duration) {
		*self = *self - other;
	}
}

impl Sub<Instant> for Instant {
	type Output = Duration;

	fn sub(self, other: Instant) -> Duration {
		self.duration_since(other)
	}
}

impl fmt::Debug for Instant {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.0.fmt(f)
	}
}

impl SystemTime {
	/// An anchor in time which can be used to create new `SystemTime` instances
	/// or learn about where in time a `SystemTime` lies.
	///
	/// This constant is defined to be "1970-01-01 00:00:00 UTC" on all systems
	/// with respect to the system clock. Using `duration_since` on an existing
	/// `SystemTime` instance can tell how far away from this point in time a
	/// measurement lies, and using `UNIX_EPOCH + duration` can be used to
	/// create a `SystemTime` instance to represent another fixed point in time.
	///
	/// # Examples
	///
	/// ```no_run
	/// use std::time::SystemTime;
	///
	/// match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
	///     Ok(n) => println!("1970-01-01 00:00:00 UTC was {} seconds ago!", n.as_secs()),
	///     Err(_) => panic!("SystemTime before UNIX EPOCH!"),
	/// }
	/// ```

	pub const UNIX_EPOCH: SystemTime = UNIX_EPOCH;

	/// Returns the system time corresponding to "now".
	///
	/// # Examples
	///
	/// ```
	/// use std::time::SystemTime;
	///
	/// let sys_time = SystemTime::now();
	/// ```

	pub fn now() -> SystemTime {
		SystemTime(time::SystemTime::now())
	}

	/// Returns the amount of time elapsed from an earlier point in time.
	///
	/// This function may fail because measurements taken earlier are not
	/// guaranteed to always be before later measurements (due to anomalies such
	/// as the system clock being adjusted either forwards or backwards).
	///
	/// If successful, [`Ok`]`(`[`Duration`]`)` is returned where the duration
	/// represents the amount of time elapsed from the specified measurement to
	/// this one.
	///
	/// Returns an [`Err`] if `earlier` is later than `self`, and the error
	/// contains how far from `self` the time is.
	///
	/// [`Ok`]: ../../std/result/enum.Result.html#variant.Ok
	/// [`Duration`]: ../../std/time/struct.Duration.html
	/// [`Err`]: ../../std/result/enum.Result.html#variant.Err
	///
	/// # Examples
	///
	/// ```
	/// use std::time::SystemTime;
	///
	/// let sys_time = SystemTime::now();
	/// let difference = sys_time.duration_since(sys_time)
	///                          .expect("SystemTime::duration_since failed");
	/// println!("{:?}", difference);
	/// ```

	pub fn duration_since(&self, earlier: SystemTime) -> Result<Duration, SystemTimeError> {
		self.0.sub_time(&earlier.0).map_err(SystemTimeError)
	}

	/// Returns the amount of time elapsed since this system time was created.
	///
	/// This function may fail as the underlying system clock is susceptible to
	/// drift and updates (e.g., the system clock could go backwards), so this
	/// function may not always succeed. If successful, [`Ok`]`(`[`Duration`]`)`
	/// is returned where the duration represents the amount of time elapsed
	/// from this time measurement to the current time.
	///
	/// Returns an [`Err`] if `self` is later than the current system time, and
	/// the error contains how far from the current system time `self` is.
	///
	/// [`Ok`]: ../../std/result/enum.Result.html#variant.Ok
	/// [`Duration`]: ../../std/time/struct.Duration.html
	/// [`Err`]: ../../std/result/enum.Result.html#variant.Err
	///
	/// # Examples
	///
	/// ```no_run
	/// use std::thread::sleep;
	/// use std::time::{Duration, SystemTime};
	///
	/// let sys_time = SystemTime::now();
	/// let one_sec = Duration::from_secs(1);
	/// sleep(one_sec);
	/// assert!(sys_time.elapsed().unwrap() >= one_sec);
	/// ```

	pub fn elapsed(&self) -> Result<Duration, SystemTimeError> {
		SystemTime::now().duration_since(*self)
	}

	/// Returns `Some(t)` where `t` is the time `self + duration` if `t` can be
	/// represented as `SystemTime` (which means it's inside the bounds of the
	/// underlying data structure), `None` otherwise.

	pub fn checked_add(&self, duration: Duration) -> Option<SystemTime> {
		self.0.checked_add_duration(&duration).map(SystemTime)
	}

	/// Returns `Some(t)` where `t` is the time `self - duration` if `t` can be
	/// represented as `SystemTime` (which means it's inside the bounds of the
	/// underlying data structure), `None` otherwise.

	pub fn checked_sub(&self, duration: Duration) -> Option<SystemTime> {
		self.0.checked_sub_duration(&duration).map(SystemTime)
	}
}

impl Add<Duration> for SystemTime {
	type Output = SystemTime;

	/// # Panics
	///
	/// This function may panic if the resulting point in time cannot be
	/// represented by the underlying data structure. See [`checked_add`] for a
	/// version without panic.
	///
	/// [`checked_add`]:
	/// ../../std/time/struct.SystemTime.html#method.checked_add
	fn add(self, dur: Duration) -> SystemTime {
		self.checked_add(dur)
			.expect("overflow when adding duration to instant")
	}
}

impl AddAssign<Duration> for SystemTime {
	fn add_assign(&mut self, other: Duration) {
		*self = *self + other;
	}
}

impl Sub<Duration> for SystemTime {
	type Output = SystemTime;

	fn sub(self, dur: Duration) -> SystemTime {
		self.checked_sub(dur)
			.expect("overflow when subtracting duration from instant")
	}
}

impl SubAssign<Duration> for SystemTime {
	fn sub_assign(&mut self, other: Duration) {
		*self = *self - other;
	}
}

impl fmt::Debug for SystemTime {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.0.fmt(f)
	}
}

/// An anchor in time which can be used to create new `SystemTime` instances or
/// learn about where in time a `SystemTime` lies.
///
/// This constant is defined to be "1970-01-01 00:00:00 UTC" on all systems with
/// respect to the system clock. Using `duration_since` on an existing
/// [`SystemTime`] instance can tell how far away from this point in time a
/// measurement lies, and using `UNIX_EPOCH + duration` can be used to create a
/// [`SystemTime`] instance to represent another fixed point in time.
///
/// [`SystemTime`]: ../../std/time/struct.SystemTime.html
///
/// # Examples
///
/// ```no_run
/// use std::time::{SystemTime, UNIX_EPOCH};
///
/// match SystemTime::now().duration_since(UNIX_EPOCH) {
///     Ok(n) => println!("1970-01-01 00:00:00 UTC was {} seconds ago!", n.as_secs()),
///     Err(_) => panic!("SystemTime before UNIX EPOCH!"),
/// }
/// ```

pub const UNIX_EPOCH: SystemTime = SystemTime(time::UNIX_EPOCH);

impl SystemTimeError {
	/// Returns the positive duration which represents how far forward the
	/// second system time was from the first.
	///
	/// A `SystemTimeError` is returned from the [`duration_since`] and
	/// [`elapsed`] methods of [`SystemTime`] whenever the second system time
	/// represents a point later in time than the `self` of the method call.
	///
	/// [`duration_since`]:
	/// ../../std/time/struct.SystemTime.html#method.duration_since [`elapsed`]:
	/// ../../std/time/struct.SystemTime.html#method.elapsed [`SystemTime`]:
	/// ../../std/time/struct.SystemTime.html
	///
	/// # Examples
	///
	/// ```no_run
	/// use std::thread::sleep;
	/// use std::time::{Duration, SystemTime};
	///
	/// let sys_time = SystemTime::now();
	/// sleep(Duration::from_secs(1));
	/// let new_sys_time = SystemTime::now();
	/// match sys_time.duration_since(new_sys_time) {
	///     Ok(_) => {}
	///     Err(e) => println!("SystemTimeError difference: {:?}", e.duration()),
	/// }
	/// ```

	pub fn duration(&self) -> Duration {
		self.0
	}
}

impl Error for SystemTimeError {
	fn description(&self) -> &str {
		"other time was not earlier than self"
	}
}

impl fmt::Display for SystemTimeError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "second time provided was later than self")
	}
}

impl FromInner<time::SystemTime> for SystemTime {
	fn from_inner(time: time::SystemTime) -> SystemTime {
		SystemTime(time)
	}
}
