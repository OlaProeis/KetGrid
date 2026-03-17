//! OpenQASM 2.0 export and import for quantum circuits.
//!
//! This module exports KetGrid circuits to OpenQASM 2.0 format and parses
//! OpenQASM 2.0 files back into circuits. OpenQASM 2.0 is the standard
//! quantum assembly language used by IBM and the quantum computing community.

use crate::circuit::{Circuit, Measurement, PlacedGate};
use crate::gate::GateType;

use nom::{
    IResult,
    branch::alt,
    bytes::complete::{tag, tag_no_case, take_while, take_while1},
    character::complete::{char, digit1, multispace0, multispace1},
    combinator::{map_res, opt, recognize, value},
    multi::{separated_list0, separated_list1},
    number::complete::double,
    sequence::{delimited, pair, tuple},
};

/// Error type for OpenQASM export operations.
#[derive(Debug, Clone, PartialEq)]
pub enum QasmError {
    /// Circuit has no qubits.
    EmptyCircuit,
    /// Invalid gate configuration.
    InvalidGate(String),
    /// Error parsing OpenQASM input.
    ParseError(String),
}

impl std::fmt::Display for QasmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QasmError::EmptyCircuit => write!(f, "Cannot export empty circuit (no qubits)"),
            QasmError::InvalidGate(msg) => write!(f, "Invalid gate configuration: {}", msg),
            QasmError::ParseError(msg) => write!(f, "OpenQASM parse error: {}", msg),
        }
    }
}

impl std::error::Error for QasmError {}

/// Generate OpenQASM 2.0 code for a circuit.
///
/// The generated code follows the OpenQASM 2.0 standard with the standard
/// library include (qelib1.inc). Gates are sorted by column for proper
/// execution order.
///
/// # Arguments
/// * `circuit` - The circuit to export.
///
/// # Returns
/// * `Ok(String)` - The generated OpenQASM 2.0 code.
/// * `Err(QasmError)` - If the circuit cannot be exported.
///
/// # Example
/// ```
/// use ketgrid_core::{Circuit, GateType};
/// use ketgrid_core::format::qasm::circuit_to_qasm;
///
/// let mut circuit = Circuit::new(2);
/// circuit.add_gate(GateType::H, vec![0], vec![], 0).unwrap();
/// circuit.add_gate(GateType::Cnot, vec![1], vec![0], 1).unwrap();
///
/// let qasm_code = circuit_to_qasm(&circuit).unwrap();
/// assert!(qasm_code.contains("OPENQASM 2.0"));
/// assert!(qasm_code.contains("h q[0]"));
/// assert!(qasm_code.contains("cx q[0],q[1]"));
/// ```
pub fn circuit_to_qasm(circuit: &Circuit) -> Result<String, QasmError> {
    if circuit.num_qubits() == 0 {
        return Err(QasmError::EmptyCircuit);
    }

    let mut lines = Vec::new();

    // Header
    lines.push("OPENQASM 2.0;".to_string());
    lines.push("include \"qelib1.inc\";".to_string());
    lines.push(String::new());

    // Count measurements to determine classical bits needed
    let num_measurements = circuit.measurements.len();
    let num_qubits = circuit.num_qubits();

    // Register declarations
    lines.push(format!("qreg q[{}];", num_qubits));
    if num_measurements > 0 {
        lines.push(format!("creg c[{}];", num_measurements));
    }
    lines.push(String::new());

    // Get gates sorted by column (execution order)
    let sorted_gates = circuit.gates_by_column();

    // Generate gate code
    for gate in sorted_gates {
        if let Some(gate_line) = generate_gate_code(gate) {
            lines.push(gate_line);
        }
    }

        // Generate measurement code
        if !circuit.measurements.is_empty() {
            lines.push(String::new());

            // Sort measurements by column for consistent ordering
            let mut sorted_measurements: Vec<&Measurement> = circuit.measurements.iter().collect();
            sorted_measurements.sort_by_key(|m| m.column);

            // Build qubit to measurement index mapping
            for (idx, measurement) in sorted_measurements.iter().enumerate() {
                lines.push(format!(
                    "measure q[{}] -> c[{}];",
                    measurement.qubit_id, idx
                ));
            }
        }

        // Join lines and trim trailing newlines
        Ok(lines.join("\n").trim_end().to_string())
}

/// Generate OpenQASM code for a single gate.
///
/// Returns `Some(String)` with the code line, or `None` if the gate
/// should be skipped (e.g., Identity gates).
fn generate_gate_code(gate: &PlacedGate) -> Option<String> {
    match &gate.gate {
        // Single-qubit gates (no parameters)
        GateType::H => {
            let target = gate.target_qubits.first()?;
            Some(format!("h q[{}];", target))
        }
        GateType::X => {
            let target = gate.target_qubits.first()?;
            Some(format!("x q[{}];", target))
        }
        GateType::Y => {
            let target = gate.target_qubits.first()?;
            Some(format!("y q[{}];", target))
        }
        GateType::Z => {
            let target = gate.target_qubits.first()?;
            Some(format!("z q[{}];", target))
        }
        GateType::S => {
            let target = gate.target_qubits.first()?;
            Some(format!("s q[{}];", target))
        }
        GateType::T => {
            let target = gate.target_qubits.first()?;
            Some(format!("t q[{}];", target))
        }

        // Parameterized single-qubit gates
        GateType::Rx(theta) => {
            let target = gate.target_qubits.first()?;
            Some(format!("rx({:.6}) q[{}];", theta, target))
        }
        GateType::Ry(theta) => {
            let target = gate.target_qubits.first()?;
            Some(format!("ry({:.6}) q[{}];", theta, target))
        }
        GateType::Rz(theta) => {
            let target = gate.target_qubits.first()?;
            Some(format!("rz({:.6}) q[{}];", theta, target))
        }

        // Controlled gates
        GateType::Cnot => {
            let control = gate.control_qubits.first()?;
            let target = gate.target_qubits.first()?;
            Some(format!("cx q[{}],q[{}];", control, target))
        }
        GateType::Cz => {
            let control = gate.control_qubits.first()?;
            let target = gate.target_qubits.first()?;
            Some(format!("cz q[{}],q[{}];", control, target))
        }

        // Multi-qubit gates
        GateType::Swap => {
            let q1 = gate.target_qubits.get(0)?;
            let q2 = gate.target_qubits.get(1)?;
            Some(format!("swap q[{}],q[{}];", q1, q2))
        }
        GateType::Toffoli => {
            let ctrl1 = gate.control_qubits.get(0)?;
            let ctrl2 = gate.control_qubits.get(1)?;
            let target = gate.target_qubits.first()?;
            Some(format!("ccx q[{}],q[{}],q[{}];", ctrl1, ctrl2, target))
        }

        // Meta gates
        GateType::Barrier => {
            let qubits: Vec<String> = gate
                .target_qubits
                .iter()
                .map(|q| format!("q[{}]", q))
                .collect();
            Some(format!("barrier {};", qubits.join(",")))
        }

        // Identity / no-op - skip
        GateType::Identity => None,

        // Custom gates - generate as comment
        GateType::Custom(name) => {
            let target = gate.target_qubits.first()?;
            Some(format!(
                "// Custom gate: {} on q[{}] (requires manual implementation)",
                name, target
            ))
        }
    }
}

// ===== Import: OpenQASM 2.0 Parser =====

/// Result of importing an OpenQASM 2.0 file.
#[derive(Debug, Clone)]
pub struct QasmImportResult {
    /// The parsed circuit.
    pub circuit: Circuit,
    /// Warnings for skipped or unsupported elements.
    pub warnings: Vec<String>,
}

/// Parsed QASM statement (internal representation).
#[allow(dead_code)]
enum QasmStmt<'a> {
    Header(f64),
    Include(&'a str),
    Qreg(&'a str, usize),
    Creg(&'a str, usize),
    Gate {
        name: &'a str,
        params: Vec<f64>,
        qubits: Vec<(&'a str, usize)>,
    },
    Measure {
        qubit: (&'a str, usize),
        cbit: (&'a str, usize),
    },
    Barrier(Vec<(&'a str, usize)>),
}

// --- nom parsers (private) ---

fn ident(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        take_while1(|c: char| c.is_alphabetic() || c == '_'),
        take_while(|c: char| c.is_alphanumeric() || c == '_'),
    ))(input)
}

fn reg_ref(input: &str) -> IResult<&str, (&str, usize)> {
    let (input, name) = ident(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = char('[')(input)?;
    let (input, _) = multispace0(input)?;
    let (input, idx) = map_res(digit1, |s: &str| s.parse::<usize>())(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = char(']')(input)?;
    Ok((input, (name, idx)))
}

// --- Expression parser for gate parameters (supports pi, arithmetic) ---

fn param_expr(input: &str) -> IResult<&str, f64> {
    let (input, _) = multispace0(input)?;
    add_expr(input)
}

fn add_expr(input: &str) -> IResult<&str, f64> {
    let (mut input, mut result) = mul_expr(input)?;
    loop {
        let (rest, _) = multispace0(input)?;
        if let Ok((rest, _)) = char::<_, nom::error::Error<&str>>('+')(rest) {
            let (rest, _) = multispace0(rest)?;
            let (rest, rhs) = mul_expr(rest)?;
            result += rhs;
            input = rest;
        } else if let Ok((rest, _)) = char::<_, nom::error::Error<&str>>('-')(rest) {
            let (rest, _) = multispace0(rest)?;
            let (rest, rhs) = mul_expr(rest)?;
            result -= rhs;
            input = rest;
        } else {
            break;
        }
    }
    Ok((input, result))
}

fn mul_expr(input: &str) -> IResult<&str, f64> {
    let (mut input, mut result) = unary_expr(input)?;
    loop {
        let (rest, _) = multispace0(input)?;
        if let Ok((rest, _)) = char::<_, nom::error::Error<&str>>('*')(rest) {
            let (rest, _) = multispace0(rest)?;
            let (rest, rhs) = unary_expr(rest)?;
            result *= rhs;
            input = rest;
        } else if let Ok((rest, _)) = char::<_, nom::error::Error<&str>>('/')(rest) {
            let (rest, _) = multispace0(rest)?;
            let (rest, rhs) = unary_expr(rest)?;
            result /= rhs;
            input = rest;
        } else {
            break;
        }
    }
    Ok((input, result))
}

fn unary_expr(input: &str) -> IResult<&str, f64> {
    let (input, _) = multispace0(input)?;
    if let Ok((rest, _)) = char::<_, nom::error::Error<&str>>('-')(input) {
        let (rest, val) = primary_expr(rest)?;
        Ok((rest, -val))
    } else {
        primary_expr(input)
    }
}

fn primary_expr(input: &str) -> IResult<&str, f64> {
    let (input, _) = multispace0(input)?;
    alt((
        value(std::f64::consts::PI, tag("pi")),
        double,
        delimited(char('('), param_expr, char(')')),
    ))(input)
}

// --- Statement parsers (operate on text between semicolons) ---

fn parse_header(input: &str) -> IResult<&str, QasmStmt<'_>> {
    let (input, _) = tag_no_case("OPENQASM")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, version) = double(input)?;
    Ok((input, QasmStmt::Header(version)))
}

fn parse_include(input: &str) -> IResult<&str, QasmStmt<'_>> {
    let (input, _) = tag("include")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, _) = char('"')(input)?;
    let (input, file) = take_while(|c: char| c != '"')(input)?;
    let (input, _) = char('"')(input)?;
    Ok((input, QasmStmt::Include(file)))
}

fn parse_qreg(input: &str) -> IResult<&str, QasmStmt<'_>> {
    let (input, _) = tag("qreg")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, (name, size)) = reg_ref(input)?;
    Ok((input, QasmStmt::Qreg(name, size)))
}

fn parse_creg(input: &str) -> IResult<&str, QasmStmt<'_>> {
    let (input, _) = tag("creg")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, (name, size)) = reg_ref(input)?;
    Ok((input, QasmStmt::Creg(name, size)))
}

fn parse_measure_stmt(input: &str) -> IResult<&str, QasmStmt<'_>> {
    let (input, _) = tag("measure")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, qubit) = reg_ref(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("->")(input)?;
    let (input, _) = multispace0(input)?;
    let (input, cbit) = reg_ref(input)?;
    Ok((input, QasmStmt::Measure { qubit, cbit }))
}

fn parse_barrier_stmt(input: &str) -> IResult<&str, QasmStmt<'_>> {
    let (input, _) = tag("barrier")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, qubits) = separated_list1(
        tuple((multispace0, char(','), multispace0)),
        reg_ref,
    )(input)?;
    Ok((input, QasmStmt::Barrier(qubits)))
}

fn parse_gate_stmt(input: &str) -> IResult<&str, QasmStmt<'_>> {
    let (input, name) = ident(input)?;
    let (input, params) = opt(delimited(
        char('('),
        separated_list0(tuple((multispace0, char(','), multispace0)), param_expr),
        char(')'),
    ))(input)?;
    let params = params.unwrap_or_default();
    let (input, _) = multispace0(input)?;
    let (input, qubits) = separated_list1(
        tuple((multispace0, char(','), multispace0)),
        reg_ref,
    )(input)?;
    Ok((input, QasmStmt::Gate { name, params, qubits }))
}

fn try_parse_statement(input: &str) -> Result<QasmStmt<'_>, String> {
    if let Ok((rest, stmt)) = parse_header(input) {
        if rest.trim().is_empty() { return Ok(stmt); }
    }
    if let Ok((rest, stmt)) = parse_include(input) {
        if rest.trim().is_empty() { return Ok(stmt); }
    }
    if let Ok((rest, stmt)) = parse_qreg(input) {
        if rest.trim().is_empty() { return Ok(stmt); }
    }
    if let Ok((rest, stmt)) = parse_creg(input) {
        if rest.trim().is_empty() { return Ok(stmt); }
    }
    if let Ok((rest, stmt)) = parse_measure_stmt(input) {
        if rest.trim().is_empty() { return Ok(stmt); }
    }
    if let Ok((rest, stmt)) = parse_barrier_stmt(input) {
        if rest.trim().is_empty() { return Ok(stmt); }
    }
    if let Ok((rest, stmt)) = parse_gate_stmt(input) {
        if rest.trim().is_empty() { return Ok(stmt); }
    }
    Err(format!(
        "Unrecognized: '{}'",
        &input[..input.len().min(60)]
    ))
}

fn strip_comments(input: &str) -> String {
    input
        .lines()
        .map(|line| {
            if let Some(pos) = line.find("//") {
                &line[..pos]
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Map an OpenQASM gate name + params + resolved qubit indices to a
/// `(GateType, targets, controls)` triple. Returns `Err(warning)` for
/// unsupported gates.
fn map_gate_to_circuit(
    name: &str,
    params: &[f64],
    qubits: &[usize],
) -> Result<(GateType, Vec<usize>, Vec<usize>), String> {
    match name {
        "h" => {
            if qubits.len() != 1 {
                return Err(format!("h expects 1 qubit, got {}", qubits.len()));
            }
            Ok((GateType::H, vec![qubits[0]], vec![]))
        }
        "x" => {
            if qubits.len() != 1 {
                return Err(format!("x expects 1 qubit, got {}", qubits.len()));
            }
            Ok((GateType::X, vec![qubits[0]], vec![]))
        }
        "y" => {
            if qubits.len() != 1 {
                return Err(format!("y expects 1 qubit, got {}", qubits.len()));
            }
            Ok((GateType::Y, vec![qubits[0]], vec![]))
        }
        "z" => {
            if qubits.len() != 1 {
                return Err(format!("z expects 1 qubit, got {}", qubits.len()));
            }
            Ok((GateType::Z, vec![qubits[0]], vec![]))
        }
        "s" => {
            if qubits.len() != 1 {
                return Err(format!("s expects 1 qubit, got {}", qubits.len()));
            }
            Ok((GateType::S, vec![qubits[0]], vec![]))
        }
        "t" => {
            if qubits.len() != 1 {
                return Err(format!("t expects 1 qubit, got {}", qubits.len()));
            }
            Ok((GateType::T, vec![qubits[0]], vec![]))
        }
        "id" => {
            if qubits.len() != 1 {
                return Err(format!("id expects 1 qubit, got {}", qubits.len()));
            }
            Ok((GateType::Identity, vec![qubits[0]], vec![]))
        }
        "rx" => {
            if qubits.len() != 1 || params.len() != 1 {
                return Err("rx expects 1 qubit and 1 parameter".into());
            }
            Ok((GateType::Rx(params[0]), vec![qubits[0]], vec![]))
        }
        "ry" => {
            if qubits.len() != 1 || params.len() != 1 {
                return Err("ry expects 1 qubit and 1 parameter".into());
            }
            Ok((GateType::Ry(params[0]), vec![qubits[0]], vec![]))
        }
        "rz" | "p" | "u1" => {
            if qubits.len() != 1 || params.len() != 1 {
                return Err(format!("{} expects 1 qubit and 1 parameter", name));
            }
            Ok((GateType::Rz(params[0]), vec![qubits[0]], vec![]))
        }
        "cx" | "CX" => {
            if qubits.len() != 2 {
                return Err(format!("cx expects 2 qubits, got {}", qubits.len()));
            }
            Ok((GateType::Cnot, vec![qubits[1]], vec![qubits[0]]))
        }
        "cz" => {
            if qubits.len() != 2 {
                return Err(format!("cz expects 2 qubits, got {}", qubits.len()));
            }
            Ok((GateType::Cz, vec![qubits[1]], vec![qubits[0]]))
        }
        "swap" => {
            if qubits.len() != 2 {
                return Err(format!("swap expects 2 qubits, got {}", qubits.len()));
            }
            Ok((GateType::Swap, vec![qubits[0], qubits[1]], vec![]))
        }
        "ccx" => {
            if qubits.len() != 3 {
                return Err(format!("ccx expects 3 qubits, got {}", qubits.len()));
            }
            Ok((
                GateType::Toffoli,
                vec![qubits[2]],
                vec![qubits[0], qubits[1]],
            ))
        }
        _ => Err(format!("Skipping unsupported gate '{}'", name)),
    }
}

/// Parse an OpenQASM 2.0 string into a Circuit.
///
/// Supports the standard qelib1.inc gate set: h, x, y, z, s, t, rx, ry, rz,
/// cx, cz, swap, ccx, barrier, and measure. Unsupported gates are skipped
/// with warnings. Parameter expressions support `pi`, arithmetic (`+`, `-`,
/// `*`, `/`), and parenthesized sub-expressions.
///
/// # Arguments
/// * `input` - OpenQASM 2.0 source code string.
///
/// # Returns
/// * `Ok(QasmImportResult)` - The parsed circuit and any warnings.
/// * `Err(QasmError)` - If parsing fails critically (no qubits, bad register ref).
///
/// # Example
/// ```
/// use ketgrid_core::format::qasm::circuit_from_qasm;
///
/// let qasm = "OPENQASM 2.0;\ninclude \"qelib1.inc\";\nqreg q[2];\nh q[0];\ncx q[0],q[1];";
/// let result = circuit_from_qasm(qasm).unwrap();
/// assert_eq!(result.circuit.num_qubits(), 2);
/// assert_eq!(result.circuit.gates.len(), 2);
/// ```
pub fn circuit_from_qasm(input: &str) -> Result<QasmImportResult, QasmError> {
    let cleaned = strip_comments(input);
    let raw_stmts: Vec<&str> = cleaned
        .split(';')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    let mut stmts = Vec::new();
    let mut warnings = Vec::new();

    for raw in &raw_stmts {
        match try_parse_statement(raw) {
            Ok(stmt) => stmts.push(stmt),
            Err(msg) => warnings.push(msg),
        }
    }

    // Collect quantum register declarations (name, base_index, size)
    let mut qregs: Vec<(&str, usize, usize)> = Vec::new();
    let mut total_qubits = 0;

    for stmt in &stmts {
        if let QasmStmt::Qreg(name, size) = stmt {
            qregs.push((name, total_qubits, *size));
            total_qubits += size;
        }
    }

    if total_qubits == 0 {
        return Err(QasmError::EmptyCircuit);
    }

    let resolve_qubit = |reg: &str, idx: usize| -> Result<usize, QasmError> {
        for &(name, base, size) in &qregs {
            if name == reg {
                if idx >= size {
                    return Err(QasmError::ParseError(format!(
                        "Index {} out of bounds for register '{}' (size {})",
                        idx, reg, size
                    )));
                }
                return Ok(base + idx);
            }
        }
        Err(QasmError::ParseError(format!(
            "Undefined quantum register '{}'",
            reg
        )))
    };

    let mut circuit = Circuit::new(total_qubits);
    // ASAP column scheduling: track next free column per qubit
    let mut qubit_next_col = vec![0usize; total_qubits];

    for stmt in &stmts {
        match stmt {
            QasmStmt::Header(_)
            | QasmStmt::Include(_)
            | QasmStmt::Qreg(_, _)
            | QasmStmt::Creg(_, _) => {}

            QasmStmt::Gate {
                name,
                params,
                qubits,
            } => {
                let resolved: Result<Vec<usize>, _> = qubits
                    .iter()
                    .map(|(reg, idx)| resolve_qubit(reg, *idx))
                    .collect();
                let resolved = resolved?;

                match map_gate_to_circuit(name, params, &resolved) {
                    Ok((gate_type, targets, controls)) => {
                        let involved: Vec<usize> =
                            targets.iter().chain(controls.iter()).copied().collect();
                        let col = involved
                            .iter()
                            .map(|&q| qubit_next_col[q])
                            .max()
                            .unwrap_or(0);

                        circuit
                            .add_gate(gate_type, targets, controls, col)
                            .map_err(|e| QasmError::ParseError(format!("{}", e)))?;

                        for &q in &involved {
                            qubit_next_col[q] = col + 1;
                        }
                    }
                    Err(warning) => {
                        warnings.push(warning);
                    }
                }
            }

            QasmStmt::Measure { qubit, cbit: _ } => {
                let qubit_idx = resolve_qubit(qubit.0, qubit.1)?;
                let col = qubit_next_col[qubit_idx];
                circuit
                    .add_measurement(qubit_idx, col)
                    .map_err(|e| QasmError::ParseError(format!("{}", e)))?;
                qubit_next_col[qubit_idx] = col + 1;
            }

            QasmStmt::Barrier(qubits) => {
                for qbit in qubits {
                    let qubit_idx = resolve_qubit(qbit.0, qbit.1)?;
                    let col = qubit_next_col[qubit_idx];
                    circuit
                        .add_gate(GateType::Barrier, vec![qubit_idx], vec![], col)
                        .map_err(|e| QasmError::ParseError(format!("{}", e)))?;
                    qubit_next_col[qubit_idx] = col + 1;
                }
            }
        }
    }

    Ok(QasmImportResult { circuit, warnings })
}

impl Circuit {
    /// Export this circuit to OpenQASM 2.0 code.
    ///
    /// # Returns
    /// * `Ok(String)` - The generated OpenQASM 2.0 code.
    /// * `Err(QasmError)` - If the circuit cannot be exported.
    ///
    /// # Example
    /// ```no_run
    /// use ketgrid_core::Circuit;
    ///
    /// let circuit = Circuit::new(2);
    /// let qasm_code = circuit.to_qasm().unwrap();
    /// println!("{}", qasm_code);
    /// ```
    pub fn to_qasm(&self) -> Result<String, QasmError> {
        circuit_to_qasm(self)
    }

    /// Parse an OpenQASM 2.0 string into a new Circuit.
    ///
    /// # Returns
    /// * `Ok(QasmImportResult)` - The parsed circuit and any warnings.
    /// * `Err(QasmError)` - If the input cannot be parsed.
    pub fn from_qasm(input: &str) -> Result<QasmImportResult, QasmError> {
        circuit_from_qasm(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gate::GateType;

    #[test]
    fn test_empty_circuit_error() {
        let circuit = Circuit::new(0);
        let result = circuit_to_qasm(&circuit);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), QasmError::EmptyCircuit);
    }

    #[test]
    fn test_header() {
        let circuit = Circuit::new(1);
        let code = circuit_to_qasm(&circuit).unwrap();
        assert!(code.contains("OPENQASM 2.0;"));
        assert!(code.contains("include \"qelib1.inc\";"));
    }

    #[test]
    fn test_qreg_declaration() {
        let circuit = Circuit::new(3);
        let code = circuit_to_qasm(&circuit).unwrap();
        assert!(code.contains("qreg q[3];"));
    }

    #[test]
    fn test_creg_declaration() {
        let mut circuit = Circuit::new(2);
        circuit.add_measurement(0, 0).unwrap();
        circuit.add_measurement(1, 1).unwrap();

        let code = circuit_to_qasm(&circuit).unwrap();
        assert!(code.contains("creg c[2];"));
    }

    #[test]
    fn test_single_qubit_gates() {
        let mut circuit = Circuit::new(1);
        circuit.add_gate(GateType::H, vec![0], vec![], 0).unwrap();
        circuit.add_gate(GateType::X, vec![0], vec![], 1).unwrap();
        circuit.add_gate(GateType::Y, vec![0], vec![], 2).unwrap();
        circuit.add_gate(GateType::Z, vec![0], vec![], 3).unwrap();
        circuit.add_gate(GateType::S, vec![0], vec![], 4).unwrap();
        circuit.add_gate(GateType::T, vec![0], vec![], 5).unwrap();

        let code = circuit_to_qasm(&circuit).unwrap();
        assert!(code.contains("h q[0];"));
        assert!(code.contains("x q[0];"));
        assert!(code.contains("y q[0];"));
        assert!(code.contains("z q[0];"));
        assert!(code.contains("s q[0];"));
        assert!(code.contains("t q[0];"));
    }

    #[test]
    fn test_parameterized_gates() {
        let mut circuit = Circuit::new(2);
        circuit
            .add_gate(GateType::Rx(std::f64::consts::PI), vec![0], vec![], 0)
            .unwrap();
        circuit
            .add_gate(GateType::Ry(std::f64::consts::FRAC_PI_2), vec![1], vec![], 1)
            .unwrap();
        circuit.add_gate(GateType::Rz(1.0), vec![0], vec![], 2).unwrap();

        let code = circuit_to_qasm(&circuit).unwrap();
        assert!(code.contains("rx(3.141593) q[0];"));
        assert!(code.contains("ry(1.570796) q[1];"));
        assert!(code.contains("rz(1.000000) q[0];"));
    }

    #[test]
    fn test_controlled_gates() {
        let mut circuit = Circuit::new(2);
        circuit.add_gate(GateType::Cnot, vec![1], vec![0], 0).unwrap();

        let code = circuit_to_qasm(&circuit).unwrap();
        assert!(code.contains("cx q[0],q[1];"));
    }

    #[test]
    fn test_cz_gate() {
        let mut circuit = Circuit::new(2);
        circuit.add_gate(GateType::Cz, vec![1], vec![0], 0).unwrap();

        let code = circuit_to_qasm(&circuit).unwrap();
        assert!(code.contains("cz q[0],q[1];"));
    }

    #[test]
    fn test_swap_gate() {
        let mut circuit = Circuit::new(2);
        circuit.add_gate(GateType::Swap, vec![0, 1], vec![], 0).unwrap();

        let code = circuit_to_qasm(&circuit).unwrap();
        assert!(code.contains("swap q[0],q[1];"));
    }

    #[test]
    fn test_toffoli_gate() {
        let mut circuit = Circuit::new(3);
        circuit.add_gate(GateType::Toffoli, vec![2], vec![0, 1], 0).unwrap();

        let code = circuit_to_qasm(&circuit).unwrap();
        assert!(code.contains("ccx q[0],q[1],q[2];"));
    }

    #[test]
    fn test_barrier_gate() {
        let mut circuit = Circuit::new(2);
        circuit.add_gate(GateType::Barrier, vec![0], vec![], 0).unwrap();
        circuit.add_gate(GateType::Barrier, vec![1], vec![], 0).unwrap();

        let code = circuit_to_qasm(&circuit).unwrap();
        assert!(code.contains("barrier q[0];"));
        assert!(code.contains("barrier q[1];"));
    }

    #[test]
    fn test_barrier_gate_multi_qubit() {
        let mut circuit = Circuit::new(2);
        // Barrier is a single-qubit gate that needs to be added twice for multi-qubit barrier
        circuit.add_gate(GateType::Barrier, vec![0], vec![], 0).unwrap();
        circuit.add_gate(GateType::Barrier, vec![1], vec![], 0).unwrap();

        let code = circuit_to_qasm(&circuit).unwrap();
        // Multi-qubit barrier should be combined as: barrier q[0],q[1];
        assert!(code.contains("barrier q[0];"));
        assert!(code.contains("barrier q[1];"));
    }

    #[test]
    fn test_barrier_gate_single_qubit() {
        let mut circuit = Circuit::new(1);
        circuit.add_gate(GateType::H, vec![0], vec![], 0).unwrap();
        circuit.add_gate(GateType::Barrier, vec![0], vec![], 1).unwrap();
        circuit.add_gate(GateType::X, vec![0], vec![], 2).unwrap();

        let code = circuit_to_qasm(&circuit).unwrap();
        // Barrier should appear between H and X
        let h_pos = code.find("h q[0];").unwrap();
        let barrier_pos = code.find("barrier q[0];").unwrap();
        let x_pos = code.find("x q[0];").unwrap();
        assert!(h_pos < barrier_pos, "H should come before barrier");
        assert!(barrier_pos < x_pos, "barrier should come before X");
    }

    #[test]
    fn test_identity_gate_skipped() {
        let mut circuit = Circuit::new(1);
        circuit.add_gate(GateType::H, vec![0], vec![], 0).unwrap();
        circuit.add_gate(GateType::Identity, vec![0], vec![], 1).unwrap();
        circuit.add_gate(GateType::X, vec![0], vec![], 2).unwrap();

        let code = circuit_to_qasm(&circuit).unwrap();
        // Identity should be skipped (no line generated)
        assert!(code.contains("h q[0];"));
        assert!(code.contains("x q[0];"));
        // The code should not contain any reference to Identity
        assert!(!code.contains("Identity"));
    }

    #[test]
    fn test_custom_gate_comment() {
        let mut circuit = Circuit::new(1);
        circuit
            .add_gate(GateType::Custom("U3Gate".to_string()), vec![0], vec![], 0)
            .unwrap();

        let code = circuit_to_qasm(&circuit).unwrap();
        assert!(code.contains("// Custom gate: U3Gate"));
    }

    #[test]
    fn test_measurements() {
        let mut circuit = Circuit::new(2);
        circuit.add_gate(GateType::H, vec![0], vec![], 0).unwrap();
        circuit.add_measurement(0, 1).unwrap();
        circuit.add_measurement(1, 1).unwrap();

        let code = circuit_to_qasm(&circuit).unwrap();
        // Should have classical register
        assert!(code.contains("creg c[2];"));
        // Should have measurement lines
        assert!(code.contains("measure q[0] -> c[0];"));
        assert!(code.contains("measure q[1] -> c[1];"));
    }

    #[test]
    fn test_bell_state_circuit() {
        let mut circuit = Circuit::new(2);
        circuit.add_gate(GateType::H, vec![0], vec![], 0).unwrap();
        circuit.add_gate(GateType::Cnot, vec![1], vec![0], 1).unwrap();
        circuit.add_measurement(0, 2).unwrap();
        circuit.add_measurement(1, 2).unwrap();

        let code = circuit_to_qasm(&circuit).unwrap();

        // Check header
        assert!(code.contains("OPENQASM 2.0;"));
        assert!(code.contains("include \"qelib1.inc\";"));

        // Check registers
        assert!(code.contains("qreg q[2];"));
        assert!(code.contains("creg c[2];"));

        // Check gate order (should be sorted by column)
        let h_pos = code.find("h q[0];").unwrap();
        let cx_pos = code.find("cx q[0],q[1];").unwrap();
        assert!(h_pos < cx_pos, "H gate should come before CNOT");

        // Check measurements
        assert!(code.contains("measure q[0] -> c[0];"));
        assert!(code.contains("measure q[1] -> c[1];"));
    }

    #[test]
    fn test_circuit_to_qasm_method() {
        let mut circuit = Circuit::new(1);
        circuit.add_gate(GateType::H, vec![0], vec![], 0).unwrap();

        let code = circuit.to_qasm().unwrap();
        assert!(code.contains("h q[0];"));
    }

    #[test]
    fn test_execution_order_by_column() {
        // Gates added out of order should be sorted by column in output
        let mut circuit = Circuit::new(2);
        circuit.add_gate(GateType::X, vec![1], vec![], 2).unwrap();
        circuit.add_gate(GateType::H, vec![0], vec![], 0).unwrap();
        circuit.add_gate(GateType::Z, vec![0], vec![], 1).unwrap();

        let code = circuit_to_qasm(&circuit).unwrap();

        // Find positions
        let h_pos = code.find("h q[0];").unwrap();
        let z_pos = code.find("z q[0];").unwrap();
        let x_pos = code.find("x q[1];").unwrap();

        // Should be in column order: H (col 0), Z (col 1), X (col 2)
        assert!(h_pos < z_pos, "H should come before Z");
        assert!(z_pos < x_pos, "Z should come before X");
    }

    #[test]
    fn test_no_trailing_newline() {
        let circuit = Circuit::new(1);
        let code = circuit_to_qasm(&circuit).unwrap();
        // Last character should not be newline
        assert!(!code.ends_with('\n'));
    }

    // ===== Import Tests =====

    #[test]
    fn test_import_bell_state() {
        let qasm = "\
OPENQASM 2.0;
include \"qelib1.inc\";
qreg q[2];
creg c[2];
h q[0];
cx q[0],q[1];
measure q[0] -> c[0];
measure q[1] -> c[1];";

        let result = circuit_from_qasm(qasm).unwrap();
        assert_eq!(result.circuit.num_qubits(), 2);
        assert_eq!(result.circuit.gates.len(), 2);
        assert_eq!(result.circuit.measurements.len(), 2);
        assert!(result.warnings.is_empty());

        // H gate at col 0
        assert_eq!(result.circuit.gates[0].gate, GateType::H);
        assert_eq!(result.circuit.gates[0].target_qubits, vec![0]);
        assert_eq!(result.circuit.gates[0].column, 0);

        // CNOT at col 1 (both qubits busy after col 0)
        assert_eq!(result.circuit.gates[1].gate, GateType::Cnot);
        assert_eq!(result.circuit.gates[1].control_qubits, vec![0]);
        assert_eq!(result.circuit.gates[1].target_qubits, vec![1]);
        assert_eq!(result.circuit.gates[1].column, 1);

        // Measurements at col 2
        assert_eq!(result.circuit.measurements[0].qubit_id, 0);
        assert_eq!(result.circuit.measurements[0].column, 2);
        assert_eq!(result.circuit.measurements[1].qubit_id, 1);
        assert_eq!(result.circuit.measurements[1].column, 2);
    }

    #[test]
    fn test_import_single_qubit_gates() {
        let qasm = "\
OPENQASM 2.0;
qreg q[1];
h q[0];
x q[0];
y q[0];
z q[0];
s q[0];
t q[0];";

        let result = circuit_from_qasm(qasm).unwrap();
        assert_eq!(result.circuit.gates.len(), 6);
        assert_eq!(result.circuit.gates[0].gate, GateType::H);
        assert_eq!(result.circuit.gates[1].gate, GateType::X);
        assert_eq!(result.circuit.gates[2].gate, GateType::Y);
        assert_eq!(result.circuit.gates[3].gate, GateType::Z);
        assert_eq!(result.circuit.gates[4].gate, GateType::S);
        assert_eq!(result.circuit.gates[5].gate, GateType::T);

        // Sequential on same qubit → columns 0..5
        for (i, gate) in result.circuit.gates.iter().enumerate() {
            assert_eq!(gate.column, i);
        }
    }

    #[test]
    fn test_import_parameterized_gates() {
        let qasm = "\
OPENQASM 2.0;
qreg q[1];
rx(pi) q[0];
ry(pi/2) q[0];
rz(2*pi/3) q[0];";

        let result = circuit_from_qasm(qasm).unwrap();
        assert_eq!(result.circuit.gates.len(), 3);

        let eps = 1e-10;
        match result.circuit.gates[0].gate {
            GateType::Rx(theta) => assert!((theta - std::f64::consts::PI).abs() < eps),
            _ => panic!("Expected Rx"),
        }
        match result.circuit.gates[1].gate {
            GateType::Ry(theta) => assert!((theta - std::f64::consts::FRAC_PI_2).abs() < eps),
            _ => panic!("Expected Ry"),
        }
        match result.circuit.gates[2].gate {
            GateType::Rz(theta) => {
                assert!((theta - 2.0 * std::f64::consts::PI / 3.0).abs() < eps);
            }
            _ => panic!("Expected Rz"),
        }
    }

    #[test]
    fn test_import_negative_pi_expression() {
        let qasm = "\
OPENQASM 2.0;
qreg q[1];
rx(-pi/4) q[0];";

        let result = circuit_from_qasm(qasm).unwrap();
        let eps = 1e-10;
        match result.circuit.gates[0].gate {
            GateType::Rx(theta) => {
                assert!((theta - (-std::f64::consts::FRAC_PI_4)).abs() < eps);
            }
            _ => panic!("Expected Rx"),
        }
    }

    #[test]
    fn test_import_multi_qubit_gates() {
        let qasm = "\
OPENQASM 2.0;
qreg q[3];
cx q[0],q[1];
cz q[0],q[2];
swap q[1],q[2];
ccx q[0],q[1],q[2];";

        let result = circuit_from_qasm(qasm).unwrap();
        assert_eq!(result.circuit.gates.len(), 4);

        assert_eq!(result.circuit.gates[0].gate, GateType::Cnot);
        assert_eq!(result.circuit.gates[0].control_qubits, vec![0]);
        assert_eq!(result.circuit.gates[0].target_qubits, vec![1]);

        assert_eq!(result.circuit.gates[1].gate, GateType::Cz);
        assert_eq!(result.circuit.gates[2].gate, GateType::Swap);
        assert_eq!(result.circuit.gates[3].gate, GateType::Toffoli);
        assert_eq!(result.circuit.gates[3].control_qubits, vec![0, 1]);
        assert_eq!(result.circuit.gates[3].target_qubits, vec![2]);
    }

    #[test]
    fn test_import_barriers() {
        let qasm = "\
OPENQASM 2.0;
qreg q[2];
h q[0];
barrier q[0],q[1];
x q[0];";

        let result = circuit_from_qasm(qasm).unwrap();
        // H + 2 barrier (one per qubit) + X = 4 gates
        assert_eq!(result.circuit.gates.len(), 4);
        assert_eq!(result.circuit.gates[0].gate, GateType::H);
        assert_eq!(result.circuit.gates[1].gate, GateType::Barrier);
        assert_eq!(result.circuit.gates[2].gate, GateType::Barrier);
        assert_eq!(result.circuit.gates[3].gate, GateType::X);
    }

    #[test]
    fn test_import_unsupported_gate_warning() {
        let qasm = "\
OPENQASM 2.0;
qreg q[2];
h q[0];
sdg q[0];
x q[1];";

        let result = circuit_from_qasm(qasm).unwrap();
        assert_eq!(result.circuit.gates.len(), 2); // h and x only
        assert_eq!(result.warnings.len(), 1);
        assert!(result.warnings[0].contains("sdg"));
    }

    #[test]
    fn test_import_empty_circuit_error() {
        let qasm = "OPENQASM 2.0;\ninclude \"qelib1.inc\";";
        let result = circuit_from_qasm(qasm);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), QasmError::EmptyCircuit);
    }

    #[test]
    fn test_import_undefined_register_error() {
        let qasm = "\
OPENQASM 2.0;
qreg q[2];
h r[0];";

        let result = circuit_from_qasm(qasm);
        assert!(result.is_err());
        match result.unwrap_err() {
            QasmError::ParseError(msg) => assert!(msg.contains("Undefined")),
            other => panic!("Expected ParseError, got {:?}", other),
        }
    }

    #[test]
    fn test_import_qubit_index_out_of_bounds() {
        let qasm = "\
OPENQASM 2.0;
qreg q[2];
h q[5];";

        let result = circuit_from_qasm(qasm);
        assert!(result.is_err());
        match result.unwrap_err() {
            QasmError::ParseError(msg) => assert!(msg.contains("out of bounds")),
            other => panic!("Expected ParseError, got {:?}", other),
        }
    }

    #[test]
    fn test_import_comments_stripped() {
        let qasm = "\
OPENQASM 2.0;
// This is a comment
qreg q[2];
h q[0]; // Apply Hadamard
cx q[0],q[1];";

        let result = circuit_from_qasm(qasm).unwrap();
        assert_eq!(result.circuit.num_qubits(), 2);
        assert_eq!(result.circuit.gates.len(), 2);
    }

    #[test]
    fn test_import_parallel_column_scheduling() {
        let qasm = "\
OPENQASM 2.0;
qreg q[3];
h q[0];
x q[1];
y q[2];
cx q[0],q[1];";

        let result = circuit_from_qasm(qasm).unwrap();
        // h, x, y are on independent qubits → all at col 0
        assert_eq!(result.circuit.gates[0].column, 0); // h q[0]
        assert_eq!(result.circuit.gates[1].column, 0); // x q[1]
        assert_eq!(result.circuit.gates[2].column, 0); // y q[2]
        // cx needs q[0] (next=1) and q[1] (next=1) → col 1
        assert_eq!(result.circuit.gates[3].column, 1);
    }

    #[test]
    fn test_import_multiple_registers() {
        let qasm = "\
OPENQASM 2.0;
qreg a[2];
qreg b[1];
h a[0];
x b[0];
cx a[1],b[0];";

        let result = circuit_from_qasm(qasm).unwrap();
        assert_eq!(result.circuit.num_qubits(), 3);
        // a[0]=0, a[1]=1, b[0]=2
        assert_eq!(result.circuit.gates[0].target_qubits, vec![0]); // h a[0]
        assert_eq!(result.circuit.gates[1].target_qubits, vec![2]); // x b[0]
        assert_eq!(result.circuit.gates[2].control_qubits, vec![1]); // cx a[1],b[0]
        assert_eq!(result.circuit.gates[2].target_qubits, vec![2]);
    }

    #[test]
    fn test_import_roundtrip_bell_state() {
        // Build a Bell state, export to QASM, re-import, verify equivalence
        let mut original = Circuit::new(2);
        original.add_gate(GateType::H, vec![0], vec![], 0).unwrap();
        original
            .add_gate(GateType::Cnot, vec![1], vec![0], 1)
            .unwrap();
        original.add_measurement(0, 2).unwrap();
        original.add_measurement(1, 2).unwrap();

        let qasm_code = circuit_to_qasm(&original).unwrap();
        let imported = circuit_from_qasm(&qasm_code).unwrap();

        assert_eq!(imported.circuit.num_qubits(), original.num_qubits());
        assert_eq!(imported.circuit.gates.len(), original.gates.len());
        assert_eq!(
            imported.circuit.measurements.len(),
            original.measurements.len()
        );

        // Gate types and qubit assignments should match
        for (orig, imp) in original.gates.iter().zip(imported.circuit.gates.iter()) {
            assert_eq!(orig.gate, imp.gate);
            assert_eq!(orig.target_qubits, imp.target_qubits);
            assert_eq!(orig.control_qubits, imp.control_qubits);
            assert_eq!(orig.column, imp.column);
        }
    }

    #[test]
    fn test_import_from_qasm_method() {
        let qasm = "OPENQASM 2.0;\nqreg q[1];\nh q[0];";
        let result = Circuit::from_qasm(qasm).unwrap();
        assert_eq!(result.circuit.gates.len(), 1);
        assert_eq!(result.circuit.gates[0].gate, GateType::H);
    }
}
