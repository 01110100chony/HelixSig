#include "helix/kernel.hpp"

#include <cmath>
#include <limits>

namespace helix {
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
    double baseline = 0;
    for (const auto value : samples.first(k)) {
        baseline += value;
    }
    baseline /= k;
    if (!std::isfinite(baseline)) {
        return {.status = Status::nonfinite_result};
    }
    EventResult result{.baseline = baseline,
                       .peak_amplitude = -std::numeric_limits<double>::infinity(),
                       .peak_index = k};
    for (std::size_t i = k; i < samples.size(); ++i) {
        const double corrected = samples[i] - baseline;
        result.integral += corrected;
        if (!std::isfinite(corrected) || !std::isfinite(result.integral)) {
            return {.status = Status::nonfinite_result};
        }
        // Strict comparison preserves the FIRST maximum, including negative peaks.
        if (corrected > result.peak_amplitude) {
            result.peak_amplitude = corrected;
            result.peak_index = static_cast<std::uint32_t>(i);
        }
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
