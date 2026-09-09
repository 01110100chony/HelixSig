#pragma once

#include <cstddef>
#include <cstdint>
#include <span>

namespace helix {
inline constexpr std::size_t max_samples = 4096;
inline constexpr std::size_t max_batch = 64;

struct Config {
    std::uint32_t baseline_samples = 32;
};

enum class Status : std::uint32_t {
    ok = 0,
    invalid_size = 1,
    invalid_config = 2,
    nonfinite_input = 3,
    nonfinite_result = 4,
};

struct EventResult {
    Status status = Status::ok;
    double baseline = 0;
    double peak_amplitude = 0;
    std::uint32_t peak_index = 0;
    double integral = 0;
};

enum class BatchStatus { ok, invalid_shape, invalid_config };

// Borrows input only until return. No allocation, retained pointers or shared state.
[[nodiscard]] EventResult process_event(std::span<const double> samples, Config config) noexcept;

// Structural errors leave ALL output elements unchanged. Empty batches are valid
// when width and configuration are valid. Individual numerical errors are statuses.
[[nodiscard]] BatchStatus process_batch(std::span<const double> samples,
                                       std::size_t samples_per_event, Config config,
                                       std::span<EventResult> output) noexcept;
} // namespace helix
