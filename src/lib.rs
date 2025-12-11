use pyo3::prelude::*;
use pyo3::exceptions::PyRuntimeError;
use paraseq::prelude::*;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct FastaFilterPy {
    headers: Arc<HashSet<String>>,
    writer: Arc<Mutex<BufWriter<File>>>,
    invert: bool,
    processed: Arc<Mutex<u64>>,
    written: Arc<Mutex<u64>>,
}

impl<R: Record> ParallelProcessor<R> for FastaFilterPy {
    fn process_record(&mut self, record: R) -> Result<(), paraseq::ProcessError> {
        let id = record.id_str().to_string();
        let seq_bytes = record.seq();
        let seq = std::str::from_utf8(&seq_bytes)
            .map_err(|e| paraseq::ProcessError::Process(Box::new(e)))?;

        // Update processed count
        {
            let mut count = self.processed.lock().unwrap();
            *count += 1;
        }

        let should_write = if self.invert {
            !self.headers.contains(&id)
        } else {
            self.headers.contains(&id)
        };

        if should_write {
            let mut writer = self.writer.lock().unwrap();
            writeln!(writer, ">{}\n{}", id, seq)
                .map_err(|e| paraseq::ProcessError::IoError(e))?;
            
            let mut count = self.written.lock().unwrap();
            *count += 1;
        }

        Ok(())
    }
}

/// Filter a FASTA/FASTQ file by sequence IDs
///
/// Args:
///     input_file (str): Path to input FASTA/FASTQ file (supports .gz)
///     output_file (str): Path to output FASTA/FASTQ file
///     headers (list[str]): List of sequence IDs to filter
///     invert (bool): If True, keep sequences NOT in the headers list. Default: False
///     num_threads (int | None): Number of threads to use. Default: number of CPUs
///
/// Returns:
///     dict: Dictionary with 'processed' and 'written' record counts
///
/// Example:
///     >>> import paraseq_filt
///     >>> result = paraseq_filt.filter_fasta_by_headers(
///     ...     "input.fasta",
///     ...     "output.fasta",
///     ...     ["seq1", "seq2", "seq3"],
///     ...     invert=False,
///     ...     num_threads=4
///     ... )
///     >>> print(f"Processed {result['processed']}, wrote {result['written']}")
#[pyfunction]
#[pyo3(signature = (input_file, output_file, headers, invert=false, num_threads=None))]
fn filter_fasta_by_headers(
    input_file: &str,
    output_file: &str,
    headers: Vec<String>,
    invert: bool,
    num_threads: Option<usize>,
) -> PyResult<(u64, u64)> {
    // Convert headers to HashSet
    let headers_set: HashSet<String> = headers.into_iter().collect();
    
    // Get number of threads
    let num_threads = num_threads.unwrap_or_else(num_cpus::get);
    
    // Open output file
    let output_path = Path::new(output_file);
    let output_file = File::create(output_path)
        .map_err(|e| PyRuntimeError::new_err(format!("Failed to create output file: {}", e)))?;
    let writer = BufWriter::new(output_file);
    
    // Create processor
    let mut processor = FastaFilterPy {
        headers: Arc::new(headers_set),
        writer: Arc::new(Mutex::new(writer)),
        invert,
        processed: Arc::new(Mutex::new(0)),
        written: Arc::new(Mutex::new(0)),
    };
    
    // Process FASTA file
    let reader = paraseq::fastx::Reader::from_path(input_file)
        .map_err(|e| PyRuntimeError::new_err(format!("Failed to open input file: {}", e)))?;
    
    reader
        .process_parallel(&mut processor, num_threads)
        .map_err(|e| PyRuntimeError::new_err(format!("Failed to process file: {:?}", e)))?;
    
    let total_processed = *processor.processed.lock().unwrap();
    let total_written = *processor.written.lock().unwrap();
    
    Ok((total_processed, total_written))
}

/// Load sequence IDs from a file (one per line)
///
/// Args:
///     file_path (str): Path to file containing sequence IDs (one per line)
///
/// Returns:
///     list[str]: List of sequence IDs
///
/// Example:
///     >>> import paraseq_filt
///     >>> headers = paraseq_filt.load_headers_from_file("headers.txt")
///     >>> print(f"Loaded {len(headers)} headers")
#[pyfunction]
fn load_headers_from_file(file_path: &str) -> PyResult<Vec<String>> {
    use std::io::{BufRead, BufReader};
    
    let file = File::open(file_path)
        .map_err(|e| PyRuntimeError::new_err(format!("Failed to open file: {}", e)))?;
    let reader = BufReader::new(file);
    
    let mut headers = Vec::new();
    for line in reader.lines() {
        let line = line.map_err(|e| PyRuntimeError::new_err(format!("Failed to read line: {}", e)))?;
        let header = line.trim();
        if !header.is_empty() {
            headers.push(header.to_string());
        }
    }
    
    Ok(headers)
}

/// Fast parallel FASTA/FASTQ filtering using Rust and paraseq
#[pymodule]
fn paraseq_filt(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(filter_fasta_by_headers, m)?)?;
    m.add_function(wrap_pyfunction!(load_headers_from_file, m)?)?;
    Ok(())
}
