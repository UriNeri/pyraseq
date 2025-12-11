# paraseq_filt

Parallel FASTA/FASTQ filtering tool with Rust CLI and Python bindings.

## Features

- Parallel processing using the `paraseq` crate
- Gzip input support (via niffler)
- HashSet-based lookups (O(1))
- Invert mode to exclude sequences
- Python bindings with full type hints
- CLI and library interfaces

## Installation

**Using pixi (recommended):**
```bash
pixi run build-cli      # Rust CLI
pixi run build-python   # Python bindings
```

**Without pixi:**
```bash
cargo build --release           # CLI: target/release/paraseq_filt
pip install maturin && maturin develop --release  # Python
```

## Usage

### CLI
```bash
# Count mode (for benchmarking)
paraseq_filt --count -i input.fasta

# Filter mode
paraseq_filt -i input.fasta -o output.fasta -H headers.txt
paraseq_filt -i input.fasta -o output.fasta -H "seq1,seq2,seq3"
paraseq_filt -i input.fasta -o output.fasta -H headers.txt --invert -t 8
```

### Python
```python
import paraseq_filt

processed, written = paraseq_filt.filter_fasta_by_headers(
    "input.fasta",
    "output.fasta",
    ["seq1", "seq2", "seq3"],
    num_threads=8
)

# Or from file
processed, written = paraseq_filt.filter_fasta_by_headers(
    "input.fasta.gz",
    "output.fasta",
    "headers.txt",
    invert=True
)
```

## Options

```
-i, --input <FILE>      Input file (supports .gz)
-o, --output <FILE>     Output file (required for filter mode)
-H, --headers <LIST>    Comma-separated IDs or file path (required for filter mode)
-v, --invert            Exclude matching sequences
-t, --threads <N>       Thread count (default: CPU count)
-c, --count             Count mode: just count reads and bases
```

## Python API Reference

```python
filter_fasta_by_headers(
    input_file: str | Path,
    output_file: str | Path,
    headers: list[str] | str | Path,
    invert: bool = False,
    num_threads: int | None = None,
) -> tuple[int, int]  # (processed, written)

load_headers_from_file(file_path: str | Path) -> list[str]
```

Full type stubs included for IDE support and type checking.

## Benchmarking

For integration with [biofaster](https://github.com/UriNeri/biofaster), see [BIOFASTER_INTEGRATION.md](BIOFASTER_INTEGRATION.md).

Count mode benchmark:
```bash
paraseq_filt --count -i large_file.fasta.gz -t 32
```

## Testing

```bash
pixi run test           # All tests
pixi run test-rust      # CLI tests
bash tests/test_count.sh  # Count mode tests
pixi run test-python    # Python tests
```

Or manually: `bash tests/test.sh` and `python tests/test_python.py`
