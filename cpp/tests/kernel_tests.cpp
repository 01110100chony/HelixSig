#include "helix/kernel.hpp"

#include <array>
#include <cmath>
#include <cstdlib>
#include <iostream>
#include <limits>
#include <vector>

namespace {
void check(bool condition, const char *message) {
    // Never use assert: these checks must run under Release/NDEBUG too.
    if (!condition) {
        std::cerr << "FAIL: " << message << '\n';
        std::exit(1);
    }
}
} // namespace

int main() {
    using helix::Status;
    const std::array<double, 6> signal{-2, -2, 1, 5, 5, -3};
    const auto result = helix::process_event(signal, {2});
    check(result.status == Status::ok, "valid signal");
    check(result.baseline == -2 && result.peak_amplitude == 7, "baseline and amplitude");
    check(result.peak_index == 3 && result.integral == 16, "first peak and signed integral");
    const std::array<double, 4> below{3, 3, 1, 2};
    const auto negative = helix::process_event(below, {2});
    check(negative.peak_amplitude == -1 && negative.integral == -3, "no implicit clipping");
    const std::array<double, 2> smallest{4, 9};
    check(helix::process_event(smallest, {1}).integral == 5, "minimum size/config");
    std::vector<double> largest(helix::max_samples, 2);
    check(helix::process_event(largest, {4095}).peak_index == 4095, "maximum size/config");
    check(helix::process_event({}, {1}).status == Status::invalid_size, "empty event");
    largest.push_back(2);
    check(helix::process_event(largest, {1}).status == Status::invalid_size, "oversize event");
    check(helix::process_event(signal, {0}).status == Status::invalid_config, "zero baseline");
    check(helix::process_event(signal, {6}).status == Status::invalid_config, "full baseline");
    auto invalid = signal;
    for (const auto bad : {std::numeric_limits<double>::quiet_NaN(),
                           std::numeric_limits<double>::infinity(),
                           -std::numeric_limits<double>::infinity()}) {
        invalid[4] = bad;
        check(helix::process_event(invalid, {2}).status == Status::nonfinite_input,
              "nonfinite input");
    }
    const double max = std::numeric_limits<double>::max();
    const std::array<double, 3> overflow_baseline{max, max, 1};
    check(helix::process_event(overflow_baseline, {2}).status == Status::nonfinite_result,
          "baseline overflow");
    const std::array<double, 3> overflow_integral{0, max, max};
    check(helix::process_event(overflow_integral, {1}).status == Status::nonfinite_result,
          "integral overflow");
    const std::array<double, 2> overflow_corrected{-max, max};
    check(helix::process_event(overflow_corrected, {1}).status == Status::nonfinite_result,
          "subtraction overflow");

    std::vector<double> batch(signal.begin(), signal.end());
    batch.reserve(3 * signal.size());
    batch.insert(batch.end(), invalid.begin(), invalid.end());
    batch.insert(batch.end(), signal.begin(), signal.end());
    std::array<helix::EventResult, 3> output{};
    check(helix::process_batch(batch, 6, {2}, output) == helix::BatchStatus::ok,
          "partial three-event batch");
    check(output[0].integral == 16 && output[1].status == Status::nonfinite_input &&
              output[2].peak_index == 3,
          "invalid event is isolated");
    output[0].integral = 123;
    check(helix::process_batch(batch, 0, {2}, output) == helix::BatchStatus::invalid_shape,
          "zero batch width");
    check(helix::process_batch(batch, 5, {2}, output) == helix::BatchStatus::invalid_shape,
          "shape mismatch");
    check(helix::process_batch(batch, 6, {0}, output) == helix::BatchStatus::invalid_config,
          "invalid batch config");
    check(output[0].integral == 123, "structural errors preserve output");
    check(helix::process_batch({}, 6, {2}, {}) == helix::BatchStatus::ok, "empty batch");
    std::cout << "kernel contract PASS\n";
}
