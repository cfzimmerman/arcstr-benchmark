# String v. Arc<str>

### Summary

After watching [this video](https://youtu.be/A4cKi7PTJSs?si=plUGwRyWAUZrBwva), I started using `Arc<str>` instead of `String` in a high-throughput Tokio project. However, since atomics also come at a cost, I wanted to investigate the performance difference more closely.

This project has an async task that clones an `AsRef<str>` some input number of times. This task is spawned some input number of times with either a `String` or `Arc<str>` parameter. For various lengths of randomly generated strings, this computes the time for all tasks to finish.

### Local benchmark

I generated the results below on an M1 Mac. This config spawns one million tasks over five trials for each pre-configured string length with each task cloning the string 2048 times:

```bash
cargo run —release csv-report 1000000 5 2048 ./output/1m_tasks.csv
```

The cleaned-up CSV output can be viewed here: https://docs.google.com/spreadsheets/d/1zzypKX4YaPGdfQYl2N_NGOUf-X4mDFYXeSDcWQpqRiA/edit?usp=sharing

### Conclusions

Disclaimers: I ran this on the only hardware I have access to. I have no idea how architecture-dependent results are, and I’d be very curious to see what output other computers produce. Also, as can be seen in the flamegraphs in `./output`, Tokio uses a decent amount of CPU time on its own. My motivating case mirrors this structure with Tokio, but these results might not generalize to other situations.

That said, it seems `String` beats `Arc<str>` up to a character length of 32 but not past 64. Critically for my usage, this implies UUIDs (length 36) likely fare better with an ordinary `String` (or `Box<str>`) over `Arc<str>`. I assume the relative performance of `String` is due to stronger memory optimizations and the complexity of atomic arithmetic.

The more mysterious question this raises, however, is why absolute `Arc<str>` times actually decrease as the string gets larger. This makes no sense to me, and I'll leave it for now assuming something related the memory allocator or cache behavior.
