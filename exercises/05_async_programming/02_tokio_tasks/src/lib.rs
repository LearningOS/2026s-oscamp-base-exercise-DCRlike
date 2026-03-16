//! # Tokio Async Tasks
//!
//! In this exercise, you will use `tokio::spawn` to create concurrent asynchronous tasks.
//!
//! ## Concepts
//! - `tokio::spawn` creates asynchronous tasks
//! - `JoinHandle` waits for task completion
//! - Concurrent execution between asynchronous tasks


use tokio::time::{Duration};

/// Concurrently compute the square of each number in 0..n, collect results and return in order.
///
/// Hint: Create `tokio::spawn` task for each i, collect JoinHandle, await them sequentially.
pub async fn concurrent_squares(n: usize) -> Vec<usize> {
    // TODO: Create n asynchronous tasks, each computing i * i
    // TODO: Collect all JoinHandle
    // TODO: Await each one to get result
    async fn square(i: usize) -> usize {
        i * i
    }
    let mut tasks = Vec::new();
    for i in 0..n {
        let task = tokio::spawn(square(i)).await.unwrap();
        tasks.push(task);
    }
    tasks
}

/// Concurrently execute multiple "time-consuming" tasks (simulated with sleep), return all results.
/// Each task sleeps `duration_ms` milliseconds and then returns its `task_id`.
///
/// Key: All tasks should execute concurrently, total duration should be close to single task duration, not sum of all tasks.
pub async fn parallel_sleep_tasks(n: usize, duration_ms: u64) -> Vec<usize> {
    // TODO: Create asynchronous task for each id in 0..n
    // TODO: Each task sleeps specified duration and returns its own id
    // TODO: Collect all results and sort
    let mut handles = Vec::new();

    for i in 0..n {
        // spawn 后不立刻 await，先把 handle 存起来
        let handle = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(duration_ms)).await;
            i  // 返回 task_id
        });
        handles.push(handle);
    }

    let mut results = Vec::new();
    for handle in handles {
        let id = handle.await.unwrap();
        results.push(id);
    }

    results.sort();
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::Instant;

    #[tokio::test]
    async fn test_squares_basic() {
        let result = concurrent_squares(5).await;
        assert_eq!(result, vec![0, 1, 4, 9, 16]);
    }

    #[tokio::test]
    async fn test_squares_zero() {
        let result = concurrent_squares(0).await;
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_squares_one() {
        let result = concurrent_squares(1).await;
        assert_eq!(result, vec![0]);
    }

    #[tokio::test]
    async fn test_parallel_sleep() {
        let start = Instant::now();
        let result = parallel_sleep_tasks(5, 100).await;
        let elapsed = start.elapsed();

        assert_eq!(result, vec![0, 1, 2, 3, 4]);
        // Concurrent execution, total time should be much less than 5 * 100ms
        assert!(
            elapsed.as_millis() < 400,
            "Tasks should run concurrently, took {}ms",
            elapsed.as_millis()
        );
    }
}
