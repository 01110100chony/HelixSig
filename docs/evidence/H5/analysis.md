# H5 experiment analysis

Evidence: **measured**. Commit: `1d8724c9521e7d88a43e6f751c4681e1fbbf75c6`.
744 runs; 124 configurations; 5 measured repetitions.

Median [Q1, Q3] across measured runs. Warmups are excluded. No minimum speedup is required.

## Pipeline

| W | Q | B | Policy | FFI | Written events/s | Drop fraction |
|---|---|---|---|---|---|---|
| 2 | 256 | 16 | drop-new | event | 424598 [323061, 424790] | 0.9128 [0.8995, 0.9266] |
| 2 | 256 | 1 | block | batch | 833176 [803598, 934449] | 0.0000 [0.0000, 0.0000] |
| 4 | 256 | 16 | drop-new | event | 382905 [364177, 398545] | 0.8904 [0.8901, 0.9048] |
| 4 | 256 | 16 | drop-new | batch | 393115 [322438, 394719] | 0.8715 [0.8702, 0.8849] |
| 2 | 64 | 16 | block | batch | 521841 [519616, 550969] | 0.0000 [0.0000, 0.0000] |
| 2 | 256 | 64 | block | batch | 902598 [887926, 910432] | 0.0000 [0.0000, 0.0000] |
| 2 | 1024 | 16 | block | batch | 463112 [444035, 473688] | 0.0000 [0.0000, 0.0000] |
| 2 | 256 | 64 | drop-new | event | 852741 [802617, 904811] | 0.7716 [0.7705, 0.7722] |
| 2 | 256 | 1 | block | event | 726035 [722318, 758921] | 0.0000 [0.0000, 0.0000] |
| 1 | 256 | 16 | block | event | 363903 [363647, 364110] | 0.0000 [0.0000, 0.0000] |
| 2 | 64 | 16 | drop-new | event | 480255 [340324, 545644] | 0.9095 [0.8738, 0.9417] |
| 2 | 1024 | 16 | drop-new | batch | 417554 [320644, 428608] | 0.8994 [0.8979, 0.9050] |
| 1 | 256 | 16 | block | batch | 368418 [366754, 369828] | 0.0000 [0.0000, 0.0000] |
| 2 | 1024 | 16 | block | event | 451309 [447177, 462004] | 0.0000 [0.0000, 0.0000] |
| 2 | 64 | 16 | block | event | 542316 [518785, 569446] | 0.0000 [0.0000, 0.0000] |
| 1 | 256 | 16 | drop-new | event | 511246 [511154, 528630] | 0.8794 [0.8718, 0.8914] |
| 2 | 64 | 16 | drop-new | batch | 422908 [417737, 572108] | 0.9115 [0.8713, 0.9126] |
| 4 | 256 | 16 | block | event | 513126 [494562, 519482] | 0.0000 [0.0000, 0.0000] |
| 4 | 256 | 16 | block | batch | 509060 [508902, 513668] | 0.0000 [0.0000, 0.0000] |
| 2 | 256 | 64 | block | event | 953582 [941878, 954339] | 0.0000 [0.0000, 0.0000] |
| 2 | 256 | 1 | drop-new | event | 864206 [718183, 895250] | 0.7491 [0.7386, 0.7596] |
| 2 | 256 | 16 | drop-new | batch | 402177 [370130, 531113] | 0.8925 [0.8853, 0.8995] |
| 2 | 256 | 16 | block | event | 499233 [483305, 506875] | 0.0000 [0.0000, 0.0000] |
| 2 | 256 | 16 | block | batch | 507085 [496920, 512669] | 0.0000 [0.0000, 0.0000] |
| 1 | 256 | 16 | drop-new | batch | 544423 [537478, 571094] | 0.8718 [0.8696, 0.8776] |
| 2 | 256 | 1 | drop-new | batch | 739568 [737640, 760002] | 0.7811 [0.7629, 0.7901] |
| 2 | 1024 | 16 | drop-new | event | 370946 [368815, 403048] | 0.8889 [0.8840, 0.8992] |
| 2 | 256 | 64 | drop-new | batch | 918903 [760817, 1031631] | 0.7191 [0.7073, 0.7264] |

## Microbenchmark

| N | B | Engine | FFI | Preparation | ns/event |
|---|---|---|---|---|---|
| 64 | 16 | rust_ffi | event | prepared | 107.4 [107.4, 111.2] |
| 256 | 1 | rust_ffi | event | pack | 526.6 [511.9, 528.1] |
| 4096 | 64 | rust_ffi | event | prepared | 7228.1 [7108.5, 7452.4] |
| 64 | 4 | rust_ffi | batch | prepared | 121.3 [119.6, 127.3] |
| 4096 | 16 | native | batch | pack | 8953.4 [8916.1, 8962.0] |
| 4096 | 4 | native | batch | prepared | 7552.3 [7277.1, 7926.3] |
| 4096 | 1 | rust_ffi | batch | pack | 8853.5 [8771.5, 9022.8] |
| 64 | 64 | rust_ffi | event | prepared | 105.5 [104.5, 106.4] |
| 4096 | 16 | rust_ffi | batch | pack | 8764.4 [8688.3, 8798.5] |
| 4096 | 1 | rust_ffi | event | pack | 8609.7 [8338.4, 8639.4] |
| 4096 | 64 | native | event | pack | 9395.6 [9357.8, 9750.1] |
| 64 | 4 | native | event | prepared | 107.2 [102.6, 115.2] |
| 256 | 16 | rust_ffi | event | pack | 476.5 [470.6, 477.4] |
| 256 | 64 | rust_ffi | event | pack | 483.6 [460.5, 494.0] |
| 256 | 1 | native | batch | prepared | 423.5 [420.1, 429.8] |
| 4096 | 4 | rust_ffi | batch | prepared | 7181.8 [7109.0, 7410.7] |
| 64 | 1 | rust_ffi | batch | prepared | 185.5 [182.2, 187.1] |
| 256 | 1 | native | event | prepared | 450.7 [434.8, 450.8] |
| 4096 | 4 | native | event | prepared | 7523.4 [7522.7, 7675.5] |
| 64 | 16 | rust_ffi | event | pack | 123.6 [122.3, 123.7] |
| 64 | 1 | rust_ffi | event | pack | 171.3 [170.5, 177.1] |
| 256 | 1 | rust_ffi | batch | prepared | 522.4 [515.5, 534.0] |
| 256 | 64 | rust_ffi | batch | pack | 447.2 [439.3, 452.9] |
| 64 | 1 | native | event | pack | 133.1 [128.2, 136.4] |
| 64 | 16 | native | batch | prepared | 90.9 [87.9, 91.6] |
| 4096 | 1 | native | event | pack | 8930.0 [8805.1, 8999.9] |
| 256 | 4 | rust_ffi | batch | pack | 494.2 [493.2, 498.5] |
| 256 | 64 | native | batch | pack | 448.0 [447.2, 448.0] |
| 256 | 1 | rust_ffi | event | prepared | 446.6 [441.7, 468.7] |
| 64 | 16 | rust_ffi | batch | pack | 144.0 [136.0, 144.6] |
| 64 | 1 | native | batch | pack | 111.5 [110.6, 112.8] |
| 64 | 64 | native | batch | prepared | 93.9 [90.2, 97.7] |
| 4096 | 16 | native | batch | prepared | 7788.0 [7709.7, 7954.4] |
| 4096 | 16 | native | event | pack | 9333.9 [9169.3, 9362.3] |
| 256 | 16 | native | event | pack | 490.0 [489.7, 501.0] |
| 4096 | 1 | native | batch | prepared | 7732.0 [7347.8, 7986.6] |
| 4096 | 16 | native | event | prepared | 7457.5 [7400.6, 7656.3] |
| 64 | 64 | native | event | pack | 115.0 [114.4, 142.8] |
| 256 | 64 | native | event | pack | 509.5 [487.9, 523.3] |
| 256 | 16 | native | batch | pack | 472.1 [468.1, 494.6] |
| 256 | 16 | native | event | prepared | 437.6 [431.1, 439.7] |
| 256 | 16 | rust_ffi | batch | prepared | 414.1 [409.9, 448.2] |
| 64 | 4 | rust_ffi | batch | pack | 143.6 [139.8, 144.7] |
| 64 | 1 | rust_ffi | event | prepared | 134.6 [134.3, 137.1] |
| 256 | 64 | rust_ffi | event | prepared | 410.3 [402.3, 420.3] |
| 256 | 4 | rust_ffi | event | pack | 480.3 [477.9, 489.6] |
| 64 | 4 | native | batch | pack | 104.2 [102.7, 105.2] |
| 4096 | 1 | native | event | prepared | 7628.2 [7595.8, 7715.5] |
| 4096 | 1 | rust_ffi | batch | prepared | 7354.0 [7304.1, 7606.8] |
| 64 | 4 | rust_ffi | event | pack | 132.7 [131.3, 133.9] |
| 4096 | 1 | rust_ffi | event | prepared | 7375.4 [7170.4, 7391.8] |
| 4096 | 1 | native | batch | pack | 8713.8 [8650.4, 9007.6] |
| 4096 | 64 | native | batch | pack | 9583.4 [9382.0, 9792.6] |
| 4096 | 4 | rust_ffi | event | pack | 8872.4 [8827.1, 8883.3] |
| 256 | 1 | native | event | pack | 467.4 [465.3, 487.0] |
| 4096 | 16 | rust_ffi | event | prepared | 7212.0 [7194.5, 7289.2] |
| 256 | 64 | native | event | prepared | 453.2 [451.6, 462.7] |
| 256 | 4 | native | event | pack | 493.5 [490.0, 493.5] |
| 4096 | 4 | rust_ffi | event | prepared | 7706.0 [7332.3, 7808.9] |
| 64 | 16 | rust_ffi | batch | prepared | 101.1 [100.5, 101.5] |
| 64 | 4 | rust_ffi | event | prepared | 113.7 [113.0, 113.9] |
| 4096 | 64 | rust_ffi | event | pack | 9439.2 [9371.4, 9468.4] |
| 64 | 4 | native | event | pack | 117.3 [117.0, 117.6] |
| 4096 | 64 | native | event | prepared | 7476.2 [7387.7, 7648.5] |
| 256 | 16 | rust_ffi | batch | pack | 472.8 [467.4, 475.5] |
| 256 | 4 | rust_ffi | event | prepared | 415.5 [404.5, 419.3] |
| 64 | 1 | native | event | prepared | 110.9 [110.6, 112.9] |
| 64 | 1 | native | batch | prepared | 97.6 [95.2, 98.5] |
| 256 | 1 | rust_ffi | batch | pack | 599.9 [599.1, 610.4] |
| 64 | 16 | native | batch | pack | 102.4 [100.3, 103.1] |
| 256 | 1 | native | batch | pack | 495.3 [480.6, 501.0] |
| 256 | 16 | rust_ffi | event | prepared | 423.4 [421.6, 435.5] |
| 4096 | 64 | rust_ffi | batch | pack | 9379.3 [9104.6, 9382.3] |
| 64 | 16 | native | event | pack | 117.8 [115.2, 118.0] |
| 256 | 16 | native | batch | prepared | 405.4 [401.0, 415.6] |
| 256 | 4 | rust_ffi | batch | prepared | 418.1 [412.1, 421.7] |
| 4096 | 16 | rust_ffi | batch | prepared | 7156.1 [7058.4, 7232.2] |
| 4096 | 4 | rust_ffi | batch | pack | 8538.6 [8469.3, 8647.7] |
| 64 | 16 | native | event | prepared | 105.4 [104.5, 105.4] |
| 64 | 64 | rust_ffi | batch | pack | 114.5 [114.5, 115.4] |
| 256 | 4 | native | batch | prepared | 409.1 [408.5, 412.5] |
| 4096 | 4 | native | batch | pack | 8653.4 [8529.6, 8698.3] |
| 4096 | 4 | native | event | pack | 8924.4 [8497.9, 9131.2] |
| 64 | 64 | native | batch | pack | 109.3 [107.8, 128.7] |
| 256 | 64 | rust_ffi | batch | prepared | 395.8 [385.0, 411.8] |
| 64 | 4 | native | batch | prepared | 92.3 [90.7, 94.6] |
| 64 | 64 | native | event | prepared | 103.7 [103.5, 105.1] |
| 256 | 64 | native | batch | prepared | 422.7 [413.9, 423.9] |
| 4096 | 64 | native | batch | prepared | 7525.6 [7364.0, 7727.2] |
| 256 | 4 | native | event | prepared | 432.7 [431.7, 453.4] |
| 64 | 64 | rust_ffi | batch | prepared | 103.4 [98.5, 110.6] |
| 64 | 64 | rust_ffi | event | pack | 124.5 [124.1, 131.2] |
| 256 | 4 | native | batch | pack | 460.6 [450.0, 462.7] |
| 4096 | 16 | rust_ffi | event | pack | 8942.9 [8772.0, 9057.4] |
| 4096 | 64 | rust_ffi | batch | prepared | 7163.8 [7059.7, 7174.2] |
| 64 | 1 | rust_ffi | batch | pack | 230.1 [229.9, 230.3] |

## Interpretation limits

These are local finite-replay observations. WSL scheduling, competing host load, cache state and the writer affect results. The pipeline includes writing; the microbenchmark excludes I/O and allocation, includes result consumption, and pack mode includes the row-to-flat copy. Differences between executables are not exact isolated FFI overhead. Single-axis comparisons do not establish interactions. Five runs do not characterize production tails or hard real time.

Raw Parquet retains successful-event latencies only. Per-run empirical p50/p95/p99, population, small-population flags and histogram overflow are in analysis.json; percentiles are never averaged across runs. Loss fractions must accompany drop-new throughput. Peak Linux process RSS is not a whole-system or Windows-host memory measure.
