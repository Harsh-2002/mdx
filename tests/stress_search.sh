#!/usr/bin/env bash
# Stress test for mdx search at scale (50–500 files)
# Tests BM25 ranking correctness, performance scaling, and memory usage
set -euo pipefail

# ── Configuration ──────────────────────────────────────────────────────────
SCALES=(50 100 150 200 300 400 500)
SCRIPT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
MDX="$SCRIPT_DIR/target/release/mdx"
TEST_ROOT="/tmp/mdx_stress_test"
DOCS_DIR="$TEST_ROOT/docs"
RESULTS_CSV="$TEST_ROOT/results.csv"
SUMMARY_FILE="$TEST_ROOT/summary.txt"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
RESET='\033[0m'

# ── Phase 0: Setup ─────────────────────────────────────────────────────────
echo -e "${BOLD}Phase 0: Building release binary${RESET}"
cd "$SCRIPT_DIR"
cargo build --release 2>&1 | tail -1
echo "Binary: $MDX"

rm -rf "$TEST_ROOT"
mkdir -p "$DOCS_DIR"

echo "scale,test,time_ms,results,top_file,pass,peak_rss_kb" > "$RESULTS_CSV"

# ── Topic Data ─────────────────────────────────────────────────────────────

TOPICS=(programming cooking science history travel music mathematics biology engineering philosophy)

# 8 paragraph templates per topic (indexed 0-7)
paragraph() {
    local topic="$1"
    local idx="$2"
    case "$topic" in
        programming)
            case $((idx % 8)) in
                0) echo "Optimization of algorithms is a fundamental concern in software development. Efficient data structures enable better performance across all layers of the application stack.";;
                1) echo "Modern programming languages provide powerful abstractions for concurrent execution. Rust and Go have pioneered new approaches to safe parallelism.";;
                2) echo "The compiler performs multiple optimization passes to transform source code into efficient machine instructions. Loop unrolling and inlining are common techniques.";;
                3) echo "Functional programming emphasizes immutability and pure functions. This paradigm enables easier reasoning about program behavior and facilitates optimization.";;
                4) echo "Debugging distributed systems requires sophisticated tooling. Tracing frameworks help developers understand request flows across microservices.";;
                5) echo "Code review is an essential practice for maintaining quality. Automated linting and static analysis complement human review effectively.";;
                6) echo "Async programming models allow efficient I/O handling without blocking threads. Event loops and futures provide the foundation for scalable network services.";;
                7) echo "Memory management strategies differ significantly across languages. Garbage collection, reference counting, and ownership models each have distinct trade-offs for optimization.";;
            esac;;
        cooking)
            case $((idx % 8)) in
                0) echo "Optimization of cooking techniques requires understanding heat transfer and chemical reactions. The Maillard reaction creates complex flavors in seared proteins.";;
                1) echo "Fermentation is an ancient preservation technique that transforms simple ingredients. Sourdough starters contain wild yeast and lactobacillus cultures.";;
                2) echo "Knife skills are fundamental to efficient kitchen work. A sharp chef's knife and proper technique reduce preparation time significantly.";;
                3) echo "Sous vide cooking achieves precise temperature control for consistent results. This optimization of traditional methods yields remarkable texture.";;
                4) echo "Spice blending is both art and science. Toasting whole spices before grinding releases volatile oils and deepens flavor profiles.";;
                5) echo "Baking relies on precise measurements and chemical leavening agents. Understanding gluten development is key to bread texture optimization.";;
                6) echo "Stock making extracts collagen and minerals from bones over long simmering periods. A well-made stock is the foundation of French cuisine.";;
                7) echo "Food preservation methods include canning, dehydrating, and pickling. Each technique offers different advantages for extending shelf life.";;
            esac;;
        science)
            case $((idx % 8)) in
                0) echo "Optimization of experimental design reduces confounding variables. Randomized controlled trials remain the gold standard for causal inference.";;
                1) echo "Quantum mechanics describes nature at the smallest scales. Wave-particle duality challenges classical intuitions about physical reality.";;
                2) echo "Climate models integrate atmospheric, oceanic, and terrestrial data. Computational optimization enables higher resolution predictions of future conditions.";;
                3) echo "CRISPR gene editing has revolutionized molecular biology. This technology allows precise modifications to DNA sequences with unprecedented efficiency.";;
                4) echo "Particle accelerators probe the fundamental structure of matter. The Standard Model successfully describes three of the four fundamental forces.";;
                5) echo "Astronomical observations span the electromagnetic spectrum. Radio telescopes detect signals from the earliest epochs of the universe.";;
                6) echo "Materials science combines chemistry and physics to develop new substances. Graphene and metamaterials offer extraordinary properties for engineering applications.";;
                7) echo "Neural networks in computational neuroscience model brain function. Understanding synaptic plasticity is essential for explaining learning and memory.";;
            esac;;
        history)
            case $((idx % 8)) in
                0) echo "The optimization of agricultural practices during the Neolithic revolution transformed human societies. Surplus production enabled specialization and urbanization.";;
                1) echo "The Renaissance marked a period of extraordinary intellectual and artistic achievement. Humanism placed individual potential at the center of philosophical inquiry.";;
                2) echo "Industrial revolutions reshaped economies and social structures. Mechanization and optimization of production processes accelerated economic growth exponentially.";;
                3) echo "Ancient trade routes connected civilizations across vast distances. The Silk Road facilitated exchange of goods, ideas, and cultural practices.";;
                4) echo "The development of writing systems enabled record-keeping and administration. Cuneiform and hieroglyphics represent early solutions to information management.";;
                5) echo "Colonial expansion altered global power dynamics for centuries. The resulting cultural exchanges and conflicts shaped the modern world order.";;
                6) echo "Democratic ideals evolved from ancient Athens through Enlightenment philosophy. Constitutional frameworks attempted to balance liberty with collective governance.";;
                7) echo "Archaeological methods continue to reveal new insights about ancient civilizations. Carbon dating and DNA analysis have transformed historical understanding.";;
            esac;;
        travel)
            case $((idx % 8)) in
                0) echo "Route optimization for long-distance travel has evolved from celestial navigation to GPS satellites. Modern algorithms calculate efficient paths across complex networks.";;
                1) echo "Sustainable tourism practices aim to minimize environmental impact. Ecotourism initiatives support conservation while providing authentic cultural experiences.";;
                2) echo "High-speed rail networks connect major cities with remarkable efficiency. Japan's Shinkansen demonstrates the optimization of ground transportation infrastructure.";;
                3) echo "Backpacking through remote regions requires careful planning and preparation. Lightweight gear optimization enables longer journeys with greater comfort.";;
                4) echo "Cultural immersion enhances the travel experience beyond sightseeing. Learning local customs and basic language skills deepens connection with communities.";;
                5) echo "Aviation technology has made global travel accessible. Modern aircraft design prioritizes fuel efficiency and passenger comfort through aerodynamic optimization.";;
                6) echo "Mountain expeditions demand physical preparation and logistical planning. Altitude acclimatization is critical for safety above four thousand meters.";;
                7) echo "Digital nomadism combines remote work with travel freedom. Coworking spaces in popular destinations support this growing lifestyle trend.";;
            esac;;
        music)
            case $((idx % 8)) in
                0) echo "Audio optimization in recording studios involves careful microphone placement and room acoustics. Signal processing enhances clarity while preserving natural dynamics.";;
                1) echo "Music theory provides a framework for understanding harmonic relationships. Counterpoint and voice leading principles guide composition across genres.";;
                2) echo "Digital audio workstations have democratized music production. Home studios now achieve professional quality through software optimization and affordable hardware.";;
                3) echo "Orchestration requires knowledge of instrument ranges, timbres, and techniques. Balancing an ensemble involves careful optimization of dynamic levels.";;
                4) echo "Improvisation in jazz develops through mastery of scales, arpeggios, and patterns. Active listening and interaction drive collective musical creation.";;
                5) echo "Sound synthesis generates tones through mathematical waveform manipulation. Subtractive, additive, and FM synthesis offer distinct tonal palettes.";;
                6) echo "Music cognition research explores how the brain processes rhythm, melody, and harmony. Neural responses to music reveal deep connections between sound and emotion.";;
                7) echo "Live sound engineering adapts to venue acoustics and audience size. Feedback suppression and equalization optimization ensure clear audio delivery.";;
            esac;;
        mathematics)
            case $((idx % 8)) in
                0) echo "Optimization problems seek to find the best solution within defined constraints. Linear programming and gradient descent are foundational techniques.";;
                1) echo "Number theory explores properties of integers and prime numbers. The Riemann hypothesis remains one of mathematics' greatest unsolved problems.";;
                2) echo "Topology studies properties preserved under continuous deformation. Knot theory and manifold classification have applications in physics and data analysis.";;
                3) echo "Statistical methods quantify uncertainty and enable data-driven decisions. Bayesian inference updates probability estimates as new evidence emerges.";;
                4) echo "Graph theory models relationships between discrete objects. Network analysis reveals structural properties in social, biological, and technological systems.";;
                5) echo "Cryptographic algorithms rely on computational hardness assumptions. Factoring large numbers and discrete logarithm problems underpin modern security.";;
                6) echo "Differential equations describe continuous change in physical systems. Numerical methods provide approximate solutions when analytical approaches are intractable.";;
                7) echo "Category theory provides abstract frameworks for mathematical structures. Functors and natural transformations unify diverse areas of mathematics through optimization of conceptual clarity.";;
            esac;;
        biology)
            case $((idx % 8)) in
                0) echo "Evolution by natural selection is the optimization process underlying biological diversity. Adaptation to environmental pressures drives speciation over geological time.";;
                1) echo "Cellular respiration converts glucose to ATP through a series of enzymatic reactions. Mitochondria serve as the powerhouses of eukaryotic cells.";;
                2) echo "Ecosystem dynamics involve complex interactions between producers, consumers, and decomposers. Biodiversity contributes to resilience and stability in natural systems.";;
                3) echo "Protein folding determines biological function from amino acid sequences. Misfolded proteins are implicated in diseases like Alzheimer's and Parkinson's.";;
                4) echo "Symbiotic relationships range from mutualism to parasitism. Mycorrhizal networks connect plant root systems in forest ecosystems.";;
                5) echo "Genetic regulation controls when and where genes are expressed. Epigenetic modifications add layers of complexity to inheritance patterns.";;
                6) echo "Marine biology studies organisms from tidal pools to deep ocean trenches. Coral reef ecosystems support extraordinary biodiversity despite covering small areas.";;
                7) echo "Immunology investigates how organisms defend against pathogens. Adaptive immunity enables specific, memory-based responses through optimization of antibody production.";;
            esac;;
        engineering)
            case $((idx % 8)) in
                0) echo "Structural optimization balances strength and material usage. Finite element analysis simulates stress distribution to identify potential failure points.";;
                1) echo "Control systems theory governs feedback loops in automated processes. PID controllers adjust output based on error measurements to maintain setpoints.";;
                2) echo "Renewable energy engineering harnesses solar, wind, and tidal power. Grid integration requires optimization of intermittent generation with storage solutions.";;
                3) echo "Robotics combines mechanical design, electronics, and software. Sensor fusion and path planning algorithms enable autonomous navigation in complex environments.";;
                4) echo "Civil engineering projects require geotechnical analysis and environmental assessment. Bridge design exemplifies the balance between load capacity and aesthetic form.";;
                5) echo "Thermodynamic cycles govern heat engines and refrigeration systems. Carnot efficiency sets theoretical limits on energy conversion optimization.";;
                6) echo "Signal processing extracts information from noisy measurements. Fourier transforms decompose complex signals into constituent frequency components.";;
                7) echo "Manufacturing processes benefit from lean principles and continuous improvement. Six Sigma methodology applies statistical optimization to reduce defects and variation.";;
            esac;;
        philosophy)
            case $((idx % 8)) in
                0) echo "Ethical frameworks provide systematic approaches to moral reasoning. Utilitarianism seeks optimization of overall well-being across affected populations.";;
                1) echo "Epistemology examines the nature and limits of knowledge. Skeptical traditions challenge assumptions about certainty and justified belief.";;
                2) echo "Existentialism emphasizes individual freedom and responsibility. Authenticity requires confronting the absence of predetermined meaning in human existence.";;
                3) echo "Philosophy of mind explores consciousness, intentionality, and mental causation. The hard problem of consciousness remains deeply contested.";;
                4) echo "Political philosophy debates justice, liberty, and the role of the state. Social contract theories attempt to justify governmental authority.";;
                5) echo "Aesthetics investigates the nature of beauty, art, and taste. Theories of artistic expression range from formalism to institutional definitions.";;
                6) echo "Logic provides formal tools for valid reasoning. Modal logic extends classical systems to handle necessity, possibility, and counterfactuals.";;
                7) echo "Philosophy of science examines the foundations of empirical inquiry. Falsifiability and paradigm shifts describe how scientific understanding evolves through optimization of theories.";;
            esac;;
    esac
}

# 5 heading templates per topic (indexed 0-4)
heading() {
    local topic="$1"
    local idx="$2"
    case "$topic" in
        programming) case $((idx % 5)) in
            0) echo "Core Concepts";;  1) echo "Performance Analysis";;
            2) echo "Design Patterns";;  3) echo "Best Practices";;
            4) echo "Advanced Techniques";;
        esac;;
        cooking) case $((idx % 5)) in
            0) echo "Essential Techniques";;  1) echo "Flavor Development";;
            2) echo "Kitchen Fundamentals";;  3) echo "Recipe Principles";;
            4) echo "Advanced Methods";;
        esac;;
        science) case $((idx % 5)) in
            0) echo "Experimental Methods";;  1) echo "Theoretical Framework";;
            2) echo "Data Analysis";;  3) echo "Research Methodology";;
            4) echo "Recent Discoveries";;
        esac;;
        history) case $((idx % 5)) in
            0) echo "Key Events";;  1) echo "Cultural Impact";;
            2) echo "Economic Forces";;  3) echo "Social Transformation";;
            4) echo "Legacy and Influence";;
        esac;;
        travel) case $((idx % 5)) in
            0) echo "Planning Guide";;  1) echo "Destination Overview";;
            2) echo "Transportation";;  3) echo "Local Experiences";;
            4) echo "Practical Tips";;
        esac;;
        music) case $((idx % 5)) in
            0) echo "Fundamentals";;  1) echo "Production Techniques";;
            2) echo "Theory and Harmony";;  3) echo "Performance Practice";;
            4) echo "Audio Engineering";;
        esac;;
        mathematics) case $((idx % 5)) in
            0) echo "Foundational Concepts";;  1) echo "Problem Solving";;
            2) echo "Applied Methods";;  3) echo "Proof Techniques";;
            4) echo "Computational Approaches";;
        esac;;
        biology) case $((idx % 5)) in
            0) echo "Cellular Mechanisms";;  1) echo "Evolutionary Perspectives";;
            2) echo "Ecological Systems";;  3) echo "Molecular Processes";;
            4) echo "Organismal Biology";;
        esac;;
        engineering) case $((idx % 5)) in
            0) echo "Design Principles";;  1) echo "System Architecture";;
            2) echo "Analysis Methods";;  3) echo "Implementation";;
            4) echo "Testing and Validation";;
        esac;;
        philosophy) case $((idx % 5)) in
            0) echo "Central Questions";;  1) echo "Theoretical Positions";;
            2) echo "Critical Analysis";;  3) echo "Contemporary Debates";;
            4) echo "Historical Development";;
        esac;;
    esac
}

# Code block templates per topic (2-3 per topic)
code_block() {
    local topic="$1"
    local idx="$2"
    case "$topic" in
        programming) case $((idx % 3)) in
            0) printf '```rust\nfn optimize(data: &mut Vec<i32>) {\n    data.sort_unstable();\n    data.dedup();\n}\n```';;
            1) printf '```python\ndef binary_search(arr, target):\n    lo, hi = 0, len(arr) - 1\n    while lo <= hi:\n        mid = (lo + hi) // 2\n        if arr[mid] == target:\n            return mid\n        elif arr[mid] < target:\n            lo = mid + 1\n        else:\n            hi = mid - 1\n    return -1\n```';;
            2) printf '```go\nfunc worker(jobs <-chan int, results chan<- int) {\n    for j := range jobs {\n        results <- process(j)\n    }\n}\n```';;
        esac;;
        cooking) case $((idx % 2)) in
            0) printf '```yaml\nrecipe:\n  name: Sourdough Bread\n  prep_time: 30m\n  ferment_time: 12h\n  bake_temp: 450F\n```';;
            1) printf '```text\nMise en place:\n- 500g bread flour\n- 350g water (70%% hydration)\n- 10g salt\n- 100g starter\n```';;
        esac;;
        science) case $((idx % 3)) in
            0) printf '```python\nimport numpy as np\nresults = np.random.normal(mu, sigma, n_samples)\np_value = stats.ttest_ind(control, treatment).pvalue\n```';;
            1) printf '```r\nmodel <- lm(response ~ predictor1 + predictor2, data=df)\nsummary(model)\n```';;
            2) printf '```latex\nE = mc^2\n\\Delta G = \\Delta H - T\\Delta S\n```';;
        esac;;
        history) case $((idx % 2)) in
            0) printf '```text\nTimeline:\n  3000 BCE - First writing systems\n  500 BCE  - Classical Athens\n  1440 CE  - Printing press\n  1789 CE  - French Revolution\n```';;
            1) printf '```text\nPrimary Sources:\n- Archaeological artifacts\n- Written records and chronicles\n- Oral traditions and folklore\n```';;
        esac;;
        travel) case $((idx % 2)) in
            0) printf '```json\n{\n  "destination": "Kyoto",\n  "duration_days": 5,\n  "budget_usd": 2000,\n  "season": "spring"\n}\n```';;
            1) printf '```text\nPacking list:\n- Passport and copies\n- Universal adapter\n- First aid kit\n- Water purification tablets\n```';;
        esac;;
        music) case $((idx % 2)) in
            0) printf '```text\nSignal chain:\n  Microphone -> Preamp -> EQ -> Compressor -> DAW\n  Gain staging: -18dBFS target\n```';;
            1) printf '```text\nChord progression (ii-V-I):\n  Dm7 | G7 | Cmaj7 | Cmaj7\n```';;
        esac;;
        mathematics) case $((idx % 3)) in
            0) printf '```python\ndef gradient_descent(f, grad_f, x0, lr=0.01, epochs=1000):\n    x = x0\n    for _ in range(epochs):\n        x = x - lr * grad_f(x)\n    return x\n```';;
            1) printf '```text\nProof by induction:\n  Base case: P(0) holds\n  Inductive step: P(k) => P(k+1)\n  Therefore: P(n) for all n >= 0\n```';;
            2) printf '```python\nimport sympy\nx = sympy.Symbol("x")\nresult = sympy.integrate(sympy.exp(-x**2), (x, -sympy.oo, sympy.oo))\n```';;
        esac;;
        biology) case $((idx % 2)) in
            0) printf '```text\nDNA -> mRNA -> Protein\n  Transcription: RNA polymerase\n  Translation: Ribosomes + tRNA\n```';;
            1) printf '```python\nfrom Bio import SeqIO\nfor record in SeqIO.parse("genome.fasta", "fasta"):\n    print(record.id, len(record.seq))\n```';;
        esac;;
        engineering) case $((idx % 3)) in
            0) printf '```python\n# Finite element stress analysis\nK = assemble_stiffness(elements, nodes)\nu = np.linalg.solve(K, F)\nstress = compute_stress(u, elements)\n```';;
            1) printf '```text\nPID Controller:\n  u(t) = Kp*e(t) + Ki*integral(e) + Kd*de/dt\n  Tuning: Ziegler-Nichols method\n```';;
            2) printf '```python\ndef transfer_function(num, den, freq):\n    s = 1j * freq\n    return np.polyval(num, s) / np.polyval(den, s)\n```';;
        esac;;
        philosophy) case $((idx % 2)) in
            0) printf '```text\nSyllogism:\n  Premise 1: All humans are mortal\n  Premise 2: Socrates is human\n  Conclusion: Socrates is mortal\n```';;
            1) printf '```text\nTrolley problem variants:\n  1. Standard: divert trolley, kill 1 to save 5\n  2. Footbridge: push person to stop trolley\n  3. Loop: divert through person to block\n```';;
        esac;;
    esac
}

# ── File Generation ────────────────────────────────────────────────────────

generate_files() {
    local n="$1"
    rm -rf "$DOCS_DIR"/*

    for ((i = 0; i < n; i++)); do
        local topic_idx=$((i % 10))
        local topic="${TOPICS[$topic_idx]}"
        local filename
        filename=$(printf "doc_%04d_%s.md" "$i" "$topic")

        # Determine size class: 20% small, 60% medium, 20% large
        local size_class
        local mod5=$((i % 5))
        if [ "$mod5" -eq 0 ]; then
            size_class="small"
        elif [ "$mod5" -eq 4 ]; then
            size_class="large"
        else
            size_class="medium"
        fi

        local content=""

        # Front matter: 80% have it
        if [ $((i % 5)) -ne 3 ]; then
            local title
            title="$(heading "$topic" "$i") - Document $i"
            # Tags: topic tag always, plus extras based on index
            local tags="$topic"
            if [ $((i % 3)) -eq 0 ]; then
                tags="$tags, tutorial"
            fi
            if [ $((i % 7)) -eq 0 ]; then
                tags="$tags, advanced"
            fi
            content="---
title: \"$title\"
tags: $tags
---

"
        fi

        # Heading
        content+="# $(heading "$topic" "$i")

"

        # Paragraphs based on size
        local num_paragraphs
        case "$size_class" in
            small)   num_paragraphs=1;;
            medium)  num_paragraphs=4;;
            large)   num_paragraphs=8;;
        esac

        for ((p = 0; p < num_paragraphs; p++)); do
            local para_idx=$(( (i * 3 + p) % 8 ))
            if [ "$p" -gt 0 ]; then
                content+="
## $(heading "$topic" $((i + p)))

"
            fi
            content+="$(paragraph "$topic" "$para_idx")

"

            # Code blocks: 67% of files have at least one
            if [ $((i % 3)) -ne 2 ] && [ "$p" -eq 0 ]; then
                content+="$(code_block "$topic" "$i")

"
            fi
        done

        # "rust async programming" word distribution:
        # ~15% have all 3 words
        if [ $((i % 100)) -lt 15 ] || [ $((i % 7)) -eq 1 ]; then
            content+="
Rust async programming enables efficient concurrent execution without the overhead of threads.

"
        # ~15% have 2 words
        elif [ $((i % 100)) -lt 30 ] || [ $((i % 11)) -eq 0 ]; then
            content+="
Rust programming offers memory safety through its ownership system.

"
        # ~30% have 1 word
        elif [ $((i % 100)) -lt 60 ] || [ $((i % 13)) -eq 0 ]; then
            content+="
Programming paradigms influence how we approach problem solving.

"
        fi

        printf '%s' "$content" > "$DOCS_DIR/$filename"
    done

    # Needle files (always present)
    cat > "$DOCS_DIR/needle_xylophage.md" <<'NEEDLE1'
---
title: "The Xylophage Method"
tags: science
---

# Introduction

This document describes a novel approach to studying wood-boring organisms.

## The Xylophage Method

The xylophage method provides a systematic framework for analyzing insect behavior
in forest ecosystems. By observing xylophage patterns, researchers can predict
timber degradation rates with remarkable accuracy.

## Applications

This methodology has been applied to conservation efforts worldwide.
NEEDLE1

    cat > "$DOCS_DIR/needle_xylophage_body.md" <<'NEEDLE2'
---
title: "Wood-Boring Insect Study"
tags: science
---

# Forest Ecosystem Research

Understanding the role of wood-boring insects in forest ecosystems is critical
for conservation biology.

## Methodology

Our study employed field observations across twelve forest sites. The xylophage
specimens were collected using standardized trapping protocols over a six-month
period. Analysis revealed significant variation in species distribution.

## Results

Population density correlated strongly with canopy cover and deadwood availability.
NEEDLE2

    echo "  Generated $n files + 2 needles in $DOCS_DIR"
}

# ── Test Helpers ───────────────────────────────────────────────────────────

# Run a search command and capture stdout/stderr separately
# Usage: run_search "query" [extra_args...]
# Sets: STDOUT_FILE, STDERR_FILE, EXIT_CODE
run_search() {
    STDOUT_FILE="$TEST_ROOT/stdout.tmp"
    STDERR_FILE="$TEST_ROOT/stderr.tmp"
    EXIT_CODE=0
    "$MDX" search "$@" > "$STDOUT_FILE" 2> "$STDERR_FILE" || EXIT_CODE=$?
}

# Parse timing from stderr: "  N results in M files (Xms)"
# Returns milliseconds as a float string
parse_timing_ms() {
    local stderr_content
    stderr_content=$(cat "$STDERR_FILE")
    # Match patterns: (1.2ms), (345us), (1.23s)
    local timing
    timing=$(echo "$stderr_content" | grep -oE '\([0-9]+(\.[0-9]+)?(us|ms|s)\)' | tr -d '()')
    if [ -z "$timing" ]; then
        echo "0"
        return
    fi
    if echo "$timing" | grep -q 'us$'; then
        local val="${timing%us}"
        echo "$val" | awk '{printf "%.3f", $1/1000}'
    elif echo "$timing" | grep -q 'ms$'; then
        echo "${timing%ms}"
    elif echo "$timing" | grep -q 's$'; then
        local val="${timing%s}"
        echo "$val" | awk '{printf "%.1f", $1*1000}'
    else
        echo "0"
    fi
}

# Parse result count from stderr
parse_result_count() {
    local stderr_content
    stderr_content=$(cat "$STDERR_FILE")
    echo "$stderr_content" | grep -oE '^[[:space:]]*[0-9]+ results' | grep -oE '[0-9]+' || echo "0"
}

# Get first file from stdout (for ranking checks)
parse_top_file() {
    # In normal mode, first line is the file path (possibly with ANSI codes)
    head -1 "$STDOUT_FILE" | sed 's/\x1b\[[0-9;]*m//g' | awk '{print $1}' | xargs basename 2>/dev/null || echo ""
}

# Record a test result to CSV and print status
record() {
    local scale="$1" test_name="$2" time_ms="$3" results="$4" top_file="$5" pass="$6" peak_rss="$7"
    echo "$scale,$test_name,$time_ms,$results,$top_file,$pass,$peak_rss" >> "$RESULTS_CSV"
    local status_color="$GREEN"
    if [ "$pass" = "FAIL" ]; then
        status_color="$RED"
    elif [ "$pass" = "-" ]; then
        status_color="$YELLOW"
    fi
    printf "  %-16s %6sms  %4s results  %-24s ${status_color}%s${RESET}\n" \
        "$test_name" "$time_ms" "$results" "$top_file" "$pass"
}

# ── Test Functions ─────────────────────────────────────────────────────────

test_common_term() {
    local scale="$1"
    run_search "optimization" "$DOCS_DIR" -n 20
    local time_ms results pass
    time_ms=$(parse_timing_ms)
    results=$(parse_result_count)
    if [ "$results" -gt 0 ]; then
        pass="-"
    else
        pass="FAIL"
    fi
    record "$scale" "common_term" "$time_ms" "$results" "-" "$pass" "-"
}

test_needle() {
    local scale="$1"
    run_search "xylophage" "$DOCS_DIR" -n 10
    local time_ms results top_file pass
    time_ms=$(parse_timing_ms)
    results=$(parse_result_count)
    top_file=$(parse_top_file)

    if [ "$results" -eq 2 ] && [ "$top_file" = "needle_xylophage.md" ]; then
        pass="PASS"
    else
        pass="FAIL"
    fi
    record "$scale" "needle" "$time_ms" "$results" "$top_file" "$pass" "-"
}

test_heading_vs_body() {
    local scale="$1"
    # Re-use the needle search results (already captured)
    run_search "xylophage" "$DOCS_DIR" -n 10
    local time_ms results pass

    time_ms=$(parse_timing_ms)
    results=$(parse_result_count)

    # Check that heading file ranks above body-only file
    local stdout_content
    stdout_content=$(sed 's/\x1b\[[0-9;]*m//g' < "$STDOUT_FILE")
    local heading_pos body_pos
    heading_pos=$(echo "$stdout_content" | grep -n "needle_xylophage\.md" | head -1 | cut -d: -f1)
    body_pos=$(echo "$stdout_content" | grep -n "needle_xylophage_body\.md" | head -1 | cut -d: -f1)

    if [ -n "$heading_pos" ] && [ -n "$body_pos" ] && [ "$heading_pos" -lt "$body_pos" ]; then
        pass="PASS"
    else
        pass="FAIL"
    fi
    record "$scale" "heading_vs_body" "$time_ms" "$results" "-" "$pass" "-"
}

test_multi_word() {
    local scale="$1"
    run_search "rust async programming" "$DOCS_DIR" -n 20
    local time_ms results pass
    time_ms=$(parse_timing_ms)
    results=$(parse_result_count)

    if [ "$results" -gt 0 ]; then
        # Check that files with all 3 words rank high - get the top file
        local top_file
        top_file=$(parse_top_file)

        # Verify top result contains all 3 query words
        if [ -n "$top_file" ] && [ -f "$DOCS_DIR/$top_file" ]; then
            local content
            content=$(tr '[:upper:]' '[:lower:]' < "$DOCS_DIR/$top_file")
            if echo "$content" | grep -q "rust" && \
               echo "$content" | grep -q "async" && \
               echo "$content" | grep -q "programming"; then
                pass="PASS"
            else
                pass="FAIL"
            fi
        else
            pass="-"
        fi
    else
        pass="FAIL"
    fi
    record "$scale" "multi_word" "$time_ms" "$results" "-" "$pass" "-"
}

test_tag_filter() {
    local scale="$1"
    run_search "optimization" "$DOCS_DIR" --tag cooking -n 20
    local time_ms results pass
    time_ms=$(parse_timing_ms)
    results=$(parse_result_count)

    # Verify all results are cooking-tagged
    pass="PASS"
    if [ "$results" -gt 0 ]; then
        # Check files-only output for verification
        run_search "optimization" "$DOCS_DIR" --tag cooking -l -n 50
        while IFS= read -r filepath; do
            if [ -n "$filepath" ] && [ -f "$filepath" ]; then
                if ! grep -qi 'tags:.*cooking' "$filepath"; then
                    pass="FAIL"
                    break
                fi
            fi
        done < "$STDOUT_FILE"
    elif [ "$EXIT_CODE" -ne 0 ]; then
        # If no cooking files have "optimization", that's OK if there truly aren't any
        pass="-"
    fi
    record "$scale" "tag_filter" "$time_ms" "$results" "-" "$pass" "-"
}

test_files_only() {
    local scale="$1"
    run_search "optimization" "$DOCS_DIR" -l -n 50
    local time_ms results pass
    time_ms=$(parse_timing_ms)
    results=$(parse_result_count)

    # Verify: one path per line, no snippet text
    pass="PASS"
    if [ "$results" -gt 0 ]; then
        local line_count
        line_count=$(wc -l < "$STDOUT_FILE" | tr -d ' ')
        # Each result should be exactly one line (a file path)
        # Check no lines contain snippet-like content (indented text)
        if grep -qE '^\s{2,}' "$STDOUT_FILE"; then
            pass="FAIL"
        fi
    else
        pass="FAIL"
    fi
    record "$scale" "files_only" "$time_ms" "$results" "-" "$pass" "-"
}

test_no_results() {
    local scale="$1"
    run_search "zzzyyyxxx_nonexistent" "$DOCS_DIR"
    local stderr_content stdout_content pass
    stderr_content=$(cat "$STDERR_FILE")
    stdout_content=$(cat "$STDOUT_FILE")

    if echo "$stderr_content" | grep -q "No results found" && [ -z "$stdout_content" ]; then
        pass="PASS"
    else
        pass="FAIL"
    fi
    record "$scale" "no_results" "0" "0" "-" "$pass" "-"
}

test_timing() {
    local scale="$1"
    local times=()

    for run in 1 2 3; do
        run_search "optimization" "$DOCS_DIR" -n 20
        local t
        t=$(parse_timing_ms)
        times+=("$t")
    done

    # Compute min/avg/max
    local min avg max
    min=$(printf '%s\n' "${times[@]}" | sort -n | head -1)
    max=$(printf '%s\n' "${times[@]}" | sort -n | tail -1)
    avg=$(printf '%s\n' "${times[@]}" | awk '{s+=$1} END {printf "%.1f", s/NR}')

    record "$scale" "timing_min" "$min" "-" "-" "-" "-"
    record "$scale" "timing_avg" "$avg" "-" "-" "-" "-"
    record "$scale" "timing_max" "$max" "-" "-" "-" "-"
}

test_memory() {
    local scale="$1"
    local time_output
    time_output=$(/usr/bin/time -l "$MDX" search "optimization" "$DOCS_DIR" -n 20 2>&1 >/dev/null || true)

    # Parse peak RSS from /usr/bin/time -l output (macOS format)
    # The line is: "  N  maximum resident set size" (bytes on macOS)
    local peak_rss_bytes
    peak_rss_bytes=$(echo "$time_output" | grep -i 'maximum resident set size' | awk '{print $1}' || echo "0")
    local peak_rss_kb
    if [ -n "$peak_rss_bytes" ] && [ "$peak_rss_bytes" -gt 0 ] 2>/dev/null; then
        peak_rss_kb=$((peak_rss_bytes / 1024))
    else
        peak_rss_kb="0"
    fi

    record "$scale" "memory" "-" "-" "-" "-" "$peak_rss_kb"
}

# ── Phase 2: Run Tests ────────────────────────────────────────────────────

echo ""
echo -e "${BOLD}Phase 2: Running test suite${RESET}"
echo ""

for scale in "${SCALES[@]}"; do
    echo -e "${CYAN}━━━ Scale: $scale files ━━━${RESET}"
    generate_files "$scale"

    test_common_term "$scale"
    test_needle "$scale"
    test_heading_vs_body "$scale"
    test_multi_word "$scale"
    test_tag_filter "$scale"
    test_files_only "$scale"
    test_no_results "$scale"
    test_timing "$scale"
    test_memory "$scale"

    echo ""
done

# ── Phase 3: Summary Report ───────────────────────────────────────────────

echo -e "${BOLD}Phase 3: Summary Report${RESET}"
echo ""

# Print formatted table
printf "${BOLD}%-7s  %-16s  %10s  %7s  %-24s  %-5s  %10s${RESET}\n" \
    "Files" "Test" "Time(avg)" "Results" "#1 File" "Pass?" "Peak RSS"
printf "%-7s  %-16s  %10s  %7s  %-24s  %-5s  %10s\n" \
    "-------" "----------------" "----------" "-------" "------------------------" "-----" "----------"

print_summary_table() {
    read -r  # skip header
    while IFS=, read -r scale test_name time_ms results top_file pass peak_rss; do
        local time_display="${time_ms}ms"
        if [ "$time_ms" = "-" ]; then
            time_display="-"
        fi
        local rss_display="$peak_rss"
        if [ "$peak_rss" != "-" ] && [ "$peak_rss" != "0" ]; then
            rss_display="${peak_rss}KB"
        fi
        local pass_color=""
        local pass_reset=""
        if [ "$pass" = "PASS" ]; then
            pass_color="$GREEN"
            pass_reset="$RESET"
        elif [ "$pass" = "FAIL" ]; then
            pass_color="$RED"
            pass_reset="$RESET"
        fi
        printf "%-7s  %-16s  %10s  %7s  %-24s  ${pass_color}%-5s${pass_reset}  %10s\n" \
            "$scale" "$test_name" "$time_display" "$results" "$top_file" "$pass" "$rss_display"
    done
}
print_summary_table < "$RESULTS_CSV"

echo ""

# ── Scaling Analysis ──────────────────────────────────────────────────────

echo -e "${BOLD}Scaling Analysis${RESET}"
echo ""

# Extract timing_avg for each scale
echo "Timing progression (avg ms):"
prev_scale=""
prev_time=""
for scale in "${SCALES[@]}"; do
    avg_time=$(grep "^${scale},timing_avg," "$RESULTS_CSV" | cut -d, -f3)
    if [ -n "$avg_time" ] && [ "$avg_time" != "0" ]; then
        if [ -n "$prev_time" ] && [ "$prev_time" != "0" ]; then
            ratio=$(echo "$avg_time $prev_time" | awk '{printf "%.2f", $1/$2}')
            scale_ratio=$(echo "$scale $prev_scale" | awk '{printf "%.2f", $1/$2}')
            echo "  $scale files: ${avg_time}ms  (${ratio}x vs $prev_scale files, scale factor ${scale_ratio}x)"
        else
            echo "  $scale files: ${avg_time}ms"
        fi
        prev_scale="$scale"
        prev_time="$avg_time"
    fi
done

echo ""
echo "Memory progression:"
for scale in "${SCALES[@]}"; do
    rss=$(grep "^${scale},memory," "$RESULTS_CSV" | cut -d, -f7)
    if [ -n "$rss" ] && [ "$rss" != "0" ]; then
        per_file=$(echo "$rss $scale" | awk '{printf "%.1f", $1/$2}')
        echo "  $scale files: ${rss}KB peak RSS  (${per_file}KB/file)"
    fi
done

echo ""

# Check for failures
failures=$(grep 'FAIL' "$RESULTS_CSV" || true)
if [ -n "$failures" ]; then
    echo -e "${RED}${BOLD}FAILURES DETECTED:${RESET}"
    echo "$failures" | while IFS=, read -r scale test_name rest; do
        echo -e "  ${RED}$test_name at scale $scale${RESET}"
    done
    echo ""
    exit 1
else
    echo -e "${GREEN}${BOLD}All tests passed across all scales.${RESET}"
fi

# Cleanup
rm -rf "$TEST_ROOT"
echo ""
echo "Stress test complete. Temp files cleaned up."
