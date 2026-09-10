#include "helix/kernel.hpp"

#include <bit>
#include <fstream>
#include <iomanip>
#include <iostream>
#include <stdexcept>
#include <string>
#include <vector>

// Test-only raw corpus evaluator. Rows are streamed; it is not a production CLI.
int main(int argc, char **argv) {
    try {
        if (argc != 4) {
            throw std::runtime_error("usage: kernel_driver corpus.f64le width baseline_samples");
        }
        static_assert(std::endian::native == std::endian::little);
        const auto width = std::stoul(argv[2]);
        const auto baseline = std::stoul(argv[3]);
        if (width < 2 || width > helix::max_samples || baseline == 0 || baseline >= width) {
            throw std::runtime_error("invalid dimensions/configuration");
        }
        std::ifstream input(argv[1], std::ios::binary);
        if (!input) {
            throw std::runtime_error("cannot open corpus");
        }
        std::vector<double> samples(width);
        const auto bytes = static_cast<std::streamsize>(width * sizeof(double));
        std::cout << "row,status,baseline,peak_amplitude,peak_index,integral\n"
                  << std::setprecision(17);
        std::uint64_t row = 0;
        while (input.read(reinterpret_cast<char *>(samples.data()), bytes)) {
            const auto result =
                helix::process_event(samples, {static_cast<std::uint32_t>(baseline)});
            std::cout << row++ << ',' << static_cast<unsigned>(result.status) << ','
                      << result.baseline << ',' << result.peak_amplitude << ',' << result.peak_index
                      << ',' << result.integral << '\n';
        }
        if (input.gcount() != 0 || !input.eof()) {
            throw std::runtime_error("truncated corpus or input error");
        }
        return 0;
    } catch (const std::exception &error) {
        std::cerr << error.what() << '\n';
        return 1;
    }
}
