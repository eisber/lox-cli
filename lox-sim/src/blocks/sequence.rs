//! SequenceController — text-based programming block that executes
//! line-by-line sequences with sleep, set, goto, if/endif, and more.

use crate::blocks::{bool_signal, deserialize_f64s, is_high, serialize_f64s, Block};
use crate::types::Signal;

// ============================================================================
// Program AST
// ============================================================================

/// Reference to a variable (AI input, AQ output, or value1-value5).
#[derive(Debug, Clone, Copy, PartialEq)]
enum VarRef {
    AI(usize),    // 0-based index into AI1..AI8
    AQ(usize),    // 0-based index into AQ1..AQ8
    Value(usize), // 0-based index into value1..value5
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum MathOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone, PartialEq)]
enum Expr {
    Lit(f64),
    Var(VarRef),
    BinOp(Box<Expr>, MathOp, Box<Expr>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum CmpOp {
    Eq,
    Ne,
    Gt,
    Ge,
    Lt,
    Le,
}

#[derive(Debug, Clone, PartialEq)]
enum SeqLine {
    Sleep(f64),                                       // seconds
    Set { target: VarRef, expr: Expr },               // set AQ1 = expr
    SetPulse { target: VarRef, value: Option<Expr> }, // setpulse AQ1 [= expr]
    WaitCondition { left: Expr, op: CmpOp, right: Expr },
    Goto(usize), // 1-based line number
    If { left: Expr, op: CmpOp, right: Expr },
    EndIf,
    StartSequence(usize), // 1-based sequence number
    Return,
    Comment, // blank or comment line — skip
}

// ============================================================================
// Parser
// ============================================================================

/// Parse a multi-line program text. Each sequence is separated by
/// `sequence N` headers. If no header, everything is sequence 1.
fn parse_programs(text: &str) -> Vec<Vec<SeqLine>> {
    let mut programs: Vec<Vec<SeqLine>> = vec![Vec::new(); 8];
    let mut current_seq: usize = 0; // 0-based

    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with("//") || line.starts_with('#') {
            // If we're in a sequence, add a Comment line to preserve line numbering
            if !programs[current_seq].is_empty() || current_seq > 0 {
                programs[current_seq].push(SeqLine::Comment);
            }
            continue;
        }

        let lower = line.to_lowercase();

        // Sequence header: "sequence 1" .. "sequence 8"
        if let Some(rest) = lower.strip_prefix("sequence") {
            if let Ok(n) = rest.trim().parse::<usize>() {
                if (1..=8).contains(&n) {
                    current_seq = n - 1;
                }
            }
            continue;
        }

        programs[current_seq].push(parse_line(&lower, line));
    }

    programs
}

fn parse_line(lower: &str, _original: &str) -> SeqLine {
    // sleep N s | sleep N m
    if let Some(rest) = lower.strip_prefix("sleep") {
        return parse_sleep(rest.trim());
    }

    // setpulse (must check before "set")
    if let Some(rest) = lower.strip_prefix("setpulse") {
        return parse_setpulse(rest.trim());
    }

    // set AQ1 = expr
    if let Some(rest) = lower.strip_prefix("set") {
        return parse_set(rest.trim());
    }

    // waitcondition expr op expr
    if let Some(rest) = lower.strip_prefix("waitcondition") {
        return parse_waitcondition(rest.trim());
    }

    // goto N
    if let Some(rest) = lower.strip_prefix("goto") {
        if let Ok(n) = rest.trim().parse::<usize>() {
            return SeqLine::Goto(n);
        }
        return SeqLine::Comment;
    }

    // if expr op expr
    if let Some(rest) = lower.strip_prefix("if ") {
        return parse_if(rest.trim());
    }

    // endif
    if lower.starts_with("endif") {
        return SeqLine::EndIf;
    }

    // startsequence N
    if let Some(rest) = lower.strip_prefix("startsequence") {
        if let Ok(n) = rest.trim().parse::<usize>() {
            return SeqLine::StartSequence(n);
        }
        return SeqLine::Comment;
    }

    // return
    if lower == "return" {
        return SeqLine::Return;
    }

    SeqLine::Comment
}

fn parse_sleep(s: &str) -> SeqLine {
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() >= 2 {
        if let Ok(n) = parts[0].parse::<f64>() {
            let secs = if parts[1].starts_with('m') {
                n * 60.0
            } else {
                n
            };
            return SeqLine::Sleep(secs);
        }
    }
    // Fallback: just a number (seconds)
    if let Ok(n) = s.trim_end_matches('s').trim().parse::<f64>() {
        return SeqLine::Sleep(n);
    }
    SeqLine::Comment
}

fn parse_setpulse(s: &str) -> SeqLine {
    // "aq1 = 4" or just "aq1"
    if let Some(eq_pos) = s.find('=') {
        let target_str = s[..eq_pos].trim();
        let expr_str = s[eq_pos + 1..].trim();
        if let Some(target) = parse_varref(target_str) {
            if let Some(expr) = parse_expr(expr_str) {
                return SeqLine::SetPulse {
                    target,
                    value: Some(expr),
                };
            }
        }
    } else if let Some(target) = parse_varref(s.trim()) {
        return SeqLine::SetPulse {
            target,
            value: None,
        };
    }
    SeqLine::Comment
}

fn parse_set(s: &str) -> SeqLine {
    // "aq1 = ai2 + 3"
    if let Some(eq_pos) = s.find('=') {
        let target_str = s[..eq_pos].trim();
        let expr_str = s[eq_pos + 1..].trim();
        if let Some(target) = parse_varref(target_str) {
            if let Some(expr) = parse_expr(expr_str) {
                return SeqLine::Set { target, expr };
            }
        }
    }
    SeqLine::Comment
}

fn parse_waitcondition(s: &str) -> SeqLine {
    if let Some((left, op, right)) = parse_comparison(s) {
        return SeqLine::WaitCondition { left, op, right };
    }
    SeqLine::Comment
}

fn parse_if(s: &str) -> SeqLine {
    if let Some((left, op, right)) = parse_comparison(s) {
        return SeqLine::If { left, op, right };
    }
    SeqLine::Comment
}

fn parse_comparison(s: &str) -> Option<(Expr, CmpOp, Expr)> {
    // Find comparison operator (try multi-char first)
    let ops = [
        (">=", CmpOp::Ge),
        ("<=", CmpOp::Le),
        ("!=", CmpOp::Ne),
        (">", CmpOp::Gt),
        ("<", CmpOp::Lt),
        ("=", CmpOp::Eq),
    ];

    for (op_str, op) in &ops {
        // For single-char "=", avoid matching inside ">=" or "<=" or "!="
        if *op_str == "=" {
            // Find "=" that is not preceded by ">", "<", or "!"
            if let Some(pos) = find_standalone_eq(s) {
                let left_str = s[..pos].trim();
                let right_str = s[pos + 1..].trim();
                let left = parse_expr(left_str)?;
                let right = parse_expr(right_str)?;
                return Some((left, *op, right));
            }
        } else if let Some(pos) = s.find(op_str) {
            let left_str = s[..pos].trim();
            let right_str = s[pos + op_str.len()..].trim();
            let left = parse_expr(left_str)?;
            let right = parse_expr(right_str)?;
            return Some((left, *op, right));
        }
    }
    None
}

/// Find a standalone "=" that isn't part of ">=", "<=", or "!="
fn find_standalone_eq(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        if b == b'=' {
            // Check it's not preceded by '>', '<', '!' or followed by another operator char
            let prev_ok = i == 0 || !matches!(bytes[i - 1], b'>' | b'<' | b'!');
            let next_ok = i + 1 >= bytes.len() || bytes[i + 1] != b'=';
            if prev_ok && next_ok {
                return Some(i);
            }
        }
    }
    None
}

fn parse_expr(s: &str) -> Option<Expr> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    // Try to split on + or - (lowest precedence), scanning right-to-left
    // to get left-associativity
    if let Some(result) = try_split_binop(s, &[('+', MathOp::Add), ('-', MathOp::Sub)]) {
        return Some(result);
    }

    // Try * or /
    if let Some(result) = try_split_binop(s, &[('*', MathOp::Mul), ('/', MathOp::Div)]) {
        return Some(result);
    }

    // Atom: variable or literal
    if let Some(vr) = parse_varref(s) {
        return Some(Expr::Var(vr));
    }
    if let Ok(n) = s.parse::<f64>() {
        return Some(Expr::Lit(n));
    }

    None
}

/// Try splitting on any of the given operators, scanning right-to-left.
fn try_split_binop(s: &str, ops: &[(char, MathOp)]) -> Option<Expr> {
    let bytes = s.as_bytes();
    // Scan right-to-left for correct left-associativity
    for i in (0..bytes.len()).rev() {
        for (op_char, math_op) in ops {
            if bytes[i] == *op_char as u8 {
                // Don't split at position 0 (would make empty left side)
                if i == 0 {
                    continue;
                }
                let left_str = s[..i].trim();
                let right_str = s[i + 1..].trim();
                if left_str.is_empty() || right_str.is_empty() {
                    continue;
                }
                let left = parse_expr(left_str)?;
                let right = parse_expr(right_str)?;
                return Some(Expr::BinOp(Box::new(left), *math_op, Box::new(right)));
            }
        }
    }
    None
}

fn parse_varref(s: &str) -> Option<VarRef> {
    let s = s.trim().to_lowercase();
    // ai1..ai8
    if let Some(rest) = s.strip_prefix("ai") {
        if let Ok(n) = rest.parse::<usize>() {
            if (1..=8).contains(&n) {
                return Some(VarRef::AI(n - 1));
            }
        }
    }
    // aq1..aq8
    if let Some(rest) = s.strip_prefix("aq") {
        if let Ok(n) = rest.parse::<usize>() {
            if (1..=8).contains(&n) {
                return Some(VarRef::AQ(n - 1));
            }
        }
    }
    // value1..value5
    if let Some(rest) = s.strip_prefix("value") {
        if let Ok(n) = rest.parse::<usize>() {
            if (1..=5).contains(&n) {
                return Some(VarRef::Value(n - 1));
            }
        }
    }
    None
}

// ============================================================================
// Block implementation
// ============================================================================

/// SequenceController — programmable text-based sequence block.
///
/// Inputs (20):  S1-S8 (Trigger1..Trigger8), AI1-AI8, S (ATrigger), Off
/// Outputs (11): AQ1-AQ8, current sequence, current line, TQ (always 0)
#[derive(Clone)]
pub struct SequenceController {
    programs: Vec<Vec<SeqLine>>,
    current_seq: usize,  // 0 = idle, 1-8 = running
    current_line: usize, // 0-based index into current program
    sleep_remaining: f64,
    waiting: bool,
    variables: [f64; 5],             // value1..value5
    outputs: [f64; 8],               // AQ1..AQ8 persistent state
    call_stack: Vec<(usize, usize)>, // (seq 1-based, line 0-based)
    pulse_active: [bool; 8],         // tracks which outputs are in pulse mode
    interval: f64,                   // execution interval in seconds
    time_since_step: f64,
    prev_triggers: [f64; 8], // for rising edge detection on S1-S8
    prev_off: f64,
    skipping_if: usize, // depth of false-if blocks being skipped
}

impl SequenceController {
    pub fn new(program_text: &str, interval_ms: f64) -> Self {
        let interval = if interval_ms > 0.0 {
            interval_ms / 1000.0
        } else {
            0.5
        };
        Self {
            programs: parse_programs(program_text),
            current_seq: 0,
            current_line: 0,
            sleep_remaining: 0.0,
            waiting: false,
            variables: [0.0; 5],
            outputs: [0.0; 8],
            call_stack: Vec::new(),
            pulse_active: [false; 8],
            interval,
            time_since_step: 0.0,
            prev_triggers: [0.0; 8],
            prev_off: 0.0,
            skipping_if: 0,
        }
    }

    fn start_sequence(&mut self, seq: usize) {
        if (1..=8).contains(&seq) && !self.programs[seq - 1].is_empty() {
            self.current_seq = seq;
            self.current_line = 0;
            self.sleep_remaining = 0.0;
            self.waiting = false;
            self.skipping_if = 0;
        }
    }

    fn reset(&mut self) {
        self.current_seq = 0;
        self.current_line = 0;
        self.sleep_remaining = 0.0;
        self.waiting = false;
        self.outputs = [0.0; 8];
        self.variables = [0.0; 5];
        self.call_stack.clear();
        self.pulse_active = [false; 8];
        self.skipping_if = 0;
        self.time_since_step = 0.0;
    }

    fn eval_expr(&self, expr: &Expr, ai: &[f64; 8]) -> f64 {
        match expr {
            Expr::Lit(v) => *v,
            Expr::Var(vr) => self.read_var(vr, ai),
            Expr::BinOp(l, op, r) => {
                let lv = self.eval_expr(l, ai);
                let rv = self.eval_expr(r, ai);
                match op {
                    MathOp::Add => lv + rv,
                    MathOp::Sub => lv - rv,
                    MathOp::Mul => lv * rv,
                    MathOp::Div => {
                        if rv.abs() < f64::EPSILON {
                            0.0
                        } else {
                            lv / rv
                        }
                    }
                }
            }
        }
    }

    fn read_var(&self, vr: &VarRef, ai: &[f64; 8]) -> f64 {
        match vr {
            VarRef::AI(i) => ai[*i],
            VarRef::AQ(i) => self.outputs[*i],
            VarRef::Value(i) => self.variables[*i],
        }
    }

    fn write_var(&mut self, vr: &VarRef, val: f64) {
        match vr {
            VarRef::AQ(i) => self.outputs[*i] = val,
            VarRef::Value(i) => self.variables[*i] = val,
            VarRef::AI(_) => {} // AI inputs are read-only
        }
    }

    fn eval_cmp(&self, left: &Expr, op: &CmpOp, right: &Expr, ai: &[f64; 8]) -> bool {
        let lv = self.eval_expr(left, ai);
        let rv = self.eval_expr(right, ai);
        match op {
            CmpOp::Eq => (lv - rv).abs() < f64::EPSILON,
            CmpOp::Ne => (lv - rv).abs() >= f64::EPSILON,
            CmpOp::Gt => lv > rv,
            CmpOp::Ge => lv >= rv,
            CmpOp::Lt => lv < rv,
            CmpOp::Le => lv <= rv,
        }
    }

    fn current_program(&self) -> Option<&Vec<SeqLine>> {
        if self.current_seq >= 1 && self.current_seq <= 8 {
            Some(&self.programs[self.current_seq - 1])
        } else {
            None
        }
    }

    fn advance_line(&mut self) {
        self.current_line += 1;
        let at_end = self
            .current_program()
            .is_none_or(|p| self.current_line >= p.len());
        if at_end {
            // End of program — check call stack
            if let Some((ret_seq, ret_line)) = self.call_stack.pop() {
                self.current_seq = ret_seq;
                self.current_line = ret_line;
            } else {
                self.current_seq = 0;
                self.current_line = 0;
            }
        }
    }

    /// Execute lines until we hit something that blocks (sleep, wait, end).
    /// Returns how many lines were executed (for loop protection).
    fn execute_step(&mut self, ai: &[f64; 8]) {
        // Limit iterations to prevent infinite loops in a single tick
        let max_iterations = 1000;
        let mut iterations = 0;

        loop {
            if iterations >= max_iterations || self.current_seq == 0 {
                break;
            }
            iterations += 1;

            let program = match self.current_program() {
                Some(p) => p.clone(), // clone to avoid borrow issues
                None => {
                    self.current_seq = 0;
                    break;
                }
            };

            if self.current_line >= program.len() {
                self.advance_line();
                continue;
            }

            let line = program[self.current_line].clone();

            // If we're skipping a false-if block
            if self.skipping_if > 0 {
                match &line {
                    SeqLine::If { .. } => {
                        self.skipping_if += 1;
                        self.current_line += 1;
                        continue;
                    }
                    SeqLine::EndIf => {
                        self.skipping_if -= 1;
                        self.current_line += 1;
                        continue;
                    }
                    _ => {
                        self.current_line += 1;
                        continue;
                    }
                }
            }

            match &line {
                SeqLine::Comment => {
                    self.current_line += 1;
                    continue;
                }

                SeqLine::Sleep(secs) => {
                    self.sleep_remaining = *secs;
                    self.current_line += 1;
                    break; // Block execution until sleep expires
                }

                SeqLine::Set { target, expr } => {
                    let val = self.eval_expr(expr, ai);
                    self.write_var(target, val);
                    self.current_line += 1;
                    continue; // Execute next line immediately
                }

                SeqLine::SetPulse { target, value } => {
                    let val = match value {
                        Some(expr) => self.eval_expr(expr, ai),
                        None => 1.0,
                    };
                    self.write_var(target, val);
                    if let VarRef::AQ(i) = target {
                        self.pulse_active[*i] = true;
                    }
                    self.current_line += 1;
                    continue;
                }

                SeqLine::WaitCondition { left, op, right } => {
                    if self.eval_cmp(left, op, right, ai) {
                        self.waiting = false;
                        self.current_line += 1;
                        continue;
                    } else {
                        self.waiting = true;
                        break; // Block until condition met
                    }
                }

                SeqLine::Goto(target_line) => {
                    // 1-based to 0-based
                    self.current_line = target_line.saturating_sub(1);
                    continue;
                }

                SeqLine::If { left, op, right } => {
                    if self.eval_cmp(left, op, right, ai) {
                        self.current_line += 1;
                        continue;
                    } else {
                        self.skipping_if = 1;
                        self.current_line += 1;
                        continue;
                    }
                }

                SeqLine::EndIf => {
                    self.current_line += 1;
                    continue;
                }

                SeqLine::StartSequence(seq) => {
                    let seq = *seq;
                    // Push return address (next line after startsequence)
                    self.call_stack
                        .push((self.current_seq, self.current_line + 1));
                    self.start_sequence(seq);
                    continue;
                }

                SeqLine::Return => {
                    if let Some((ret_seq, ret_line)) = self.call_stack.pop() {
                        self.current_seq = ret_seq;
                        self.current_line = ret_line;
                    } else {
                        self.current_seq = 0;
                        self.current_line = 0;
                    }
                    continue;
                }
            }
        }
    }
}

impl Block for SequenceController {
    fn eval(
        &mut self,
        inputs: &[Signal],
        params: &[Signal],
        dt: f64,
        prev_inputs: &[Signal],
    ) -> Vec<Signal> {
        // Input layout: Trigger1..Trigger8 (0-7), AI1..AI8 (8-15), ATrigger (16), Off (17)
        let mut triggers = [0.0f64; 8];
        for (i, t) in triggers.iter_mut().enumerate() {
            *t = inputs.get(i).copied().unwrap_or(0.0);
        }
        let mut ai = [0.0f64; 8];
        for (i, a) in ai.iter_mut().enumerate() {
            *a = inputs.get(8 + i).copied().unwrap_or(0.0);
        }
        let atrigger = inputs.get(16).copied().unwrap_or(0.0);
        let off = inputs.get(17).copied().unwrap_or(0.0);

        // Override interval from param if provided
        let param_interval = params.first().copied().unwrap_or(0.0);
        if param_interval > 0.0 {
            self.interval = param_interval / 1000.0;
        }

        // Clear pulse outputs from previous tick
        for i in 0..8 {
            if self.pulse_active[i] {
                self.outputs[i] = 0.0;
                self.pulse_active[i] = false;
            }
        }

        // Off input (dominating) — reset on rising edge
        if is_high(off) && !is_high(self.prev_off) {
            self.reset();
        }
        self.prev_off = off;

        // If Off is held high, stay locked
        if is_high(off) {
            let mut out = self.outputs.to_vec();
            out.push(0.0); // current sequence
            out.push(0.0); // current line
            out.push(0.0); // TQ
            self.prev_triggers = triggers;
            return out;
        }

        // Check triggers S1-S8 for rising edge
        for (i, &trig) in triggers.iter().enumerate() {
            if is_high(trig) && !is_high(self.prev_triggers[i]) {
                self.call_stack.clear();
                self.start_sequence(i + 1);
                self.time_since_step = self.interval; // execute immediately
                break;
            }
        }
        self.prev_triggers = triggers;

        // ATrigger — select sequence by value (non-edge, direct)
        let prev_atrigger = prev_inputs.get(16).copied().unwrap_or(0.0);
        if atrigger != prev_atrigger && atrigger > 0.5 {
            let seq = atrigger.round() as usize;
            if (1..=8).contains(&seq) {
                self.call_stack.clear();
                self.start_sequence(seq);
                self.time_since_step = self.interval;
            }
        } else if atrigger < 0.5 && prev_atrigger >= 0.5 {
            // ATrigger going to 0 stops the sequence
            self.current_seq = 0;
            self.current_line = 0;
        }

        // Time-based execution
        if self.current_seq > 0 {
            // Handle sleeping
            if self.sleep_remaining > 0.0 {
                self.sleep_remaining -= dt;
                if self.sleep_remaining <= 0.0 {
                    self.sleep_remaining = 0.0;
                    // Sleep done — execute next step
                    self.execute_step(&ai);
                }
            } else if self.waiting {
                // Re-check wait condition every tick (execute_step handles it)
                self.execute_step(&ai);
            } else {
                // Normal interval-based execution
                self.time_since_step += dt;
                if self.time_since_step >= self.interval {
                    self.time_since_step -= self.interval;
                    self.execute_step(&ai);
                }
            }
        }

        // Build outputs: AQ1..AQ8, current_seq, current_line (1-based), TQ
        let mut out = self.outputs.to_vec();
        out.push(self.current_seq as f64);
        // Line is 1-based for output (0 if idle)
        out.push(if self.current_seq > 0 {
            (self.current_line + 1) as f64
        } else {
            0.0
        });
        out.push(0.0); // TQ — not implemented

        out
    }

    fn state(&self) -> Option<Vec<u8>> {
        let mut state = Vec::new();
        state.extend_from_slice(&serialize_f64s(&self.outputs));
        state.extend_from_slice(&serialize_f64s(&self.variables));
        state.extend_from_slice(&serialize_f64s(&[
            self.current_seq as f64,
            self.current_line as f64,
            self.sleep_remaining,
            bool_signal(self.waiting),
            self.time_since_step,
            self.interval,
        ]));
        Some(state)
    }

    fn restore(&mut self, state: &[u8]) {
        // 8 outputs + 5 variables + 6 state fields = 19 f64s
        if let Some(values) = deserialize_f64s(state, 19) {
            self.outputs.copy_from_slice(&values[0..8]);
            self.variables.copy_from_slice(&values[8..13]);
            self.current_seq = values[13] as usize;
            self.current_line = values[14] as usize;
            self.sleep_remaining = values[15];
            self.waiting = values[16] > 0.5;
            self.time_since_step = values[17];
            self.interval = values[18];
        }
    }

    fn block_type(&self) -> &str {
        "SequenceController"
    }

    fn is_edge_sensitive(&self) -> bool {
        true
    }

    fn is_time_dependent(&self) -> bool {
        true
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_inputs(triggers: &[f64; 8], ai: &[f64; 8], atrigger: f64, off: f64) -> Vec<f64> {
        let mut v = Vec::with_capacity(18);
        v.extend_from_slice(triggers);
        v.extend_from_slice(ai);
        v.push(atrigger);
        v.push(off);
        v
    }

    fn trigger_seq(n: usize) -> [f64; 8] {
        let mut t = [0.0; 8];
        if n >= 1 && n <= 8 {
            t[n - 1] = 1.0;
        }
        t
    }

    fn no_triggers() -> [f64; 8] {
        [0.0; 8]
    }

    fn no_ai() -> [f64; 8] {
        [0.0; 8]
    }

    /// Helper: step the block N times with given dt, returning the last output.
    fn step_n(
        block: &mut SequenceController,
        n: usize,
        dt: f64,
        inputs: &[f64],
        prev_inputs: &[f64],
    ) -> Vec<f64> {
        let mut out = vec![0.0; 11];
        let mut prev = prev_inputs.to_vec();
        for _ in 0..n {
            out = block.eval(inputs, &[], dt, &prev);
            prev = inputs.to_vec();
        }
        out
    }

    // --- Parser tests ---

    #[test]
    fn parse_simple_set() {
        let programs = parse_programs("set AQ1 = 5");
        assert_eq!(programs[0].len(), 1);
        assert!(matches!(
            &programs[0][0],
            SeqLine::Set {
                target: VarRef::AQ(0),
                ..
            }
        ));
    }

    #[test]
    fn parse_sleep_seconds() {
        let programs = parse_programs("sleep 2 s");
        assert!(matches!(&programs[0][0], SeqLine::Sleep(s) if (*s - 2.0).abs() < f64::EPSILON));
    }

    #[test]
    fn parse_sleep_minutes() {
        let programs = parse_programs("sleep 3 m");
        assert!(matches!(&programs[0][0], SeqLine::Sleep(s) if (*s - 180.0).abs() < f64::EPSILON));
    }

    #[test]
    fn parse_goto() {
        let programs = parse_programs("goto 5");
        assert!(matches!(&programs[0][0], SeqLine::Goto(5)));
    }

    #[test]
    fn parse_if_endif() {
        let programs = parse_programs("if AQ1 >= 4\nset AQ2 = 1\nendif");
        assert_eq!(programs[0].len(), 3);
        assert!(matches!(&programs[0][0], SeqLine::If { .. }));
        assert!(matches!(&programs[0][2], SeqLine::EndIf));
    }

    #[test]
    fn parse_multi_sequence() {
        let text = "sequence 1\nset AQ1 = 1\nsequence 2\nset AQ2 = 2";
        let programs = parse_programs(text);
        assert_eq!(programs[0].len(), 1);
        assert_eq!(programs[1].len(), 1);
    }

    #[test]
    fn parse_expression_with_binop() {
        let programs = parse_programs("set AQ1 = AI2 + 3");
        match &programs[0][0] {
            SeqLine::Set { target, expr } => {
                assert_eq!(*target, VarRef::AQ(0));
                assert!(matches!(expr, Expr::BinOp(_, MathOp::Add, _)));
            }
            _ => panic!("expected Set"),
        }
    }

    #[test]
    fn parse_varref_cases() {
        assert_eq!(parse_varref("ai1"), Some(VarRef::AI(0)));
        assert_eq!(parse_varref("AQ8"), Some(VarRef::AQ(7)));
        assert_eq!(parse_varref("value3"), Some(VarRef::Value(2)));
        assert_eq!(parse_varref("ai9"), None);
        assert_eq!(parse_varref("bogus"), None);
    }

    // --- Block eval tests ---

    #[test]
    fn sequence_controller_basic_set() {
        // Program: set AQ1 = 5, set AQ2 = 10
        let mut block = SequenceController::new("set AQ1 = 5\nset AQ2 = 10", 100.0);
        let idle = make_inputs(&no_triggers(), &no_ai(), 0.0, 0.0);

        // Trigger sequence 1
        let trig = make_inputs(&trigger_seq(1), &no_ai(), 0.0, 0.0);
        let out = block.eval(&trig, &[], 0.0, &idle);

        // Both set commands execute in one step (non-blocking)
        assert_eq!(out[0], 5.0, "AQ1 should be 5");
        assert_eq!(out[1], 10.0, "AQ2 should be 10");
    }

    #[test]
    fn sequence_controller_set_with_ai() {
        let mut block = SequenceController::new("set AQ1 = AI1 + 3", 100.0);
        let idle = make_inputs(&no_triggers(), &no_ai(), 0.0, 0.0);

        let mut ai = no_ai();
        ai[0] = 7.0;
        let trig = make_inputs(&trigger_seq(1), &ai, 0.0, 0.0);
        let out = block.eval(&trig, &[], 0.0, &idle);

        assert_eq!(out[0], 10.0, "AQ1 = AI1(7) + 3 = 10");
    }

    #[test]
    fn sequence_controller_sleep() {
        let mut block = SequenceController::new("set AQ1 = 1\nsleep 2 s\nset AQ1 = 0", 100.0);
        let idle = make_inputs(&no_triggers(), &no_ai(), 0.0, 0.0);
        let trig = make_inputs(&trigger_seq(1), &no_ai(), 0.0, 0.0);

        // Trigger — executes "set AQ1 = 1" then hits "sleep 2 s"
        let out = block.eval(&trig, &[], 0.0, &idle);
        assert_eq!(out[0], 1.0, "AQ1 should be 1 after first set");

        // Step with dt=1.0 — still sleeping
        let running = make_inputs(&no_triggers(), &no_ai(), 0.0, 0.0);
        let out = block.eval(&running, &[], 1.0, &trig);
        assert_eq!(out[0], 1.0, "AQ1 should still be 1 during sleep");

        // Step with dt=1.5 — sleep expires (2s total), executes "set AQ1 = 0"
        let out = block.eval(&running, &[], 1.5, &running);
        assert_eq!(out[0], 0.0, "AQ1 should be 0 after sleep expires");
    }

    #[test]
    fn sequence_controller_waitcondition() {
        let mut block = SequenceController::new("waitcondition AI1 > 5\nset AQ1 = 1", 100.0);
        let idle = make_inputs(&no_triggers(), &no_ai(), 0.0, 0.0);

        // Trigger with AI1 = 3 (condition false)
        let mut ai_low = no_ai();
        ai_low[0] = 3.0;
        let trig = make_inputs(&trigger_seq(1), &ai_low, 0.0, 0.0);
        let out = block.eval(&trig, &[], 0.0, &idle);
        assert_eq!(out[0], 0.0, "AQ1 should be 0 while waiting");

        // Tick with AI1 still low
        let low_running = make_inputs(&no_triggers(), &ai_low, 0.0, 0.0);
        let out = block.eval(&low_running, &[], 0.1, &trig);
        assert_eq!(out[0], 0.0, "AQ1 should still be 0");

        // Tick with AI1 = 6 (condition true)
        let mut ai_high = no_ai();
        ai_high[0] = 6.0;
        let high_running = make_inputs(&no_triggers(), &ai_high, 0.0, 0.0);
        let out = block.eval(&high_running, &[], 0.1, &low_running);
        assert_eq!(out[0], 1.0, "AQ1 should be 1 after condition met");
    }

    #[test]
    fn sequence_controller_goto() {
        // Line 1: goto 3, Line 2: set AQ1 = 99 (skipped), Line 3: set AQ2 = 42
        let mut block = SequenceController::new("goto 3\nset AQ1 = 99\nset AQ2 = 42", 100.0);
        let idle = make_inputs(&no_triggers(), &no_ai(), 0.0, 0.0);
        let trig = make_inputs(&trigger_seq(1), &no_ai(), 0.0, 0.0);

        let out = block.eval(&trig, &[], 0.0, &idle);
        assert_eq!(out[0], 0.0, "AQ1 should be 0 (line 2 skipped by goto)");
        assert_eq!(out[1], 42.0, "AQ2 should be 42 (line 3 executed)");
    }

    #[test]
    fn sequence_controller_if_true() {
        let mut block =
            SequenceController::new("set AQ1 = 5\nif AQ1 >= 4\nset AQ2 = 1\nendif", 100.0);
        let idle = make_inputs(&no_triggers(), &no_ai(), 0.0, 0.0);
        let trig = make_inputs(&trigger_seq(1), &no_ai(), 0.0, 0.0);

        let out = block.eval(&trig, &[], 0.0, &idle);
        assert_eq!(out[0], 5.0, "AQ1 = 5");
        assert_eq!(out[1], 1.0, "AQ2 = 1 (if was true)");
    }

    #[test]
    fn sequence_controller_if_false() {
        let mut block = SequenceController::new(
            "set AQ1 = 2\nif AQ1 >= 4\nset AQ2 = 1\nendif\nset AQ3 = 99",
            100.0,
        );
        let idle = make_inputs(&no_triggers(), &no_ai(), 0.0, 0.0);
        let trig = make_inputs(&trigger_seq(1), &no_ai(), 0.0, 0.0);

        let out = block.eval(&trig, &[], 0.0, &idle);
        assert_eq!(out[0], 2.0, "AQ1 = 2");
        assert_eq!(out[1], 0.0, "AQ2 = 0 (if was false, body skipped)");
        assert_eq!(out[2], 99.0, "AQ3 = 99 (after endif, continues)");
    }

    #[test]
    fn sequence_controller_off_resets() {
        let mut block = SequenceController::new("set AQ1 = 42", 100.0);
        let idle = make_inputs(&no_triggers(), &no_ai(), 0.0, 0.0);
        let trig = make_inputs(&trigger_seq(1), &no_ai(), 0.0, 0.0);

        // Trigger and execute
        let out = block.eval(&trig, &[], 0.0, &idle);
        assert_eq!(out[0], 42.0);

        // Send Off
        let off_input = make_inputs(&no_triggers(), &no_ai(), 0.0, 1.0);
        let out = block.eval(&off_input, &[], 0.0, &trig);
        assert_eq!(out[0], 0.0, "AQ1 reset to 0 by Off");
        assert_eq!(out[8], 0.0, "current seq = 0 (idle)");
    }

    #[test]
    fn sequence_controller_setpulse() {
        let mut block = SequenceController::new("setpulse AQ1\nsleep 1 s", 100.0);
        let idle = make_inputs(&no_triggers(), &no_ai(), 0.0, 0.0);
        let trig = make_inputs(&trigger_seq(1), &no_ai(), 0.0, 0.0);

        // Trigger: setpulse AQ1 sets to 1.0, then hits sleep
        let out = block.eval(&trig, &[], 0.0, &idle);
        assert_eq!(out[0], 1.0, "AQ1 pulsed to 1");

        // Next tick: pulse should be cleared
        let running = make_inputs(&no_triggers(), &no_ai(), 0.0, 0.0);
        let out = block.eval(&running, &[], 0.1, &trig);
        assert_eq!(out[0], 0.0, "AQ1 should be 0 after pulse cleared");
    }

    #[test]
    fn sequence_controller_startsequence_return() {
        // Seq 1: set AQ1=1, startsequence 2, set AQ3=3
        // Seq 2: set AQ2=2, return
        let text = "sequence 1\nset AQ1 = 1\nstartsequence 2\nset AQ3 = 3\n\
                    sequence 2\nset AQ2 = 2\nreturn";
        let mut block = SequenceController::new(text, 100.0);
        let idle = make_inputs(&no_triggers(), &no_ai(), 0.0, 0.0);
        let trig = make_inputs(&trigger_seq(1), &no_ai(), 0.0, 0.0);

        let out = block.eval(&trig, &[], 0.0, &idle);
        assert_eq!(out[0], 1.0, "AQ1 = 1 (seq 1 line 1)");
        assert_eq!(out[1], 2.0, "AQ2 = 2 (seq 2 via startsequence)");
        assert_eq!(out[2], 3.0, "AQ3 = 3 (seq 1 after return)");
    }

    #[test]
    fn sequence_controller_outputs_sequence_and_line() {
        let mut block = SequenceController::new("set AQ1 = 1\nsleep 5 s\nset AQ1 = 2", 100.0);
        let idle = make_inputs(&no_triggers(), &no_ai(), 0.0, 0.0);
        let trig = make_inputs(&trigger_seq(1), &no_ai(), 0.0, 0.0);

        let out = block.eval(&trig, &[], 0.0, &idle);
        assert_eq!(out[8], 1.0, "current sequence = 1");
        // After executing set + entering sleep, line points to sleep's next line
        assert!(out[9] > 0.0, "current line should be > 0");
    }

    #[test]
    fn sequence_controller_multiple_triggers() {
        let text = "sequence 1\nset AQ1 = 10\nsequence 2\nset AQ2 = 20";
        let mut block = SequenceController::new(text, 100.0);
        let idle = make_inputs(&no_triggers(), &no_ai(), 0.0, 0.0);

        // Trigger sequence 2
        let trig2 = make_inputs(&trigger_seq(2), &no_ai(), 0.0, 0.0);
        let out = block.eval(&trig2, &[], 0.0, &idle);
        assert_eq!(out[0], 0.0, "AQ1 should be 0 (seq 1 not triggered)");
        assert_eq!(out[1], 20.0, "AQ2 should be 20 (seq 2 triggered)");
    }

    #[test]
    fn sequence_controller_value_variables() {
        let text = "set value1 = 42\nset AQ1 = value1";
        let mut block = SequenceController::new(text, 100.0);
        let idle = make_inputs(&no_triggers(), &no_ai(), 0.0, 0.0);
        let trig = make_inputs(&trigger_seq(1), &no_ai(), 0.0, 0.0);

        let out = block.eval(&trig, &[], 0.0, &idle);
        assert_eq!(out[0], 42.0, "AQ1 = value1 = 42");
    }

    #[test]
    fn sequence_controller_math_expression() {
        let text = "set AQ1 = 10 - 3 * 2";
        let mut block = SequenceController::new(text, 100.0);
        let idle = make_inputs(&no_triggers(), &no_ai(), 0.0, 0.0);
        let trig = make_inputs(&trigger_seq(1), &no_ai(), 0.0, 0.0);

        let out = block.eval(&trig, &[], 0.0, &idle);
        // With left-to-right scanning: "10 - 3 * 2"
        // The parser splits on rightmost + or - first, giving: "10" - "3 * 2"
        // Then "3 * 2" = 6, so result = 10 - 6 = 4
        assert_eq!(out[0], 4.0, "10 - 3 * 2 = 4 (correct precedence)");
    }

    #[test]
    fn sequence_controller_state_roundtrip() {
        let mut block = SequenceController::new("set AQ1 = 42", 100.0);
        let idle = make_inputs(&no_triggers(), &no_ai(), 0.0, 0.0);
        let trig = make_inputs(&trigger_seq(1), &no_ai(), 0.0, 0.0);
        block.eval(&trig, &[], 0.0, &idle);

        let state = block.state().unwrap();
        let mut block2 = SequenceController::new("set AQ1 = 42", 100.0);
        block2.restore(&state);
        assert_eq!(block2.outputs[0], 42.0, "output restored");
    }

    #[test]
    fn sequence_controller_waitcondition_ne() {
        let mut block = SequenceController::new("waitcondition AI1 != 0\nset AQ1 = 1", 100.0);
        let idle = make_inputs(&no_triggers(), &no_ai(), 0.0, 0.0);

        // Trigger with AI1 = 0 (condition false: 0 != 0 is false)
        let trig = make_inputs(&trigger_seq(1), &no_ai(), 0.0, 0.0);
        let out = block.eval(&trig, &[], 0.0, &idle);
        assert_eq!(out[0], 0.0, "waiting, AQ1 stays 0");

        // AI1 = 5 (condition true)
        let mut ai = no_ai();
        ai[0] = 5.0;
        let high = make_inputs(&no_triggers(), &ai, 0.0, 0.0);
        let out = block.eval(&high, &[], 0.1, &trig);
        assert_eq!(out[0], 1.0, "condition met, AQ1 = 1");
    }

    #[test]
    fn sequence_controller_nested_if() {
        let text = "set AQ1 = 5\nif AQ1 > 3\nif AQ1 > 10\nset AQ2 = 1\nendif\nset AQ3 = 1\nendif";
        let mut block = SequenceController::new(text, 100.0);
        let idle = make_inputs(&no_triggers(), &no_ai(), 0.0, 0.0);
        let trig = make_inputs(&trigger_seq(1), &no_ai(), 0.0, 0.0);

        let out = block.eval(&trig, &[], 0.0, &idle);
        assert_eq!(out[0], 5.0, "AQ1 = 5");
        assert_eq!(out[1], 0.0, "AQ2 = 0 (inner if false: 5 > 10 is false)");
        assert_eq!(out[2], 1.0, "AQ3 = 1 (outer if true, after inner endif)");
    }

    #[test]
    fn sequence_controller_division_by_zero() {
        let text = "set AQ1 = 10 / 0";
        let mut block = SequenceController::new(text, 100.0);
        let idle = make_inputs(&no_triggers(), &no_ai(), 0.0, 0.0);
        let trig = make_inputs(&trigger_seq(1), &no_ai(), 0.0, 0.0);

        let out = block.eval(&trig, &[], 0.0, &idle);
        assert_eq!(out[0], 0.0, "division by zero returns 0");
    }

    #[test]
    fn sequence_controller_empty_program() {
        let mut block = SequenceController::new("", 100.0);
        let idle = make_inputs(&no_triggers(), &no_ai(), 0.0, 0.0);
        let trig = make_inputs(&trigger_seq(1), &no_ai(), 0.0, 0.0);

        // Should not crash, outputs stay at 0
        let out = block.eval(&trig, &[], 0.0, &idle);
        assert_eq!(out[0], 0.0);
        assert_eq!(out[8], 0.0, "no sequence running");
    }
}
