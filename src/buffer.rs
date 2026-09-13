use serde::Serialize;

/// Tamanho de cada metade do buffer duplo (N caracteres + sentinela).
pub const BUFFER_SIZE: usize = 32;

/// Sentinela colocada ao final de cada metade (técnica dos dois buffers).
const SENTINEL: u8 = 0;

#[derive(Debug, Clone, Serialize)]
pub struct BufferEvent {
    pub evento: String,
    pub detalhe: String,
}

#[derive(Clone)]
struct RetractFrame {
    buf: usize,
    idx: usize,
    line: usize,
    column: usize,
    pending_len: usize,
}

/// Leitura do fonte com dois buffers e dois ponteiros (begin / forward).
pub struct TwinBuffer {
    buffers: [Vec<u8>; 2],
    forward_buf: usize,
    forward_idx: usize,
    begin_buf: usize,
    begin_idx: usize,
    /// Bytes do lexema que já saíram de um buffer prestes a ser recarregado.
    pending: Vec<u8>,
    source: Vec<u8>,
    next_load: usize,
    line: usize,
    column: usize,
    begin_line: usize,
    begin_column: usize,
    retract_stack: Vec<RetractFrame>,
    pub events: Vec<BufferEvent>,
}

impl TwinBuffer {
    pub fn new(source: impl Into<String>) -> Self {
        let source = source.into().into_bytes();
        let mut buf = Self {
            buffers: [Vec::new(), Vec::new()],
            forward_buf: 0,
            forward_idx: 0,
            begin_buf: 0,
            begin_idx: 0,
            pending: Vec::new(),
            source,
            next_load: 0,
            line: 1,
            column: 1,
            begin_line: 1,
            begin_column: 1,
            retract_stack: Vec::new(),
            events: Vec::new(),
        };
        buf.load(0);
        buf
    }

    fn load(&mut self, half: usize) {
        let start = self.next_load;
        let remaining = self.source.len().saturating_sub(start);
        let n = remaining.min(BUFFER_SIZE);
        self.buffers[half].clear();
        if n > 0 {
            self.buffers[half].extend_from_slice(&self.source[start..start + n]);
            self.next_load += n;
        }
        self.buffers[half].push(SENTINEL);
        self.events.push(BufferEvent {
            evento: "carregar_buffer".into(),
            detalhe: format!("metade {half} ← {n} byte(s) do fonte (offset {start})"),
        });
    }

    fn current_byte(&self) -> u8 {
        self.buffers[self.forward_buf]
            .get(self.forward_idx)
            .copied()
            .unwrap_or(SENTINEL)
    }

    fn is_real_eof(&self) -> bool {
        self.current_byte() == SENTINEL && self.next_load >= self.source.len()
    }

    fn switch_buffer(&mut self) {
        let from = self.forward_buf;
        let start = if self.begin_buf == from {
            self.begin_idx
        } else {
            0
        };
        self.pending.extend(
            self.buffers[from]
                .iter()
                .skip(start)
                .copied()
                .filter(|&b| b != SENTINEL),
        );

        let to = 1 - from;
        self.events.push(BufferEvent {
            evento: "trocar_buffer".into(),
            detalhe: format!("forward: metade {from} → metade {to}"),
        });
        self.forward_buf = to;
        self.forward_idx = 0;
        self.begin_buf = to;
        self.begin_idx = 0;
        self.load(to);
    }

    /// Avança o ponteiro forward e devolve o caractere lido.
    pub fn advance(&mut self) -> Option<char> {
        loop {
            if self.is_real_eof() {
                return None;
            }
            if self.current_byte() == SENTINEL {
                self.switch_buffer();
                continue;
            }

            let byte = self.current_byte();
            self.retract_stack.push(RetractFrame {
                buf: self.forward_buf,
                idx: self.forward_idx,
                line: self.line,
                column: self.column,
                pending_len: self.pending.len(),
            });
            if self.retract_stack.len() > 8 {
                self.retract_stack.remove(0);
            }

            self.forward_idx += 1;
            let ch = byte as char;
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            return Some(ch);
        }
    }

    pub fn peek(&mut self) -> Option<char> {
        let ch = self.advance()?;
        self.retract();
        Some(ch)
    }

    /// Recua o ponteiro forward em um caractere.
    pub fn retract(&mut self) {
        if let Some(frame) = self.retract_stack.pop() {
            self.forward_buf = frame.buf;
            self.forward_idx = frame.idx;
            self.line = frame.line;
            self.column = frame.column;
            self.pending.truncate(frame.pending_len);
        }
    }

    /// Marca o início do próximo lexema (begin ← forward).
    pub fn mark_begin(&mut self) {
        self.pending.clear();
        self.begin_buf = self.forward_buf;
        self.begin_idx = self.forward_idx;
        self.begin_line = self.line;
        self.begin_column = self.column;
    }

    pub fn begin_position(&self) -> (usize, usize) {
        (self.begin_line, self.begin_column)
    }

    /// Extrai o lexema no intervalo [begin, forward).
    pub fn lexeme(&self) -> String {
        let mut bytes = self.pending.clone();
        if self.begin_buf == self.forward_buf {
            let start = self.begin_idx.min(self.buffers[self.begin_buf].len());
            let end = self.forward_idx.min(self.buffers[self.forward_buf].len());
            if end >= start {
                bytes.extend(
                    self.buffers[self.begin_buf][start..end]
                        .iter()
                        .copied()
                        .filter(|&b| b != SENTINEL),
                );
            }
        } else {
            bytes.extend(
                self.buffers[self.begin_buf]
                    .iter()
                    .skip(self.begin_idx)
                    .copied()
                    .filter(|&b| b != SENTINEL),
            );
            bytes.extend(
                self.buffers[self.forward_buf]
                    .iter()
                    .take(self.forward_idx)
                    .copied()
                    .filter(|&b| b != SENTINEL),
            );
        }
        String::from_utf8_lossy(&bytes).into_owned()
    }
}
