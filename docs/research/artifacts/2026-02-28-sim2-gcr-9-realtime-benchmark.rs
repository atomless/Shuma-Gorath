use std::collections::VecDeque;
use std::time::Instant;

#[derive(Clone, Copy, Debug)]
struct Event {
    ts_ms: u64,
}

#[derive(Debug)]
struct Metrics {
    delivered: u64,
    dropped_or_overflow: u64,
    calls_or_connections: u64,
    p50_latency_ms: u64,
    p95_latency_ms: u64,
    p99_latency_ms: u64,
    mean_latency_ms: f64,
    cpu_wall_ms: u128,
    peak_queue_depth: usize,
    approx_memory_bytes: u64,
}

fn percentile(sorted: &[u64], pct: f64) -> u64 {
    if sorted.is_empty() {
        return 0;
    }
    let idx = ((sorted.len() as f64 * pct).ceil() as usize)
        .saturating_sub(1)
        .min(sorted.len() - 1);
    sorted[idx]
}

fn summarize_latencies(latencies: &mut [u64]) -> (u64, u64, u64, f64) {
    if latencies.is_empty() {
        return (0, 0, 0, 0.0);
    }
    latencies.sort_unstable();
    let total: u128 = latencies.iter().map(|v| *v as u128).sum();
    let mean = total as f64 / latencies.len() as f64;
    (
        percentile(latencies, 0.50),
        percentile(latencies, 0.95),
        percentile(latencies, 0.99),
        mean,
    )
}

fn generate_constant_events(duration_ms: u64, events_per_sec: u64) -> Vec<Event> {
    let mut events = Vec::new();
    let mut accumulator = 0.0f64;
    let per_ms = events_per_sec as f64 / 1000.0;
    for t in 0..duration_ms {
        accumulator += per_ms;
        let emit = accumulator.floor() as u64;
        accumulator -= emit as f64;
        for _ in 0..emit {
            events.push(Event { ts_ms: t });
        }
    }
    events
}

fn simulate_cursor_polling(
    events: &[Event],
    duration_ms: u64,
    clients: usize,
    poll_interval_ms: u64,
    delta_limit: usize,
    event_size_bytes: u64,
) -> Metrics {
    let start = Instant::now();
    let mut cursors = vec![0usize; clients];
    let mut latencies = Vec::with_capacity(events.len().saturating_mul(clients));
    let mut delivered = 0u64;
    let mut overflow = 0u64;
    let mut calls = 0u64;

    let mut event_upper_idx = 0usize;
    let mut t = 0u64;
    while t <= duration_ms {
        while event_upper_idx < events.len() && events[event_upper_idx].ts_ms <= t {
            event_upper_idx += 1;
        }

        for cursor in &mut cursors {
            calls += 1;
            let available = event_upper_idx.saturating_sub(*cursor);
            let take = available.min(delta_limit);
            if take == 0 {
                continue;
            }
            let start_idx = *cursor;
            let end_idx = start_idx + take;
            for event in &events[start_idx..end_idx] {
                latencies.push(t.saturating_sub(event.ts_ms));
            }
            delivered += take as u64;
            *cursor = end_idx;
            if available > delta_limit {
                overflow += (available - delta_limit) as u64;
            }
        }

        t = t.saturating_add(poll_interval_ms);
        if poll_interval_ms == 0 {
            break;
        }
    }

    let (p50, p95, p99, mean) = summarize_latencies(&mut latencies);
    Metrics {
        delivered,
        dropped_or_overflow: overflow,
        calls_or_connections: calls,
        p50_latency_ms: p50,
        p95_latency_ms: p95,
        p99_latency_ms: p99,
        mean_latency_ms: mean,
        cpu_wall_ms: start.elapsed().as_millis(),
        peak_queue_depth: delta_limit,
        approx_memory_bytes: (clients as u64)
            .saturating_mul(delta_limit as u64)
            .saturating_mul(event_size_bytes),
    }
}

fn simulate_sse_streaming(
    events: &[Event],
    duration_ms: u64,
    clients: usize,
    consume_interval_ms: u64,
    queue_capacity: usize,
    event_size_bytes: u64,
) -> Metrics {
    let start = Instant::now();
    let mut queues: Vec<VecDeque<usize>> = (0..clients)
        .map(|_| VecDeque::with_capacity(queue_capacity))
        .collect();
    let mut peak_queue_depth = 0usize;
    let mut latencies = Vec::with_capacity(events.len().saturating_mul(clients));
    let mut delivered = 0u64;
    let mut dropped = 0u64;

    let mut event_idx = 0usize;
    for t in 0..=duration_ms {
        while event_idx < events.len() && events[event_idx].ts_ms <= t {
            for queue in &mut queues {
                if queue.len() >= queue_capacity {
                    dropped += 1;
                } else {
                    queue.push_back(event_idx);
                    if queue.len() > peak_queue_depth {
                        peak_queue_depth = queue.len();
                    }
                }
            }
            event_idx += 1;
        }

        if consume_interval_ms > 0 && t % consume_interval_ms == 0 {
            for queue in &mut queues {
                while let Some(idx) = queue.pop_front() {
                    let event = events[idx];
                    latencies.push(t.saturating_sub(event.ts_ms));
                    delivered += 1;
                }
            }
        }
    }

    let (p50, p95, p99, mean) = summarize_latencies(&mut latencies);
    Metrics {
        delivered,
        dropped_or_overflow: dropped,
        calls_or_connections: clients as u64,
        p50_latency_ms: p50,
        p95_latency_ms: p95,
        p99_latency_ms: p99,
        mean_latency_ms: mean,
        cpu_wall_ms: start.elapsed().as_millis(),
        peak_queue_depth,
        approx_memory_bytes: (clients as u64)
            .saturating_mul(queue_capacity as u64)
            .saturating_mul(event_size_bytes),
    }
}

fn print_row(label: &str, metrics: &Metrics) {
    println!(
        "{label}: delivered={} overflow_or_drop={} calls_or_conns={} p50={} p95={} p99={} mean={:.2} cpu_ms={} peak={} memB={}",
        metrics.delivered,
        metrics.dropped_or_overflow,
        metrics.calls_or_connections,
        metrics.p50_latency_ms,
        metrics.p95_latency_ms,
        metrics.p99_latency_ms,
        metrics.mean_latency_ms,
        metrics.cpu_wall_ms,
        metrics.peak_queue_depth,
        metrics.approx_memory_bytes,
    );
}

fn run_scenario(name: &str, events: &[Event], duration_ms: u64) {
    const EVENT_SIZE_BYTES: u64 = 256;
    println!("=== {name} ===");

    let poll_default = simulate_cursor_polling(events, duration_ms, 5, 1000, 600, EVENT_SIZE_BYTES);
    let poll_fast = simulate_cursor_polling(events, duration_ms, 5, 250, 400, EVENT_SIZE_BYTES);
    let sse = simulate_sse_streaming(events, duration_ms, 5, 250, 1024, EVENT_SIZE_BYTES);

    print_row("poll_default", &poll_default);
    print_row("poll_fast", &poll_fast);
    print_row("sse", &sse);
    println!();
}

fn main() {
    let steady = generate_constant_events(120_000, 200);
    run_scenario("steady_200eps_120s", &steady, 120_000);

    let burst = generate_constant_events(30_000, 1000);
    run_scenario("burst_1000eps_30s", &burst, 30_000);
}
