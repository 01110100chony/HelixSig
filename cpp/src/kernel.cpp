#include "helix/kernel.hpp"

#include <array>
#include <cmath>
#include <limits>

namespace helix {
namespace {
// Blocked pairwise reduction: shorter accumulation chains than a linear sum.
// The grouping is explicit so the pinned independent NumPy oracle and this
// native implementation have comparable rounding on cancellation-heavy inputs.
// This is a scalar algorithm, not a SIMD intrinsic or a call into NumPy.
double pairwise_sum(std::span<const double> values, double offset) noexcept {
    if (values.size() < 8) {
        double sum = -0.0;
        for (const double value : values) {
            sum += value - offset;
        }
        return sum;
    }
    if (values.size() > 128) {
        const auto half = (values.size() / 2) / 8 * 8;
        return pairwise_sum(values.first(half), offset) +
               pairwise_sum(values.subspan(half), offset);
    }
    std::array<double, 8> lanes{};
    for (std::size_t lane = 0; lane < lanes.size(); ++lane) {
        lanes[lane] = values[lane] - offset;
    }
    std::size_t index = 8;
    for (; index + 8 <= values.size(); index += 8) {
        for (std::size_t lane = 0; lane < lanes.size(); ++lane) {
            lanes[lane] += values[index + lane] - offset;
        }
    }
    double sum = ((lanes[0] + lanes[1]) + (lanes[2] + lanes[3])) +
                 ((lanes[4] + lanes[5]) + (lanes[6] + lanes[7]));
    for (; index < values.size(); ++index) {
        sum += values[index] - offset;
    }
    return sum;
}
} // namespace

EventResult process_event(std::span<const double> samples, Config config) noexcept {
    if (samples.size() < 2 || samples.size() > max_samples) {
        return {.status = Status::invalid_size};
    }
    const auto k = config.baseline_samples;
    if (k == 0 || k >= samples.size()) {
        return {.status = Status::invalid_config};
    }
    for (const auto value : samples) {
        if (!std::isfinite(value)) {
            return {.status = Status::nonfinite_input};
        }
    }
    const double baseline = pairwise_sum(samples.first(k), 0) / k;
    if (!std::isfinite(baseline)) {
        return {.status = Status::nonfinite_result};
    }
    EventResult result{.baseline = baseline,
                       .peak_amplitude = -std::numeric_limits<double>::infinity(),
                       .peak_index = k};
    for (std::size_t i = k; i < samples.size(); ++i) {
        const double corrected = samples[i] - baseline;
        if (!std::isfinite(corrected)) {
            return {.status = Status::nonfinite_result};
        }
        // Strict comparison preserves the FIRST maximum, including negative peaks.
        if (corrected > result.peak_amplitude) {
            result.peak_amplitude = corrected;
            result.peak_index = static_cast<std::uint32_t>(i);
        }
    }
    result.integral = pairwise_sum(samples.subspan(k), baseline);
    if (!std::isfinite(result.integral)) {
        return {.status = Status::nonfinite_result};
    }
    return result;
}

BatchStatus process_batch(std::span<const double> samples, std::size_t width, Config config,
                          std::span<EventResult> output) noexcept {
    // Division/remainder avoids multiplication overflow on untrusted dimensions.
    if (width < 2 || width > max_samples || output.size() > max_batch ||
        samples.size() % width != 0 || samples.size() / width != output.size()) {
        return BatchStatus::invalid_shape;
    }
    if (config.baseline_samples == 0 || config.baseline_samples >= width) {
        return BatchStatus::invalid_config;
    }
    for (std::size_t row = 0; row < output.size(); ++row) {
        output[row] = process_event(samples.subspan(row * width, width), config);
    }
    return BatchStatus::ok;
}
} // namespace helix
