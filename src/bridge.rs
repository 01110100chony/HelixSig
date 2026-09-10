//! Synchronous borrowed interface: C++ retains no pointer or reference after return.
//! Native statuses: 0=ok, 1=invalid size, 2=invalid configuration,
//! 3=nonfinite input, 4=nonfinite result. Structural batch errors return Err.

#[cxx::bridge(namespace = "helix::ffi")]
pub mod ffi {
    #[derive(Debug, Default, Clone, Copy, PartialEq)]
    struct NativeResult {
        status: u32,
        baseline: f64,
        peak_amplitude: f64,
        peak_index: u32,
        integral: f64,
    }

    // SAFETY: Rust/CXX borrowing guarantees the validity and lifetime of borrowed slices/spans.
    // Dimensional validation is performed by the native kernel. The adapter performs explicit
    // field mapping and forwards borrowed data safely without reinterpret-cast or raw layout-copy
    // assumptions; input is immutable, output is exclusively borrowed, and no pointers are retained.
    unsafe extern "C++" {
        include!("adapter.hpp");
        fn process_event(samples: &[f64], baseline_samples: u32) -> Result<NativeResult>;
        fn process_batch(
            samples: &[f64],
            width: usize,
            baseline_samples: u32,
            output: &mut [NativeResult],
        ) -> Result<()>;
    }
}
