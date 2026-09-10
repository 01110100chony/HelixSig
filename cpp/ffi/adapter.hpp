#pragma once
#include "rust/cxx.h"
#include <cstddef>
#include <cstdint>

namespace helix::ffi {
struct NativeResult;
NativeResult process_event(rust::Slice<const double> samples, std::uint32_t baseline_samples);
void process_batch(rust::Slice<const double> samples, std::size_t width,
                   std::uint32_t baseline_samples, rust::Slice<NativeResult> output);
} // namespace helix::ffi
