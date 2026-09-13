use std::collections::BTreeMap;

use serde::Serialize;

/// Tabela de símbolos: apenas identificadores (palavras-chave ficam de fora).
#[derive(Debug, Default, Clone, Serialize)]
pub struct SymbolTable {
    entries: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SymbolEntry {
    pub identificador: String,
    pub ocorrencias: usize,
}

impl SymbolTable {
    pub fn insert(&mut self, name: &str) {
        *self.entries.entry(name.to_string()).or_insert(0) += 1;
    }

    pub fn entries(&self) -> Vec<SymbolEntry> {
        self.entries
            .iter()
            .map(|(identificador, ocorrencias)| SymbolEntry {
                identificador: identificador.clone(),
                ocorrencias: *ocorrencias,
            })
            .collect()
    }
}
