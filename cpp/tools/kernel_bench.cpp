#include "helix/kernel.hpp"

#include <algorithm>
#include <array>
#include <bit>
#include <chrono>
#include <filesystem>
#include <fstream>
#include <iomanip>
#include <iostream>
#include <stdexcept>
#include <string>
#include <vector>

// Standalone H5 reference. The runner validates the manifest/hash before launch.
int main(int argc, char **argv) {
    try {
        static_assert(std::endian::native == std::endian::little);
        if (argc != 8) {
            throw std::runtime_error(
                "usage: kernel_bench payload N K B events event|batch prepared|pack");
        }
        const auto width = std::stoull(argv[2]);
        const auto baseline = std::stoull(argv[3]);
        const auto capacity = std::stoull(argv[4]);
        const auto events = std::stoull(argv[5]);
        const std::string mode = argv[6];
        const std::string preparation = argv[7];
        if (width < 2 || width > helix::max_samples || baseline == 0 || baseline >= width ||
            capacity == 0 || capacity > helix::max_batch || events == 0 || events > 1000000 ||
            (mode != "event" && mode != "batch") ||
            (preparation != "prepared" && preparation != "pack")) {
            throw std::runtime_error("invalid benchmark configuration");
        }
        const auto bytes = std::filesystem::file_size(argv[1]);
        if (bytes == 0 || bytes > 64 * 1024 * 1024 || bytes % (width * sizeof(double)) != 0) {
            throw std::runtime_error("invalid bounded corpus size");
        }
        std::vector<double> prepared(bytes / sizeof(double));
        std::ifstream input(argv[1], std::ios::binary);
        if (!input.read(reinterpret_cast<char *>(prepared.data()),
                        static_cast<std::streamsize>(bytes))) {
            throw std::runtime_error("cannot read corpus");
        }
        const auto rows = prepared.size() / width;
        if (rows != 256) {
            throw std::runtime_error("the benchmark requires the preregistered 256-row corpus");
        }
        std::vector<std::vector<double>> source;
        source.reserve(rows);
        for (std::size_t row = 0; row < rows; ++row) {
            source.emplace_back(prepared.begin() + row * width,
                                prepared.begin() + (row + 1) * width);
        }
        std::vector<double> packed(capacity * width);
        std::vector<helix::EventResult> results(capacity);
        const helix::Config config{static_cast<std::uint32_t>(baseline)};
        std::uint64_t count = 0;
        double checksum = 0;
        std::array<std::uint64_t, 65> batches{};
        std::vector<helix::EventResult> observed(events);
        const auto start = std::chrono::steady_clock::now();
        while (count < events) {
            const auto row = count % rows;
            const auto batch = std::min<std::uint64_t>({capacity, rows - row, events - count});
            auto samples = std::span<const double>(prepared).subspan(row * width, batch * width);
            if (preparation == "pack") {
                for (std::size_t i = 0; i < batch; ++i) {
                    std::copy(source[row + i].begin(), source[row + i].end(),
                              packed.begin() + i * width);
                }
                samples = std::span<const double>(packed).first(batch * width);
            }
            if (mode == "event") {
                for (std::size_t i = 0; i < batch; ++i) {
                    results[i] = helix::process_event(samples.subspan(i * width, width), config);
                }
            } else if (helix::process_batch(samples, width, config,
                                            std::span<helix::EventResult>(results).first(batch)) !=
                       helix::BatchStatus::ok) {
                throw std::runtime_error("invalid batch shape");
            }
            // Observable consumption is inside timing in both executables.
            for (std::size_t i = 0; i < batch; ++i) {
                const auto &result = results[i];
                if (result.status != helix::Status::ok) {
                    throw std::runtime_error("invalid numerical benchmark result");
                }
                checksum += result.baseline + result.peak_amplitude + result.integral +
                            static_cast<double>(result.peak_index);
            }
            std::copy_n(results.begin(), batch, observed.begin() + count);
            count += batch;
            ++batches[batch];
        }
        const auto duration = std::chrono::duration_cast<std::chrono::nanoseconds>(
                                  std::chrono::steady_clock::now() - start)
                                  .count();
        // Every captured result is checked after timing. Only unique corpus-row
        // tuples need to be emitted for independent tolerance-aware validation.
        for (std::size_t i = 0; i < observed.size(); ++i) {
            const auto &a = observed[i];
            const auto &b = observed[i % rows];
            if (a.status != b.status || a.baseline != b.baseline ||
                a.peak_amplitude != b.peak_amplitude || a.peak_index != b.peak_index ||
                a.integral != b.integral) {
                throw std::runtime_error("repeated corpus row produced inconsistent results");
            }
        }
        std::cout << std::setprecision(17)
                  << "{\"schema_version\":1,\"engine\":\"native\",\"events\":" << count
                  << ",\"duration_ns\":" << duration << ",\"checksum\":" << checksum
                  << ",\"samples\":" << width << ",\"batch_size\":" << capacity << ",\"ffi\":\""
                  << mode << "\",\"preparation\":\"" << preparation
                  << "\",\"actual_batch_sizes\":[";
        for (std::size_t i = 0; i < batches.size(); ++i) {
            std::cout << (i == 0 ? "" : ",") << batches[i];
        }
        std::cout << "],\"repeat_consistent\":true,\"verification\":[";
        for (std::size_t i = 0; i < std::min<std::size_t>(rows, observed.size()); ++i) {
            const auto &r = observed[i];
            std::cout << (i == 0 ? "" : ",") << '[' << static_cast<unsigned>(r.status) << ','
                      << r.baseline << ',' << r.peak_amplitude << ',' << r.peak_index << ','
                      << r.integral << ']';
        }
        std::cout << "]}\n";
        return 0;
    } catch (const std::exception &error) {
        std::cerr << error.what() << '\n';
        return 1;
    }
}
