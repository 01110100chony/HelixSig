#include "helix/kernel.hpp"
#include "helix/src/bridge.rs.h"

#include <array>
#include <stdexcept>

namespace helix::ffi {
namespace {
NativeResult translate(const helix::EventResult &value) noexcept {
    return {static_cast<std::uint32_t>(value.status), value.baseline, value.peak_amplitude,
            value.peak_index, value.integral};
}
} // namespace

NativeResult process_event(rust::Slice<const double> samples, std::uint32_t baseline_samples) {
    return translate(helix::process_event({samples.data(), samples.size()}, {baseline_samples}));
}

void process_batch(rust::Slice<const double> samples, std::size_t width,
                   std::uint32_t baseline_samples, rust::Slice<NativeResult> output) {
    if (output.size() > helix::max_batch) {
        throw std::invalid_argument("batch exceeds 64 events");
    }
    std::array<helix::EventResult, helix::max_batch> native{};
    const auto status = helix::process_batch({samples.data(), samples.size()}, width,
                                             {baseline_samples}, {native.data(), output.size()});
    if (status != helix::BatchStatus::ok) {
        throw std::invalid_argument(status == helix::BatchStatus::invalid_shape
                                        ? "invalid batch shape"
                                        : "invalid baseline configuration");
    }
    for (std::size_t index = 0; index < output.size(); ++index) {
        output[index] = translate(native[index]);
    }
}
} // namespace helix::ffi
